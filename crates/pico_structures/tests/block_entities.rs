use blocks_report::BlockStateLookup;
use minecraft_protocol::prelude::{BinaryWriter, Coordinates, EncodePacket, ProtocolVersion};
use pico_nbt::{CompressionType, IndexMap, NbtOptions, Value};
use pico_structures::prelude::Schematic;
use serde_json::json;
use std::sync::atomic::{AtomicUsize, Ordering};

fn compound(value: &mut Value) -> &mut IndexMap<String, Value> {
    match value {
        Value::Compound(fields) => fields,
        _ => panic!("expected compound"),
    }
}

fn field<'a>(value: &'a Value, key: &str) -> &'a Value {
    value.get_compound().unwrap().get(key).unwrap()
}

fn load(version: i32, data_version: i32, id: &str, state: &str, data: Value) -> Schematic {
    let mut entity = pico_nbt::json_to_nbt(json!({"Id": id})).unwrap();
    compound(&mut entity).insert("Pos".into(), Value::IntArray(vec![1, 0, 0]));
    if version == 3 {
        compound(&mut entity).insert("Data".into(), data);
    } else {
        compound(&mut entity).extend(data.get_compound().unwrap().clone());
    }
    let mut root = pico_nbt::json_to_nbt(json!({
        "Version": version, "DataVersion": data_version,
        "Width": 2, "Height": 1, "Length": 1,
        "PaletteMax": 2, "Palette": {"minecraft:air": 0, state: 1}
    }))
    .unwrap();
    let fields = compound(&mut root);
    fields.insert("BlockData".into(), Value::ByteArray(vec![0, 1]));
    fields.insert("BlockEntities".into(), Value::List(vec![entity]));
    if version == 3 {
        let mut blocks = IndexMap::new();
        blocks.insert("Palette".into(), fields.swap_remove("Palette").unwrap());
        blocks.insert("Data".into(), fields.swap_remove("BlockData").unwrap());
        blocks.insert(
            "BlockEntities".into(),
            fields.swap_remove("BlockEntities").unwrap(),
        );
        fields.insert("Blocks".into(), Value::Compound(blocks));
        root = Value::Compound(IndexMap::from([("Schematic".into(), root)]));
    }
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let path = std::env::temp_dir().join(format!(
        "pico-block-entities-{}-{}.schem",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::write(
        &path,
        root.to_byte(
            CompressionType::Gzip,
            NbtOptions::new().dynamic_lists(true),
            None,
        )
        .unwrap(),
    )
    .unwrap();
    let mapping = blocks_report::load_internal_mapping().unwrap();
    let schematic = Schematic::load_schematic_file(&path, &mapping);
    std::fs::remove_file(path).unwrap();
    let schematic = schematic.unwrap();
    let lookup = BlockStateLookup::new(&mapping);
    assert_eq!(
        schematic
            .get_block_state_id(Coordinates::new(1, 0, 0))
            .internal_id(),
        lookup.parse_state_string(state).unwrap().internal_id()
    );
    assert_eq!(schematic.get_block_entities().len(), 1);
    let position = schematic.get_block_entities()[0].get_position();
    assert_eq!((position.x(), position.y(), position.z()), (1, 0, 0));
    schematic
}

fn transmitted(schematic: &Schematic, version: ProtocolVersion) -> Value {
    let data = schematic.get_block_entities()[0].to_nbt(version).unwrap();
    let mut writer = BinaryWriter::new();
    data.encode(&mut writer, version).unwrap();
    let options = NbtOptions::new()
        .nameless_root(version.is_after_inclusive(ProtocolVersion::V1_20_2))
        .dynamic_lists(version.is_after_inclusive(ProtocolVersion::V1_21_5));
    pico_nbt::from_slice_with_options(&writer.into_inner(), options)
        .unwrap()
        .1
}

#[test]
fn schematic_versions_preserve_head_profiles_and_nbt_types() {
    let mut data = pico_nbt::json_to_nbt(json!({
        "profile": {"name": "Test", "properties": [{"name": "textures", "value": "dGV4dHVyZQ==", "signature": "signature"}]},
        "note_block_sound": "minecraft:block.note_block.bell"
    })).unwrap();
    compound(compound(&mut data).get_mut("profile").unwrap())
        .insert("id".into(), Value::IntArray(vec![1, -2, 3, -4]));
    for format in [2, 3] {
        let schematic = load(
            format,
            4325,
            "minecraft:skull",
            "minecraft:player_head[rotation=7]",
            data.clone(),
        );
        for version in [
            ProtocolVersion::V1_20_5,
            ProtocolVersion::V1_21_5,
            ProtocolVersion::V26_1,
        ] {
            assert_eq!(transmitted(&schematic, version), data);
        }
        let old = transmitted(&schematic, ProtocolVersion::V1_15);
        assert!(field(field(&old, "SkullOwner"), "Id").get_str().is_some());
        let old = load(
            format,
            2230,
            "minecraft:skull",
            "minecraft:player_head[rotation=7]",
            old,
        );
        assert_eq!(transmitted(&old, ProtocolVersion::V1_20_5), data);
        let legacy = transmitted(&schematic, ProtocolVersion::V1_20);
        let owner = field(&legacy, "SkullOwner");
        assert_eq!(field(owner, "Id"), field(field(&data, "profile"), "id"));
        let textures = field(field(owner, "Properties"), "textures")
            .get_list()
            .unwrap();
        assert_eq!(
            field(&textures[0], "Value"),
            &Value::String("dGV4dHVyZQ==".into())
        );
        assert_eq!(
            field(&textures[0], "Signature"),
            &Value::String("signature".into())
        );
        let reloaded = load(
            format,
            3465,
            "skull",
            "minecraft:player_head[rotation=7]",
            legacy,
        );
        assert_eq!(transmitted(&reloaded, ProtocolVersion::V26_1), data);
    }
}

#[test]
fn arbitrary_block_entity_properties_remain_lossless() {
    let data = Value::Compound(IndexMap::from([
        ("CustomName".into(), Value::String("\"Quoted name\"".into())),
        (
            "Patterns".into(),
            Value::List(vec![Value::Compound(IndexMap::from([
                ("Pattern".into(), "bs".into()),
                ("Color".into(), Value::Int(5)),
            ]))]),
        ),
        ("LongValue".into(), Value::Long(1)),
        ("ByteData".into(), Value::ByteArray(vec![0, 255])),
        ("LongData".into(), Value::LongArray(vec![1, i64::MAX])),
    ]));
    for format in [2, 3] {
        let schematic = load(
            format,
            3465,
            "minecraft:banner",
            "minecraft:white_banner[rotation=3]",
            data.clone(),
        );
        assert_eq!(transmitted(&schematic, ProtocolVersion::V1_20), data);
    }
}

#[test]
fn legacy_signs_keep_formatted_text_without_optional_color_or_glow() {
    let text = json!({"text": "Welcome", "bold": true, "extra": [{"text": "!", "color": "gold"}]})
        .to_string();
    for format in [2, 3] {
        let data = pico_nbt::json_to_nbt(json!({"Text1": text, "Text2": "\"Quoted\""})).unwrap();
        let schematic = load(
            format,
            2975,
            "sign",
            "minecraft:oak_sign[rotation=4,waterlogged=true]",
            data,
        );
        let legacy = transmitted(&schematic, ProtocolVersion::V1_18_2);
        assert_eq!(field(&legacy, "Text1"), &Value::String(text.clone()));
        for version in [
            ProtocolVersion::V1_20,
            ProtocolVersion::V1_21_4,
            ProtocolVersion::V1_21_5,
            ProtocolVersion::V26_1,
        ] {
            let data = transmitted(&schematic, version);
            let lines = field(field(&data, "front_text"), "messages")
                .get_list()
                .unwrap();
            assert_eq!(lines.len(), 4);
            if version.is_after_inclusive(ProtocolVersion::V1_21_5) {
                assert_eq!(field(&lines[0], "bold"), &Value::Byte(1));
                assert_eq!(lines[1], Value::String("Quoted".into()));
            } else {
                assert_eq!(lines[0], Value::String(text.clone()));
            }
            assert_eq!(
                field(field(&data, "back_text"), "messages")
                    .get_list()
                    .unwrap()
                    .len(),
                4
            );
        }
    }
}

#[test]
fn native_signs_preserve_both_faces_filtered_text_and_literal_json() {
    let data = pico_nbt::json_to_nbt(json!({
        "is_waxed": true,
        "front_text": {"color": "blue", "has_glowing_text": true,
            "messages": [{"text": "Hello", "bold": true}, "\"literal\"", "", ""],
            "filtered_messages": ["filtered", "", "", ""]},
        "back_text": {"color": "red", "has_glowing_text": false,
            "messages": ["Back", "", "", ""]}
    }))
    .unwrap();
    let schematic = load(
        3,
        4325,
        "minecraft:hanging_sign",
        "minecraft:oak_hanging_sign[attached=false,rotation=2,waterlogged=false]",
        data.clone(),
    );
    assert_eq!(transmitted(&schematic, ProtocolVersion::V26_1), data);
    let legacy = transmitted(&schematic, ProtocolVersion::V1_19_4);
    let line: serde_json::Value =
        serde_json::from_str(field(&legacy, "Text1").get_str().unwrap()).unwrap();
    assert_eq!(line["bold"], json!(true));
    assert_eq!(field(&legacy, "Color"), &Value::String("blue".into()));
    assert_eq!(field(&legacy, "GlowingText"), &Value::Byte(1));
    let filtered: serde_json::Value =
        serde_json::from_str(field(&legacy, "FilteredText1").get_str().unwrap()).unwrap();
    assert_eq!(filtered, json!("filtered"));
    let modern = transmitted(&schematic, ProtocolVersion::V1_20);
    let reloaded = load(
        3,
        3465,
        "minecraft:hanging_sign",
        "minecraft:oak_hanging_sign[attached=false,rotation=2,waterlogged=false]",
        modern,
    );
    assert_eq!(transmitted(&reloaded, ProtocolVersion::V1_21_5), data);
}

#[test]
fn named_head_profiles_convert_to_legacy_compounds() {
    let data = pico_nbt::json_to_nbt(json!({"profile": "Test"})).unwrap();
    let schematic = load(
        3,
        4325,
        "minecraft:skull",
        "minecraft:player_head[rotation=0]",
        data,
    );
    let legacy = transmitted(&schematic, ProtocolVersion::V1_18_2);
    assert_eq!(
        field(field(&legacy, "SkullOwner"), "Name"),
        &Value::String("Test".into())
    );
}

#[test]
fn malformed_sign_faces_return_an_error_instead_of_panicking() {
    let data = pico_nbt::json_to_nbt(json!({"front_text": 1})).unwrap();
    let schematic = load(
        3,
        4325,
        "minecraft:sign",
        "minecraft:oak_sign[rotation=0,waterlogged=false]",
        data,
    );
    assert!(
        schematic.get_block_entities()[0]
            .to_nbt(ProtocolVersion::V26_1)
            .is_err()
    );
}
