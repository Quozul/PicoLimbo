use pico_registries::Identifier;
use pico_registries::registry_provider::{Dimension, RegistryProvider, RuntimeRegistryProvider};
use protocol_version::protocol_version::ProtocolVersion;
use std::fmt::Write as FmtWrite;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::{env, fs};

// Helper to write binary blobs to files and return the rust code to include them
struct BlobWriter {
    out_dir: PathBuf,
    counter: usize,
}

impl BlobWriter {
    fn new(out_dir: &str) -> Self {
        Self {
            out_dir: PathBuf::from(out_dir),
            counter: 0,
        }
    }

    fn save_blob(&mut self, data: &[u8]) -> std::io::Result<String> {
        let filename = format!("blob_{}.bin", self.counter);
        self.counter += 1;

        let path = self.out_dir.join(&filename);
        fs::write(&path, data)?;

        // Return the Rust code string that loads this file
        // We use include_bytes! which returns a &'static [u8; N], coercible to &[u8]
        Ok(format!(
            "include_bytes!(concat!(env!(\"OUT_DIR\"), \"/{filename}\"))"
        ))
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = env::var("OUT_DIR")?;
    let dest_path = Path::new(&out_dir).join("precomputed_registries.rs");
    let mut file = BufWriter::new(File::create(&dest_path)?);

    // Initialize our blob writer
    let mut blob_writer = BlobWriter::new(&out_dir);

    write_header(&mut file)?;

    // 1. Biome IDs (Small integers, keep as source)
    build_biome_map(&mut file)?;

    // 2. Dimension Codecs (Large blobs -> Optimized)
    build_dimension_codec_map(&mut file, &mut blob_writer)?;

    // 3. Registry Codec (Large blobs -> Optimized)
    build_registry_codec_map(&mut file, &mut blob_writer)?;

    // 4. Dimension Info (Mixed data, small)
    build_dimension_info_map(&mut file)?;

    // 5. Registry Data (Contains NBT blobs -> Optimized)
    build_registry_data_map(&mut file, &mut blob_writer)?;

    // 6. Tagged Registries (Integer lists, keep as source)
    build_tagged_registries_map(&mut file)?;

    Ok(())
}

fn write_header(w: &mut impl Write) -> std::io::Result<()> {
    writeln!(
        w,
        r"
pub struct StaticDimensionInfo {{
    pub height: i32,
    pub min_y: i32,
    pub protocol_id: u32,
    pub registry_key: &'static str,
}}

pub struct StaticRegistryDataEntry {{
    pub entry_id: &'static str,
    pub nbt_bytes: &'static [u8],
}}

pub struct StaticTaggedRegistry {{
    pub registry_id: &'static str,
    pub tags: &'static [StaticRegistryTag],
}}

pub struct StaticRegistryTag {{
    pub identifier: &'static str,
    pub ids: &'static [u32],
}}

pub struct StaticDimensionCodecs {{
    pub overworld: &'static [u8],
    pub nether: &'static [u8],
    pub end: &'static [u8],
}}
"
    )
}

fn build_biome_map(w: &mut impl Write) -> anyhow::Result<()> {
    let mut entries = Vec::new();
    let plains_id = Identifier::vanilla_unchecked("plains");
    let registry_provider = RuntimeRegistryProvider;

    for version in ProtocolVersion::ALL_VERSION {
        if let Ok(id) = registry_provider.get_biome_protocol_id(*version, &plains_id) {
            entries.push((ver_key(*version), id.to_string()));
        }
    }

    let mut map = phf_codegen::Map::new();
    for (k, v) in &entries {
        map.entry(k, v);
    }

    writeln!(
        w,
        "pub static BIOME_IDS: phf::Map<&'static str, u32> = \n{};\n",
        map.build()
    )?;
    Ok(())
}

fn build_dimension_codec_map(w: &mut impl Write, bw: &mut BlobWriter) -> anyhow::Result<()> {
    let mut entries = Vec::new();
    let registry_provider = RuntimeRegistryProvider;

    for version in ProtocolVersion::ALL_VERSION {
        let overworld =
            registry_provider.get_dimension_codec_v1_16_2(*version, &Dimension::Overworld);
        let nether = registry_provider.get_dimension_codec_v1_16_2(*version, &Dimension::TheNether);
        let end = registry_provider.get_dimension_codec_v1_16_2(*version, &Dimension::TheEnd);

        if let (Ok(ov), Ok(ne), Ok(en)) = (overworld, nether, end) {
            let ov_code = bw.save_blob(&ov)?;
            let ne_code = bw.save_blob(&ne)?;
            let en_code = bw.save_blob(&en)?;

            let struct_lit = format!(
                "StaticDimensionCodecs {{ overworld: {ov_code}, nether: {ne_code}, end: {en_code} }}"
            );
            entries.push((ver_key(*version), struct_lit));
        }
    }

    let mut map = phf_codegen::Map::new();
    for (k, v) in &entries {
        map.entry(k, v);
    }

    writeln!(
        w,
        "pub static DIMENSION_CODECS: phf::Map<&'static str, StaticDimensionCodecs> = \n{};\n",
        map.build()
    )?;
    Ok(())
}

fn build_registry_codec_map(w: &mut impl Write, bw: &mut BlobWriter) -> anyhow::Result<()> {
    let mut entries = Vec::new();
    let registry_provider = RuntimeRegistryProvider;

    for version in ProtocolVersion::ALL_VERSION {
        if let Ok(bytes) = registry_provider.get_registry_codec_v1_16(*version) {
            let code = bw.save_blob(&bytes)?;
            entries.push((ver_key(*version), code));
        }
    }

    let mut map = phf_codegen::Map::new();
    for (k, v) in &entries {
        map.entry(k, v);
    }

    writeln!(
        w,
        "pub static REGISTRY_CODECS: phf::Map<&'static str, &'static [u8]> = \n{};\n",
        map.build()
    )?;
    Ok(())
}

fn build_dimension_info_map(w: &mut impl Write) -> anyhow::Result<()> {
    let mut entries = Vec::new();
    let dims = [
        "minecraft:overworld",
        "minecraft:the_nether",
        "minecraft:the_end",
    ];
    let registry_provider = RuntimeRegistryProvider;

    for version in ProtocolVersion::ALL_VERSION {
        for dim_str in dims {
            let dim_id = Identifier::vanilla_unchecked(
                dim_str.strip_prefix("minecraft:").unwrap_or(dim_str),
            );

            if let Ok(info) = registry_provider.get_dimension_info(*version, &dim_id) {
                let compound_key = format!("{}|{}", ver_key(*version), dim_str);
                let reg_key_str = format!("{}", info.registry_key);

                let value = format!(
                    "StaticDimensionInfo {{ height: {}, min_y: {}, protocol_id: {}, registry_key: {:?} }}",
                    info.height, info.min_y, info.protocol_id, reg_key_str
                );
                entries.push((compound_key, value));
            }
        }
    }

    let mut map = phf_codegen::Map::new();
    for (k, v) in &entries {
        map.entry(k, v);
    }

    writeln!(
        w,
        "pub static DIMENSION_INFOS: phf::Map<&'static str, StaticDimensionInfo> = \n{};\n",
        map.build()
    )?;
    Ok(())
}

fn build_registry_data_map(w: &mut impl Write, bw: &mut BlobWriter) -> anyhow::Result<()> {
    let mut entries = Vec::new();
    let registry_provider = RuntimeRegistryProvider;

    for version in ProtocolVersion::ALL_VERSION {
        if let Ok(data) = registry_provider.get_registry_data_v1_20_5(*version) {
            let mut vec_str = String::new();
            write!(vec_str, "&[")?;

            for (ident, inner_entries) in data {
                write!(vec_str, "({:?}, &[", ident.thing)?;

                for entry in inner_entries {
                    let nbt_code = bw.save_blob(&entry.nbt_bytes)?;

                    write!(
                        vec_str,
                        "StaticRegistryDataEntry {{ entry_id: {:?}, nbt_bytes: {nbt_code} }},",
                        entry.entry_id.thing
                    )?;
                }
                write!(vec_str, "]),")?;
            }
            write!(vec_str, "]")?;

            entries.push((ver_key(*version), vec_str));
        }
    }

    let mut map = phf_codegen::Map::new();
    for (k, v) in &entries {
        map.entry(k, v);
    }

    writeln!(
        w,
        "pub static REGISTRY_DATA: phf::Map<&'static str, &'static [(&'static str, &'static [StaticRegistryDataEntry])]> = \n{};\n",
        map.build()
    )?;
    Ok(())
}

fn build_tagged_registries_map(w: &mut impl Write) -> anyhow::Result<()> {
    let mut entries = Vec::new();
    let registry_provider = RuntimeRegistryProvider;

    for version in ProtocolVersion::ALL_VERSION {
        if let Ok(registries) = registry_provider.get_tagged_registries(*version) {
            let mut vec_str = String::new();
            write!(vec_str, "&[")?;

            for reg in registries {
                write!(
                    vec_str,
                    "StaticTaggedRegistry {{ registry_id: {:?}, tags: &[",
                    reg.registry_id.thing
                )?;

                for tag in reg.tags {
                    // ids are Vec<u32>. Writing these as text `&[1, 2]` is usually acceptable
                    // compared to u8 arrays, as the token count is 1/4th.
                    write!(
                        vec_str,
                        "StaticRegistryTag {{ identifier: {:?}, ids: &{:?} }},",
                        tag.identifier.thing, tag.ids
                    )?;
                }
                write!(vec_str, "] }},")?;
            }
            write!(vec_str, "]")?;

            entries.push((ver_key(*version), vec_str));
        }
    }

    let mut map = phf_codegen::Map::new();
    for (k, v) in &entries {
        map.entry(k, v);
    }

    writeln!(
        w,
        "pub static TAGGED_REGISTRIES: phf::Map<&'static str, &'static [StaticTaggedRegistry]> = \n{};\n",
        map.build()
    )?;
    Ok(())
}

fn ver_key(v: ProtocolVersion) -> String {
    format!("{v:?}")
}
