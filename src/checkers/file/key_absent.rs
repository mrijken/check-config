use crate::{
    checkers::{base::CheckDefinitionError, file::FileCheck},
    mapping::generic::Mapping,
};

use super::super::{
    GenericChecker,
    base::{CheckConstructor, CheckError, Checker},
};

#[derive(Debug)]
pub(crate) struct KeyAbsent {
    file_check: FileCheck,
    value: toml_edit::Table,
}

// [[key_absent]]
// file = "file"
// key.key_to_remove = {}
impl CheckConstructor for KeyAbsent {
    type Output = Self;

    fn from_check_table(
        generic_check: GenericChecker,
        check_table: toml_edit::Table,
    ) -> Result<Self::Output, CheckDefinitionError> {
        let file_check = FileCheck::from_check_table(generic_check, &check_table)?;

        let key_absent = match check_table.get("key") {
            None => {
                return Err(CheckDefinitionError::InvalidDefinition(
                    "`key` key is not present".into(),
                ));
            }
            Some(absent) => match absent.as_table() {
                None => {
                    return Err(CheckDefinitionError::InvalidDefinition(
                        "`key` is not a table".into(),
                    ));
                }
                Some(absent) => {
                    // todo: check if there is an array in absent
                    absent.clone()
                }
            },
        };

        Ok(Self {
            file_check,
            value: key_absent,
        })
    }
}

impl Checker for KeyAbsent {
    fn checker_type(&self) -> String {
        "key_absent".to_string()
    }

    fn generic_checker(&self) -> &GenericChecker {
        &self.file_check.generic_check
    }

    fn checker_object(&self) -> String {
        self.file_check.check_object()
    }

    fn check_(&self, fix: bool) -> Result<crate::checkers::base::CheckResult, CheckError> {
        let mut doc = self.file_check.get_mapping()?;

        unset_key(doc.as_mut(), &self.value);

        self.file_check.conclude_check_with_new_doc(self, doc, fix)
    }
}

fn unset_key(doc: &mut dyn Mapping, table_to_unset: &dyn toml_edit::TableLike) {
    for (key_to_unset, value_to_unset) in table_to_unset.iter() {
        if let Some(child_table_to_unset) = value_to_unset.as_table_like() {
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
