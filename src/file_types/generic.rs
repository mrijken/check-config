use crate::mapping::generic::Mapping;

pub(crate) fn add_entries(
    doc: &mut dyn Mapping,
    entries_to_add: &toml::map::Map<String, toml::Value>,
) {
    for (key_to_add, value_to_add) in entries_to_add {
        if !value_to_add.is_table() {
            panic!("Unexpected value type");
        }
        let table_to_add = value_to_add.as_table().unwrap();
        if table_to_add.contains_key("__items__") {
            let doc_array = doc
                .get_array(key_to_add, true)
                .expect("expecting key to exist");

            for item in table_to_add.get("__items__").unwrap().as_array().unwrap() {
                doc_array.insert_when_not_present(item)
            }
            continue;
        }

        let child_doc = doc.get_mapping(key_to_add, true).unwrap();
        add_entries(child_doc, table_to_add);
    }
}

pub(crate) fn remove_entries(doc: &mut dyn Mapping, entries_to_remove: &toml::Table) {
    for (key_to_remove, value_to_remove) in entries_to_remove {
        if !value_to_remove.is_table() {
            panic!("Unexpected value type");
        }
        let value_to_remove = value_to_remove.as_table().unwrap();
        if value_to_remove.contains_key("__items__") {
            let doc_array = match doc.get_array(key_to_remove, false) {
                Ok(a) => a,
                Err(_) => panic!("expecting key to exist"),
            };

            for item in value_to_remove
                .get("__items__")
                .unwrap()
                .as_array()
                .unwrap()
            {
                doc_array.remove(item)
            }
            continue;
        }
        let child_doc = doc.get_mapping(key_to_remove, false).unwrap();
        remove_entries(child_doc, value_to_remove);
    }
}

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
            Err(MappingError::MissingKey(key3))
        );

        // Add new sub map

        assert!(!map.contains_key(key3));
        matches!(
            map.get_array(key3, false).err().unwrap(),
            MappingError::MissingKey(key3)
        );
        assert!(!map.contains_key(key3));
        assert!(map.get_array(key3, true).is_ok());

        assert!(map.contains_key(key3));
    }
}
