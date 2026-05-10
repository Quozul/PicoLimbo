//! Integration test ensuring registry JSON files load with canonical NBT integer
//! tags (Int/Long) and Float (32-bit) floats when numeric widening is enabled —
//! required for MC 1.21.11+ clients and strict third-party codecs
//! (e.g. `PacketEvents`).

use pico_nbt::Value;
use pico_registries::{Identifier, Registry, RegistryKeys};
use std::path::PathBuf;

fn data_root() -> PathBuf {
    // CARGO_MANIFEST_DIR points at pico_libraries/pico_registries/.
    // The on-disk layout is data/generated/<VERSION>/data/minecraft/<registry>/<entry>.json,
    // and Registry::load joins "<namespace>/<thing>" onto its resource_path,
    // so we pass <repo>/data/generated/V26_1/data here.
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data/generated/V26_1/data")
}

#[test]
#[allow(clippy::manual_assert)] // pico_registries' clippy.toml forbids std::assert.
fn timeline_day_emits_canonical_int_and_float_tags() {
    let registry = Registry::load(&RegistryKeys::Timeline, &data_root())
        .expect("V26_1 timeline registry must load");

    let day_id = Identifier::vanilla_unchecked("day");
    let day_entry = registry.try_get(&day_id).expect("timeline/day must exist");

    let Value::Compound(root) = day_entry.get_raw_value() else {
        panic!(
            "expected timeline/day root to be a Compound, got {:?}",
            day_entry.get_raw_value()
        );
    };

    // period_ticks must be Int(24000), not Short(24000).
    let period_ticks = root
        .get("period_ticks")
        .expect("timeline/day must have period_ticks");
    assert_eq!(
        *period_ticks,
        Value::Int(24_000),
        "expected period_ticks: Int(24000), got {period_ticks:?}",
    );

    // time_markers.minecraft:wake_up_from_sleep is the bare scalar `0` — without
    // widening it would be Byte(0).
    let Value::Compound(time_markers) = root
        .get("time_markers")
        .expect("timeline/day must have time_markers")
    else {
        panic!("expected time_markers to be a Compound");
    };
    let wake_up = time_markers
        .get("minecraft:wake_up_from_sleep")
        .expect("time_markers must contain wake_up_from_sleep");
    assert_eq!(
        *wake_up,
        Value::Int(0),
        "expected wake_up_from_sleep: Int(0), got {wake_up:?}",
    );

    // time_markers.minecraft:day.ticks must be Int(1000), not Short(1000).
    let Value::Compound(day_marker) = time_markers
        .get("minecraft:day")
        .expect("time_markers must contain minecraft:day")
    else {
        panic!("expected time_markers.minecraft:day to be a Compound");
    };
    let day_ticks = day_marker
        .get("ticks")
        .expect("time_markers.minecraft:day must have ticks");
    assert_eq!(
        *day_ticks,
        Value::Int(1_000),
        "expected day.ticks: Int(1000), got {day_ticks:?}",
    );

    // show_in_commands is a JSON boolean — must remain Byte(1).
    let show = day_marker
        .get("show_in_commands")
        .expect("day must have show_in_commands");
    assert_eq!(
        *show,
        Value::Byte(1),
        "expected show_in_commands: Byte(1) (boolean), got {show:?}",
    );

    // tracks.minecraft:visual/moon_angle.ease.cubic_bezier[0] is ~0.362 — must
    // collapse to Float (lossy) under widening. Without widening the legacy
    // code keeps it as Double to preserve precision.
    let Value::Compound(tracks) = root.get("tracks").expect("timeline/day must have tracks")
    else {
        panic!("expected tracks to be a Compound");
    };
    let Value::Compound(moon_angle) = tracks
        .get("minecraft:visual/moon_angle")
        .expect("tracks must contain minecraft:visual/moon_angle")
    else {
        panic!("expected minecraft:visual/moon_angle to be a Compound");
    };
    let Value::Compound(ease) = moon_angle.get("ease").expect("moon_angle.ease must exist")
    else {
        panic!("expected moon_angle.ease to be a Compound");
    };
    let Value::List(cubic_bezier) = ease
        .get("cubic_bezier")
        .expect("ease.cubic_bezier must exist")
    else {
        panic!("expected ease.cubic_bezier to be a List");
    };
    let first = cubic_bezier
        .first()
        .expect("cubic_bezier list must be non-empty");
    let Value::Float(f) = first else {
        panic!("expected cubic_bezier[0] to be Float, got {first:?}");
    };
    if (*f - 0.362_f32).abs() >= 0.001 {
        panic!("expected cubic_bezier[0] ≈ 0.362 (Float), got Float({f})");
    }
}
