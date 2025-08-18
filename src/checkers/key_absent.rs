use crate::mapping::generic::Mapping;

use super::{
    base::{Action, Check, CheckConstructor, CheckError},
    DefaultContent, GenericCheck,
};

#[derive(Debug)]
pub(crate) struct KeyAbsent {
    generic_check: GenericCheck,
    value: toml_edit::Table,
}

impl CheckConstructor for KeyAbsent {
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
impl Check for KeyAbsent {
    fn check_type(&self) -> String {
        "key_absent".to_string()
    }

    fn generic_check(&self) -> &GenericCheck {
        &self.generic_check
    }

    fn get_action(&self) -> Result<Action, CheckError> {
        // if !self.generic_check().file_to_check().exists() {
        //     return Ok(Action::None);
        // }

        let contents = self
            .generic_check()
            .get_file_contents(DefaultContent::EmptyString)?;
        let mut doc = self.generic_check().get_mapping()?;

        unset_key(doc.as_mut(), &self.value);

        let new_contents = doc.to_string()?;

        if contents == new_contents {
            Ok(Action::None)
        } else {
            Ok(Action::SetContents(new_contents))
        }
    }
}

fn unset_key(doc: &mut dyn Mapping, table_to_unset: &toml_edit::Table) {
    for (key_to_unset, value_to_unset) in table_to_unset {
        if let Some(child_table_to_unset) = value_to_unset.as_table() {
            if child_table_to_unset.is_empty() {
                doc.remove(key_to_unset);
            } else if let Ok(child_doc) = doc.get_mapping(key_to_unset, false) {
                unset_key(child_doc, child_table_to_unset);
            } else {
                log::info!(
                    "Key {key_to_unset} is not found in toml, so we can not remove that key",
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::checkers::test_helpers::read_test_files;

    use super::*;

    #[test]
    fn test_test_files() {
        for (test_path, test_input, test_expected_output, checker) in read_test_files("key_absent")
        {
            let mut test_input = test_input;
            unset_key(test_input.as_mut(), &checker);

            assert_eq!(
                *test_expected_output,
                test_input.to_string().unwrap(),
                "test_path {test_path} failed"
            );
        }
    }
}
