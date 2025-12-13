#![cfg(feature = "serde")]

mod serialization {
    use pico_identifier::prelude::*;
    use serde_test::{Token, assert_tokens};

    #[test]
    fn serde_serialize_simple() {
        let id = Identifier::new_unchecked("minecraft", "stone");
        assert_tokens(&id, &[Token::Str("minecraft:stone")]);
    }

    #[test]
    fn serde_serialize_with_path() {
        let id = Identifier::new_unchecked("mymod", "items/diamond_sword");
        assert_tokens(&id, &[Token::Str("mymod:items/diamond_sword")]);
    }

    #[test]
    fn serde_serialize_special_chars() {
        let id = Identifier::new_unchecked("mod-name_v2.0", "item.type-01/path");
        assert_tokens(&id, &[Token::Str("mod-name_v2.0:item.type-01/path")]);
    }
}

mod deserialization {
    use pico_identifier::prelude::*;

    #[test]
    fn serde_deserialize_valid() {
        let json = r#""minecraft:stone""#;
        let id: Identifier = serde_json::from_str(json).unwrap();
        assert_eq!(id.namespace, "minecraft");
        assert_eq!(id.thing, "stone");
    }

    #[test]
    fn serde_deserialize_with_path() {
        let json = r#""mymod:items/weapons/sword""#;
        let id: Identifier = serde_json::from_str(json).unwrap();
        assert_eq!(id.namespace, "mymod");
        assert_eq!(id.thing, "items/weapons/sword");
    }

    #[test]
    fn serde_deserialize_missing_colon() {
        let json = r#""no-colon-here""#;
        let result: Result<Identifier, _> = serde_json::from_str(json);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("missing ':'"));
    }

    #[test]
    fn serde_deserialize_empty_namespace() {
        let json = r#"":thing""#;
        let result: Result<Identifier, _> = serde_json::from_str(json);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("namespace must not be empty"));
    }

    #[test]
    fn serde_deserialize_empty_thing() {
        let json = r#""namespace:""#;
        let result: Result<Identifier, _> = serde_json::from_str(json);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("thing must not be empty"));
    }

    #[test]
    fn serde_deserialize_invalid_namespace_char() {
        let json = r#""name!space:thing""#;
        let result: Result<Identifier, _> = serde_json::from_str(json);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("illegal character"));
        assert!(err.contains("namespace"));
    }

    #[test]
    fn serde_deserialize_invalid_thing_char() {
        let json = r#""namespace:th!ng""#;
        let result: Result<Identifier, _> = serde_json::from_str(json);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("illegal character"));
        assert!(err.contains("thing"));
    }
}

mod integration {
    use pico_identifier::prelude::*;

    #[test]
    fn serde_roundtrip() {
        let original = Identifier::new_unchecked("mymod", "items/special_sword");
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: Identifier = serde_json::from_str(&json).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn serde_in_struct() {
        #[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)]
        struct Item {
            id: Identifier,
            count: i32,
        }

        let item = Item {
            id: Identifier::new_unchecked("minecraft", "diamond"),
            count: 64,
        };

        let json = serde_json::to_string(&item).unwrap();
        assert!(json.contains("minecraft:diamond"));

        let deserialized: Item = serde_json::from_str(&json).unwrap();
        assert_eq!(item, deserialized);
    }

    #[test]
    fn serde_in_vec() {
        let ids = vec![
            Identifier::new_unchecked("minecraft", "stone"),
            Identifier::new_unchecked("mymod", "custom_item"),
            Identifier::new_unchecked("other", "items/special"),
        ];

        let json = serde_json::to_string(&ids).unwrap();
        let deserialized: Vec<Identifier> = serde_json::from_str(&json).unwrap();
        assert_eq!(ids, deserialized);
    }
}
