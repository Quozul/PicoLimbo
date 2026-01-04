use pico_registries::Identifier;
use pico_registries::registry_provider::{Dimension, RegistryProvider, RuntimeRegistryProvider};
use protocol_version::protocol_version::ProtocolVersion;
use std::fmt::Write as FmtWrite;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::{env, fs};

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

        Ok(format!(
            "include_bytes!(concat!(env!(\"OUT_DIR\"), \"/{filename}\"))"
        ))
    }
}

fn versions_with_registries() -> impl Iterator<Item = &'static ProtocolVersion> {
    ProtocolVersion::ALL_VERSION
        .iter()
        .filter(|v| v.has_registries())
}

fn main() -> anyhow::Result<()> {
    let out_dir = env::var("OUT_DIR")?;
    let dest_path = Path::new(&out_dir).join("precomputed_registries.rs");
    let mut file = BufWriter::new(File::create(&dest_path)?);
    let mut blob_writer = BlobWriter::new(&out_dir);

    write_header(&mut file)?;
    build_biome_map(&mut file)?;
    build_dimension_codec_map(&mut file, &mut blob_writer)?;
    build_registry_codec_map(&mut file, &mut blob_writer)?;
    build_dimension_info_map(&mut file)?;
    build_registry_data_map(&mut file, &mut blob_writer)?;
    build_tagged_registries_map(&mut file)?;

    Ok(())
}

fn load_registry_provider(
    protocol_version: ProtocolVersion,
) -> anyhow::Result<RuntimeRegistryProvider> {
    let start_dir = env::var_os("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .ok_or_else(|| anyhow::anyhow!("Missing `CARGO_MANIFEST_DIR`"))?
        .join("../../data/generated");
    Ok(RuntimeRegistryProvider::new(&start_dir, protocol_version)?)
}

fn write_header(w: &mut impl Write) -> anyhow::Result<()> {
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
"
    )?;

    writeln!(w, "pub struct StaticDimensionInfos {{")?;
    for (field_name, _) in DIMENSIONS {
        writeln!(w, "    pub {field_name}: StaticDimensionInfo,")?;
    }
    writeln!(w, "}}\n")?;

    writeln!(w, "pub struct StaticDimensionCodecs {{")?;
    for (field_name, _) in DIMENSIONS {
        writeln!(w, "    pub {field_name}: &'static [u8],")?;
    }
    writeln!(w, "}}\n")?;

    Ok(())
}

fn build_biome_map(w: &mut impl Write) -> anyhow::Result<()> {
    let mut entries = Vec::new();
    let plains_id = Identifier::vanilla_unchecked("plains");

    for version in versions_with_registries() {
        let registry_provider = load_registry_provider(*version)?;
        if let Ok(id) = registry_provider.get_biome_protocol_id(&plains_id) {
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

const DIMENSIONS: &[(&str, Dimension)] = &[
    ("overworld", Dimension::Overworld),
    ("nether", Dimension::Nether),
    ("end", Dimension::End),
];

fn build_per_version_dim_map<F>(
    w: &mut impl Write,
    map_name: &str,
    struct_type_name: &str,
    mut value_generator: F,
) -> anyhow::Result<()>
where
    F: FnMut(ProtocolVersion, Dimension) -> anyhow::Result<String>,
{
    let mut entries = Vec::new();

    for version in versions_with_registries() {
        let field_results: Result<Vec<_>, _> = DIMENSIONS
            .iter()
            .map(|(field_name, dim)| {
                value_generator(*version, *dim).map(|value| format!("{field_name}: {value}"))
            })
            .collect();

        if let Ok(fields) = field_results {
            let struct_lit = format!("{} {{ {} }}", struct_type_name, fields.join(", "));
            entries.push((ver_key(*version), struct_lit));
        }
    }

    let mut map = phf_codegen::Map::new();
    for (k, v) in &entries {
        map.entry(k, v);
    }

    writeln!(
        w,
        "pub static {}: phf::Map<&'static str, {}> = \n{};\n",
        map_name,
        struct_type_name,
        map.build()
    )?;

    Ok(())
}

fn build_dimension_codec_map(w: &mut impl Write, bw: &mut BlobWriter) -> anyhow::Result<()> {
    build_per_version_dim_map(
        w,
        "DIMENSION_CODECS",
        "StaticDimensionCodecs",
        |ver, dim| {
            let registry_provider = load_registry_provider(ver)?;
            let codec = registry_provider.get_dimension_codec_v1_16_2(dim)?;
            let blob_id = bw.save_blob(&codec)?;
            Ok(blob_id)
        },
    )
}

fn build_dimension_info_map(w: &mut impl Write) -> anyhow::Result<()> {
    build_per_version_dim_map(w, "DIMENSION_INFOS", "StaticDimensionInfos", |ver, dim| {
        let registry_provider = load_registry_provider(ver)?;
        let info = registry_provider.get_dimension_info(dim)?;

        Ok(format!(
            "StaticDimensionInfo {{ height: {}, min_y: {}, protocol_id: {}, registry_key: {:?} }}",
            info.height,
            info.min_y,
            info.protocol_id,
            info.registry_key.to_string()
        ))
    })
}

fn build_registry_codec_map(w: &mut impl Write, bw: &mut BlobWriter) -> anyhow::Result<()> {
    let mut entries = Vec::new();

    for version in versions_with_registries() {
        let registry_provider = load_registry_provider(*version)?;
        if let Ok(bytes) = registry_provider.get_registry_codec_v1_16() {
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

fn build_registry_data_map(w: &mut impl Write, bw: &mut BlobWriter) -> anyhow::Result<()> {
    let mut entries = Vec::new();

    for version in versions_with_registries() {
        let registry_provider = load_registry_provider(*version)?;
        if let Ok(data) = registry_provider.get_registry_data_v1_20_5() {
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

    for version in versions_with_registries() {
        let registry_provider = load_registry_provider(*version)?;
        if let Ok(registries) = registry_provider.get_tagged_registries() {
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
