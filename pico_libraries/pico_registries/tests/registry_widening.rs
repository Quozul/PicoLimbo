//! Integration test that locks the canonical NBT types emitted when loading
//! the real `V26_1/data/minecraft/timeline/day.json` data report.
//!
//! This test guards against regressions of the `numeric_widening` flag: every
//! field below corresponds to a Mojang `Codec` declaration whose canonical NBT
//! encoding via `NbtOps.INSTANCE` is `IntTag` / `FloatTag` / `ByteTag`,
//! verified directly against the MC 26.1.2 server jar bytecode.

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::manual_assert,
    clippy::indexing_slicing
)]

use pico_nbt::{IndexMap, Value};
use pico_registries::{Identifier, Registry, RegistryKeys};
use std::path::PathBuf;

fn data_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("data")
        .join("generated")
        .join("V26_1")
        .join("data")
}

fn load_timeline_day() -> Value {
    let registry = Registry::load(&RegistryKeys::Timeline, &data_root())
        .expect("timeline registry should load from V26_1 data");
    let day_id = Identifier::vanilla_unchecked("day");
    let entry = registry
        .get(&day_id)
        .expect("day entry should be present in timeline registry");
    entry.get_raw_value().clone()
}

fn compound<'a>(value: &'a Value, msg: &str) -> &'a IndexMap<String, Value> {
    let Value::Compound(map) = value else {
        panic!("Expected compound at {msg}, got {value:?}");
    };
    map
}

fn list<'a>(value: &'a Value, msg: &str) -> &'a Vec<Value> {
    let Value::List(items) = value else {
        panic!("Expected list at {msg}, got {value:?}");
    };
    items
}

#[test]
fn period_ticks_is_int() {
    let day = load_timeline_day();
    let root = compound(&day, "root");
    assert_eq!(
        root.get("period_ticks"),
        Some(&Value::Int(24000)),
        "period_ticks must be IntTag, not Short/Byte — Mojang's ExtraCodecs.POSITIVE_INT \
         (a Codec<Integer>) calls NbtOps.createInt which always emits IntTag"
    );
}

#[test]
fn time_marker_scalar_is_int() {
    let day = load_timeline_day();
    let root = compound(&day, "root");
    let markers = compound(
        root.get("time_markers")
            .expect("time_markers field must exist"),
        "time_markers",
    );
    assert_eq!(
        markers.get("minecraft:wake_up_from_sleep"),
        Some(&Value::Int(0)),
        "scalar time marker tick value must be IntTag, not ByteTag(0)"
    );
}

#[test]
fn time_marker_compound_ticks_is_int_and_bool_is_byte() {
    let day = load_timeline_day();
    let root = compound(&day, "root");
    let markers = compound(
        root.get("time_markers")
            .expect("time_markers field must exist"),
        "time_markers",
    );
    let day_marker = compound(
        markers
            .get("minecraft:day")
            .expect("minecraft:day marker must exist"),
        "minecraft:day",
    );
    assert_eq!(
        day_marker.get("ticks"),
        Some(&Value::Int(1000)),
        "ticks field must be IntTag, not ShortTag(1000)"
    );
    assert_eq!(
        day_marker.get("show_in_commands"),
        Some(&Value::Byte(1)),
        "boolean show_in_commands must remain ByteTag(1) — NbtOps.createBoolean \
         emits ByteTag, never IntTag"
    );
}

#[test]
fn cubic_bezier_keyframe_is_float() {
    let day = load_timeline_day();
    let root = compound(&day, "root");
    let tracks = compound(
        root.get("tracks").expect("tracks field must exist"),
        "tracks",
    );
    let moon_angle = compound(
        tracks
            .get("minecraft:visual/moon_angle")
            .expect("moon_angle track must exist"),
        "moon_angle",
    );
    let ease = compound(
        moon_angle.get("ease").expect("ease field must exist"),
        "ease",
    );
    let bezier = list(
        ease.get("cubic_bezier")
            .expect("cubic_bezier field must exist"),
        "cubic_bezier",
    );
    assert_eq!(bezier.len(), 4, "cubic_bezier must have 4 control values");
    let Value::Float(f) = bezier[0] else {
        panic!(
            "cubic_bezier[0] must be FloatTag — EasingType$CubicBezierControls is a Java record \
             with four float fields encoded via Codec.FLOAT; got {:?}",
            bezier[0]
        );
    };
    if (f - 0.362_f32).abs() >= 1e-5 {
        panic!("cubic_bezier[0] expected ~0.362f, got {f}");
    }
}
