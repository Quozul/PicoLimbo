mod try_from {
    use pico_identifier::prelude::*;

    #[test]
    fn parse_valid() {
        let id = Identifier::try_from("foo:bar/baz").unwrap();
        assert_eq!(id.namespace, "foo");
        assert_eq!(id.thing, "bar/baz");
    }

    #[test]
    fn missing_colon() {
        let err = Identifier::try_from("no‑colon").unwrap_err();
        assert_eq!(err, IdentifierParseError::MissingColon);
    }

    #[test]
    fn bad_namespace_char() {
        let err = Identifier::try_from("f!oo:thing").unwrap_err();
        match err {
            IdentifierParseError::InvalidNamespaceChar { ch, pos, namespace } => {
                assert_eq!(ch, '!');
                assert_eq!(pos, 1);
                assert_eq!(namespace, "f!oo");
            }
            _ => panic!("wrong error variant"),
        }
    }

    #[test]
    fn try_from_valid_all_allowed_chars() {
        // Test all valid characters in namespace and thing
        let id = Identifier::try_from("abc123_-.#:xyz789_-./path").unwrap();
        assert_eq!(id.namespace, "abc123_-.#");
        assert_eq!(id.thing, "xyz789_-./path");
    }

    #[test]
    fn try_from_empty_namespace() {
        let err = Identifier::try_from(":thing").unwrap_err();
        assert_eq!(err, IdentifierParseError::EmptyNamespace);
    }

    #[test]
    fn try_from_empty_thing() {
        let err = Identifier::try_from("namespace:").unwrap_err();
        assert_eq!(err, IdentifierParseError::EmptyThing);
    }

    #[test]
    fn try_from_both_empty() {
        let err = Identifier::try_from(":").unwrap_err();
        assert_eq!(err, IdentifierParseError::EmptyNamespace);
    }

    #[test]
    fn try_from_invalid_thing_char() {
        let err = Identifier::try_from("namespace:th!ng").unwrap_err();
        match err {
            IdentifierParseError::InvalidThingChar { ch, pos, thing } => {
                assert_eq!(ch, '!');
                assert_eq!(pos, 2);
                assert_eq!(thing, "th!ng");
            }
            _ => panic!("wrong error variant"),
        }
    }

    #[test]
    fn try_from_uppercase_namespace_invalid() {
        let err = Identifier::try_from("Minecraft:stone").unwrap_err();
        match err {
            IdentifierParseError::InvalidNamespaceChar { ch, pos, .. } => {
                assert_eq!(ch, 'M');
                assert_eq!(pos, 0);
            }
            _ => panic!("wrong error variant"),
        }
    }

    #[test]
    fn try_from_uppercase_thing_invalid() {
        let err = Identifier::try_from("minecraft:Stone").unwrap_err();
        match err {
            IdentifierParseError::InvalidThingChar { ch, pos, .. } => {
                assert_eq!(ch, 'S');
                assert_eq!(pos, 0);
            }
            _ => panic!("wrong error variant"),
        }
    }

    #[test]
    fn try_from_hash_in_namespace_valid() {
        let id = Identifier::try_from("name#space:thing").unwrap();
        assert_eq!(id.namespace, "name#space");
    }

    #[test]
    fn try_from_hash_in_thing_invalid() {
        let err = Identifier::try_from("namespace:th#ing").unwrap_err();
        match err {
            IdentifierParseError::InvalidThingChar { ch, .. } => {
                assert_eq!(ch, '#');
            }
            _ => panic!("wrong error variant"),
        }
    }

    #[test]
    fn try_from_slash_in_namespace_invalid() {
        let err = Identifier::try_from("name/space:thing").unwrap_err();
        match err {
            IdentifierParseError::InvalidNamespaceChar { ch, .. } => {
                assert_eq!(ch, '/');
            }
            _ => panic!("wrong error variant"),
        }
    }

    #[test]
    fn try_from_slash_in_thing_valid() {
        let id = Identifier::try_from("namespace:thing/path/deep").unwrap();
        assert_eq!(id.thing, "thing/path/deep");
    }

    #[test]
    fn try_from_multiple_colons() {
        // Colon is not allowed in the thing part, so this should fail
        let err = Identifier::try_from("namespace:thing:extra").unwrap_err();
        match err {
            IdentifierParseError::InvalidThingChar { ch, .. } => {
                assert_eq!(ch, ':');
            }
            _ => panic!("wrong error variant"),
        }
    }
}

mod new {
    use pico_identifier::prelude::*;

    #[test]
    fn new_valid() {
        let id = Identifier::new("minecraft", "stone").unwrap();
        assert_eq!(id.namespace, "minecraft");
        assert_eq!(id.thing, "stone");
    }

