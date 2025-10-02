use crate::{
    checkers::{base::CheckDefinitionError, file::FileCheck},
    mapping::generic::Mapping,
};

use super::super::{
    GenericChecker,
    base::{CheckConstructor, CheckError, Checker},
};

#[derive(Debug)]
pub(crate) struct KeyValuePresent {
    file_check: FileCheck,
    value: toml_edit::Table,
}

// [[key_value_present]]
// file = "file"
// key.key = "value"
impl CheckConstructor for KeyValuePresent {
    type Output = Self;

    fn from_check_table(
        generic_check: GenericChecker,
        check_table: toml_edit::Table,
    ) -> Result<Self::Output, CheckDefinitionError> {
        let file_check = FileCheck::from_check_table(generic_check, &check_table)?;

        let key_value_present = match check_table.get("key") {
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
            value: key_value_present,
        })
    }
}

impl Checker for KeyValuePresent {
    fn checker_type(&self) -> String {
        "key_value_present".to_string()
    }

    fn checker_object(&self) -> String {
        self.file_check.check_object()
    }

    fn generic_checker(&self) -> &GenericChecker {
        &self.file_check.generic_check
    }

    fn check_(&self, fix: bool) -> Result<crate::checkers::base::CheckResult, CheckError> {
        let mut doc = self.file_check.get_mapping()?;

        set_key_value(doc.as_mut(), &self.value);

        self.file_check.conclude_check_with_new_doc(self, doc, fix)
    }
}

fn set_key_value(doc: &mut dyn Mapping, table_to_set: &dyn toml_edit::TableLike) {
    for (k, v) in table_to_set.iter() {
        if v.is_table_like() {
            set_key_value(
                doc.get_mapping(k, true).expect("key exists"),
                v.as_table_like().expect("value is a table"),
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
