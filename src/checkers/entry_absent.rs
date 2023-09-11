use crate::mapping::generic::Mapping;

use super::{
    base::{Action, Check, CheckError},
    DefaultContent, GenericCheck,
};

#[derive(Debug)]
pub(crate) struct EntryAbsent {
    generic_check: GenericCheck,
    value: toml::Table,
}

impl EntryAbsent {
    pub fn new(generic_check: GenericCheck, value: toml::Table) -> Self {
        Self {
            generic_check,
            value,
        }
    }
}

impl Check for EntryAbsent {
    fn check_type(&self) -> String {
        "entry_absent".to_string()
    }

    fn generic_check(&self) -> &GenericCheck {
        &self.generic_check
    }

    fn get_action(&self) -> Result<Action, CheckError> {
        let contents = self
            .generic_check()
            .get_file_contents(DefaultContent::EmptyString)?;
        let mut doc = self.generic_check().get_mapping()?;

        remove_entries(doc.as_mut(), &self.value);

        let new_contents = doc.to_string()?;

        if contents == new_contents {
            Ok(Action::None)
        } else {
            Ok(Action::SetContents(new_contents))
        }
    }
}

fn remove_entries(doc: &mut dyn Mapping, entries_to_remove: &toml::Table) {
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
mod tests {
    use crate::checkers::test_helpers::read_test_files;
    use crate::file_types::{self, FileType};

    use super::*;

    #[test]
    fn test_test_files() {
        for (test_path, test_input, test_expected_output, checker) in
            read_test_files("entry_absent")
        {
            let mut test_input = test_input;
            remove_entries(test_input.as_mut(), checker.as_table().unwrap());

            assert_eq!(
                *test_expected_output,
                test_input.to_string().unwrap(),
                "test_path {} failed",
                test_path
            );
        }
    }

    #[test]
    fn test_remove_entries_with_tables() {
        let entries_to_remove = r#"
[key.list]
__items__ = [{key = "3"}, {key = "2"}, {key = "4"}]
"#;
        let entries_to_remove = toml::from_str::<toml::Value>(entries_to_remove).unwrap();
        let entries_to_remove = entries_to_remove.as_table().unwrap();

        let toml_contents = r#"[key]
list = [{key = "1"}, {key = "2"}]
"#;
        let toml_new_contents = "[key]\nlist = [{key = \"1\"}]\n";

        let mut toml_doc = file_types::toml::Toml::new()
            .to_mapping(toml_contents)
            .unwrap();
        remove_entries(toml_doc.as_mut(), entries_to_remove);

        assert_eq!(toml_new_contents, toml_doc.to_string().unwrap());
    }
}
