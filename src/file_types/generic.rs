

use crate::{mapping::generic::Mapping};



#[cfg(test)]
mod test {

    use super::*;
    use crate::mapping::generic::MappingError;

    #[test]
    fn test_mapping() {
        let contents = r#"{
            "1": {
                "1.1": 1
            },
            "2": {
                "2.1": "2"
            }

        }
        "#;
        let mut doc: serde_json::Value = serde_json::from_str(contents).unwrap();
        let map = doc.as_object_mut().unwrap();

        let key3 = "3";

        // check existing sub map
        assert!(map.contains_key("1"));
        assert!(map
            .get_mapping("1", false)
            .ok()
            .unwrap()
            .contains_key("1.1"),);
        assert!(map
            .get_mapping("2", false)
            .ok()
            .unwrap()
            .contains_key("2.1"),);
        matches!(
            map.get_mapping("2", false)
                .ok()
                .unwrap()
                .get_string("2.1-absent"),
            Err(MappingError::MissingKey(_key3))
        );

        // Add new sub map

        assert!(!map.contains_key(key3));
        matches!(
            map.get_array(key3, false).err().unwrap(),
            MappingError::MissingKey(_key3)
        );
        assert!(!map.contains_key(key3));
        assert!(map.get_array(key3, true).is_ok());

        assert!(map.contains_key(key3));
    }
}
