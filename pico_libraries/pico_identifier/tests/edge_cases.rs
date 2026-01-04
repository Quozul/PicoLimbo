use pico_identifier::prelude::*;

#[test]
fn single_char_namespace_and_thing() {
    let id = Identifier::new("a", "b").unwrap();
    assert_eq!(id.namespace, "a");
    assert_eq!(id.thing, "b");
}

#[test]
fn very_long_identifier() {
    let long_ns = "a".repeat(1000);
    let long_thing = "b".repeat(1000);
    let id = Identifier::new(&long_ns, &long_thing).unwrap();
    assert_eq!(id.namespace.len(), 1000);
    assert_eq!(id.thing.len(), 1000);
}

#[test]
fn all_valid_namespace_special_chars() {
    let id = Identifier::new("test_-.#", "thing").unwrap();
    assert_eq!(id.namespace, "test_-.#");
}

#[test]
fn all_valid_thing_special_chars() {
    let id = Identifier::new("namespace", "test_-./").unwrap();
    assert_eq!(id.thing, "test_-./");
}

#[test]
fn numeric_only() {
    let id = Identifier::new("123456", "789").unwrap();
    assert_eq!(id.namespace, "123456");
    assert_eq!(id.thing, "789");
}

#[test]
fn deeply_nested_path() {
    let id = Identifier::new("mod", "a/b/c/d/e/f/g/h/i/j").unwrap();
    assert_eq!(id.thing, "a/b/c/d/e/f/g/h/i/j");
}

#[test]
fn consecutive_special_chars() {
    let id = Identifier::new("mod__.--", "item__..--//").unwrap();
    assert_eq!(id.namespace, "mod__.--");
    assert_eq!(id.thing, "item__..--//");
}

#[test]
fn starting_with_number() {
    let id = Identifier::new("0mod", "0item").unwrap();
    assert_eq!(id.namespace, "0mod");
    assert_eq!(id.thing, "0item");
}

#[test]
fn ending_with_special_char() {
    let id = Identifier::new("mod_", "item/").unwrap();
    assert_eq!(id.namespace, "mod_");
    assert_eq!(id.thing, "item/");
}

#[test]
fn all_underscores() {
    let id = Identifier::new("___", "____").unwrap();
    assert_eq!(id.namespace, "___");
    assert_eq!(id.thing, "____");
}

#[test]
fn all_numbers() {
    let id = Identifier::new("0123456789", "9876543210").unwrap();
    assert_eq!(id.namespace, "0123456789");
    assert_eq!(id.thing, "9876543210");
}

#[test]
fn mixed_separators_in_path() {
    let id = Identifier::new("my-mod_v2.0", "items/weapons_melee/sword-legendary.tier_5").unwrap();
    assert_eq!(id.namespace, "my-mod_v2.0");
    assert_eq!(id.thing, "items/weapons_melee/sword-legendary.tier_5");
}

#[test]
fn hash_at_various_positions() {
    let id1 = Identifier::new("#namespace", "thing").unwrap();
    assert_eq!(id1.namespace, "#namespace");

    let id2 = Identifier::new("name#space", "thing").unwrap();
    assert_eq!(id2.namespace, "name#space");

    let id3 = Identifier::new("namespace#", "thing").unwrap();
    assert_eq!(id3.namespace, "namespace#");
}

#[test]
fn slash_at_various_positions() {
    let id1 = Identifier::new("namespace", "/thing").unwrap();
    assert_eq!(id1.thing, "/thing");

    let id2 = Identifier::new("namespace", "th/ing").unwrap();
    assert_eq!(id2.thing, "th/ing");

    let id3 = Identifier::new("namespace", "thing/").unwrap();
    assert_eq!(id3.thing, "thing/");
}

#[test]
fn empty_path_segments() {
    // Multiple consecutive slashes create "empty" path segments
    let id = Identifier::new("namespace", "a//b///c").unwrap();
    assert_eq!(id.thing, "a//b///c");
}