    #[test]
    fn new_empty_namespace() {
        let err = Identifier::new("", "thing").unwrap_err();
        assert_eq!(err, IdentifierParseError::EmptyNamespace);
    }

    #[test]
    fn new_empty_thing() {
        let err = Identifier::new("namespace", "").unwrap_err();
        assert_eq!(err, IdentifierParseError::EmptyThing);
    }

    #[test]
    fn new_invalid_namespace_char_at_end() {
        let err = Identifier::new("namespace!", "thing").unwrap_err();
        match err {
            IdentifierParseError::InvalidNamespaceChar { ch, pos, .. } => {
                assert_eq!(ch, '!');
                assert_eq!(pos, 9);
            }
            _ => panic!("wrong error variant"),
        }
    }

    #[test]
    fn new_invalid_thing_char_middle() {
        let err = Identifier::new("namespace", "th@ing").unwrap_err();
        match err {
            IdentifierParseError::InvalidThingChar { ch, pos, .. } => {
                assert_eq!(ch, '@');
                assert_eq!(pos, 2);
            }
            _ => panic!("wrong error variant"),
        }
    }

    #[test]
    fn new_with_string_types() {
        let id = Identifier::new(String::from("ns"), String::from("thing")).unwrap();
        assert_eq!(id.namespace, "ns");
        assert_eq!(id.thing, "thing");
    }
}

mod new_unchecked {
    use pico_identifier::prelude::*;

    #[test]
    fn new_unchecked_valid() {
        let id = Identifier::new_unchecked("minecraft", "diamond_sword");
        assert_eq!(id.namespace, "minecraft");
        assert_eq!(id.thing, "diamond_sword");
    }

    #[test]
    fn new_unchecked_allows_invalid_chars() {
        // Should NOT validate - this is the point of unchecked
        let id = Identifier::new_unchecked("Name!Space", "Thing@");
        assert_eq!(id.namespace, "Name!Space");
        assert_eq!(id.thing, "Thing@");
    }

    #[test]
    fn new_unchecked_allows_empty() {
        // Should NOT validate - allows empty strings
        let id = Identifier::new_unchecked("", "");
        assert_eq!(id.namespace, "");
        assert_eq!(id.thing, "");
    }

    #[test]
    fn new_unchecked_with_string_types() {
        let id = Identifier::new_unchecked(String::from("ns"), String::from("thing"));
        assert_eq!(id.namespace, "ns");
        assert_eq!(id.thing, "thing");
    }
}

mod vanilla {
    use pico_identifier::prelude::*;

    #[test]
    fn vanilla_valid() {
        let id = Identifier::vanilla("stone").unwrap();
        assert_eq!(id.namespace, "minecraft");
        assert_eq!(id.thing, "stone");
    }

    #[test]
    fn vanilla_with_path() {
        let id = Identifier::vanilla("block/stone").unwrap();
        assert_eq!(id.namespace, "minecraft");
        assert_eq!(id.thing, "block/stone");
    }

    #[test]
    fn vanilla_empty_thing() {
        let err = Identifier::vanilla("").unwrap_err();
        assert_eq!(err, IdentifierParseError::EmptyThing);
    }

    #[test]
    fn vanilla_invalid_thing_char() {
        let err = Identifier::vanilla("invalid!thing").unwrap_err();
        match err {
            IdentifierParseError::InvalidThingChar { ch, .. } => {
                assert_eq!(ch, '!');
            }
            _ => panic!("wrong error variant"),
        }
    }

    #[test]
    fn vanilla_with_string_type() {
        let id = Identifier::vanilla(String::from("diamond")).unwrap();
        assert_eq!(id.namespace, "minecraft");
        assert_eq!(id.thing, "diamond");
    }
}

mod vanilla_unchecked {
    use pico_identifier::prelude::*;

    #[test]
    fn vanilla_unchecked_valid() {
        let id = Identifier::vanilla_unchecked("gold_ingot");
        assert_eq!(id.namespace, "minecraft");
        assert_eq!(id.thing, "gold_ingot");
    }

    #[test]
    fn vanilla_unchecked_allows_invalid() {
        let id = Identifier::vanilla_unchecked("Invalid!Thing");
        assert_eq!(id.namespace, "minecraft");
        assert_eq!(id.thing, "Invalid!Thing");
    }

    #[test]
    fn vanilla_unchecked_allows_empty() {
        let id = Identifier::vanilla_unchecked("");
        assert_eq!(id.namespace, "minecraft");
        assert_eq!(id.thing, "");
    }
}
