use crate::mapping::generic::Mapping;

use super::{
    base::{Action, Check, CheckError},
    DefaultContent, GenericCheck,
};

#[derive(Debug)]
pub(crate) struct KeyValuePresent {
    generic_check: GenericCheck,
    value: toml::Table,
}

impl KeyValuePresent {
    pub fn new(generic_check: GenericCheck, value: toml::Table) -> Self {
        Self {
            generic_check,
            value,
        }
    }
}

impl Check for KeyValuePresent {
    fn check_type(&self) -> String {
        "key_value_present".to_string()
    }

    fn generic_check(&self) -> &GenericCheck {
        &self.generic_check
    }

    fn get_action(&self) -> Result<Action, CheckError> {
        let contents = self
            .generic_check()
            .get_file_contents(DefaultContent::EmptyString)?;
        let mut doc = self.generic_check().get_mapping()?;

        set_key_value(doc.as_mut(), &self.value);

        let new_contents = doc.to_string()?;

        if contents == new_contents {
            Ok(Action::None)
        } else {
            Ok(Action::SetContents(new_contents))
        }
    }
}

fn set_key_value(doc: &mut dyn Mapping, table_to_set: &toml::Table) {
    for (k, v) in table_to_set {
        if !v.is_table() {
            doc.insert(k, v);
            continue;
        }
        let child_doc = doc.get_mapping(k, true).expect("key exists or is added");
        set_key_value(child_doc, v.as_table().expect("value is a table"));
    }
}

#[cfg(test)]
mod tests {
    use crate::checkers::test_helpers::read_test_files;

    use super::*;

    #[test]
    fn test_test_files() {
        for (test_path, test_input, test_expected_output, checker) in
            read_test_files("key_value_present")
        {
            let mut test_input = test_input;
            set_key_value(test_input.as_mut(), checker.as_table().unwrap());

            assert_eq!(
                *test_expected_output,
                test_input.to_string().unwrap(),
                "test_path {} failed",
                test_path
            );
        }
    }
}
