mod clone {
    use pico_identifier::prelude::*;

    #[test]
    fn clone_works() {
        let id1 = Identifier::new_unchecked("minecraft", "stone");
        let id2 = id1.clone();
        assert_eq!(id1, id2);
        assert_eq!(id1.namespace, id2.namespace);
        assert_eq!(id1.thing, id2.thing);
    }
}

mod eq {
    use pico_identifier::prelude::*;

    #[test]
    fn equality_same() {
        let id1 = Identifier::new_unchecked("minecraft", "stone");
        let id2 = Identifier::new_unchecked("minecraft", "stone");
        assert_eq!(id1, id2);
    }

    #[test]
    fn equality_different_namespace() {
        let id1 = Identifier::new_unchecked("minecraft", "stone");
        let id2 = Identifier::new_unchecked("mymod", "stone");
        assert_ne!(id1, id2);
    }

    #[test]
    fn equality_different_thing() {
        let id1 = Identifier::new_unchecked("minecraft", "stone");
        let id2 = Identifier::new_unchecked("minecraft", "dirt");
        assert_ne!(id1, id2);
    }
}

mod hash {
    use pico_identifier::prelude::*;
    use std::collections::HashMap;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    #[test]
    fn hash_works() {
        let mut map = HashMap::new();
        let id = Identifier::new_unchecked("minecraft", "stone");
        map.insert(id.clone(), "value");

        assert_eq!(map.get(&id), Some(&"value"));
    }

    #[test]
    fn hash_same_identifiers() {
        let id1 = Identifier::new_unchecked("minecraft", "stone");
        let id2 = Identifier::new_unchecked("minecraft", "stone");

        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();

        id1.hash(&mut hasher1);
        id2.hash(&mut hasher2);

        assert_eq!(hasher1.finish(), hasher2.finish());
    }

    #[test]
    fn hash_different_identifiers() {
        let id1 = Identifier::new_unchecked("minecraft", "stone");
        let id2 = Identifier::new_unchecked("minecraft", "dirt");

        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();

        id1.hash(&mut hasher1);
        id2.hash(&mut hasher2);

        assert_ne!(hasher1.finish(), hasher2.finish());
    }

    #[test]
    fn hash_map_multiple_keys() {
        let mut map = HashMap::new();

        let id1 = Identifier::new_unchecked("minecraft", "stone");
        let id2 = Identifier::new_unchecked("minecraft", "dirt");
        let id3 = Identifier::new_unchecked("mymod", "stone");

        map.insert(id1.clone(), "value1");
        map.insert(id2.clone(), "value2");
        map.insert(id3.clone(), "value3");

        assert_eq!(map.get(&id1), Some(&"value1"));
        assert_eq!(map.get(&id2), Some(&"value2"));
        assert_eq!(map.get(&id3), Some(&"value3"));
        assert_eq!(map.len(), 3);
    }
}
