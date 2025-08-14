pub(crate) use crate::mapping::generic::Mapping;

use super::{
    base::{Action, Check, CheckConstructor, CheckDefinitionError, CheckError},
    DefaultContent, GenericCheck,
};

#[derive(Debug)]
pub(crate) struct EntryPresent {
    generic_check: GenericCheck,
    value: toml_edit::Table,
}

impl CheckConstructor for EntryPresent {
    type Output = Self;
    fn from_check_table(
        generic_check: GenericCheck,
        value: toml_edit::Table,
    ) -> Result<Self::Output, CheckDefinitionError> {
        Ok(Self {
            generic_check,
            value,
        })
    }
}
impl Check for EntryPresent {
    fn check_type(&self) -> String {
        "entry_present".to_string()
    }

    fn generic_check(&self) -> &GenericCheck {
        &self.generic_check
    }

    fn get_action(&self) -> Result<Action, CheckError> {
        let contents = self
            .generic_check()
            .get_file_contents(DefaultContent::EmptyString)?;
        let mut doc = self.generic_check().get_mapping()?;

        add_entries(doc.as_mut(), &self.value);

        let new_contents = doc.to_string()?;

        if contents == new_contents {
            Ok(Action::None)
        } else {
            Ok(Action::SetContents(new_contents))
        }
    }
}

fn add_entries(doc: &mut dyn Mapping, entries_to_add: &toml_edit::Table) {
    for (key_to_add, value_to_add) in entries_to_add {
        if !value_to_add.is_table_like() {
            log::error!("Unexpected value type");
            std::process::exit(1);
        }
        let table_to_add = value_to_add.as_table().expect("value is a table");
        if table_to_add.contains_key("__items__") {
            let doc_array = doc
                .get_array(key_to_add, true)
                .expect("expecting key to exist");

            for item in table_to_add
                .get("__items__")
                .expect("__items__ is present")
                .as_array()
                .expect("__items__ is an array")
            {
                doc_array.insert_when_not_present(&toml_edit::Item::Value(item.to_owned()))
            }
            continue;
        }

        let child_doc = doc
            .get_mapping(key_to_add, true)
            .expect("expecting key to exist");
        add_entries(child_doc, table_to_add);
    }
}

#[cfg(test)]
mod tests {
    use crate::checkers::test_helpers::read_test_files;

    use super::*;

    #[test]
    fn test_test_files() {
        for (test_path, test_input, test_expected_output, checker) in
            read_test_files("entry_present")
        {
            let mut test_input = test_input;
            add_entries(test_input.as_mut(), &checker);

            assert_eq!(
                *test_expected_output,
                test_input.to_string().unwrap(),
                "test_path {test_path} failed"
            );
        }
    }
}
