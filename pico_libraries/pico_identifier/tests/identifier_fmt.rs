mod display {
    use pico_identifier::prelude::*;

    #[test]
    fn to_string() {
        let id = Identifier::try_from("foo:bar/baz").unwrap();
        assert_eq!(id.to_string(), "foo:bar/baz");
    }

    #[test]
    fn display_simple() {
        let id = Identifier::new_unchecked("minecraft", "stone");
        assert_eq!(format!("{}", id), "minecraft:stone");
    }

    #[test]
    fn display_with_path() {
        let id = Identifier::new_unchecked("mymod", "items/sword/legendary");
        assert_eq!(format!("{}", id), "mymod:items/sword/legendary");
    }

    #[test]
    fn display_with_special_chars() {
        let id = Identifier::new_unchecked("mod_name.v2", "item-type_01");
        assert_eq!(format!("{}", id), "mod_name.v2:item-type_01");
    }
}

mod debug {
    use pico_identifier::prelude::*;
    #[test]
    fn debug_simple() {
        let id = Identifier::new_unchecked("minecraft", "stone");
        assert_eq!(format!("{:?}", id), "Identifier(minecraft:stone)");
    }

    #[test]
    fn debug_with_path() {
        let id = Identifier::new_unchecked("mymod", "items/sword");
        assert_eq!(format!("{:?}", id), "Identifier(mymod:items/sword)");
    }

    #[test]
    fn debug_complex() {
        let id = Identifier::new_unchecked("mod.name-v2", "item_type.01/path");
        assert_eq!(
            format!("{:?}", id),
            "Identifier(mod.name-v2:item_type.01/path)"
        );
    }
}
