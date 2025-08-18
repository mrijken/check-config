use crate::mapping::generic::Mapping;

use super::{
    base::{Action, Check, CheckConstructor, CheckError},
    DefaultContent, GenericCheck,
};

#[derive(Debug)]
pub(crate) struct KeyValuePresent {
    generic_check: GenericCheck,
    value: toml_edit::Table,
}

impl CheckConstructor for KeyValuePresent {
    type Output = Self;

    fn from_check_table(
        generic_check: GenericCheck,
        value: toml_edit::Table,
    ) -> Result<Self::Output, super::base::CheckDefinitionError> {
        Ok(Self {
            generic_check,
            value,
        })
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

fn set_key_value(doc: &mut dyn Mapping, table_to_set: &toml_edit::Table) {
    for (k, v) in table_to_set.iter() {
        if v.is_table() {
            set_key_value(
                doc.get_mapping(k, true).expect("key exists"),
                v.as_table().expect("value is a table"),
            );
            continue;
        }
        doc.insert(
            table_to_set.key(k).expect("key exists"),
            &toml_edit::Item::Value(v.as_value().unwrap().to_owned()),
        );
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
            set_key_value(test_input.as_mut(), &checker);

            assert_eq!(
                *test_expected_output,
                test_input.to_string().unwrap(),
                "test_path {test_path} failed"
            );
        }
    }
}
