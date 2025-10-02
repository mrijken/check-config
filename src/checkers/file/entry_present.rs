use crate::checkers::file::FileCheck;
pub(crate) use crate::mapping::generic::Mapping;

use super::super::{
    GenericChecker,
    base::{CheckConstructor, CheckDefinitionError, CheckError, Checker},
};

#[derive(Debug)]
pub(crate) struct EntryPresent {
    file_check: FileCheck,
    present: toml_edit::Table,
}

// [[entry_present]]
// file = "file"
// entry.key = ["item1"]
impl CheckConstructor for EntryPresent {
    type Output = Self;
    fn from_check_table(
        generic_check: GenericChecker,
        check_table: toml_edit::Table,
    ) -> Result<Self::Output, CheckDefinitionError> {
        let file_check = FileCheck::from_check_table(generic_check, &check_table)?;
        let present_entries = match check_table.get("entry") {
            None => {
                return Err(CheckDefinitionError::InvalidDefinition(
                    "`entry` key is not present".into(),
                ));
            }
            Some(present) => match present.as_table() {
                None => {
                    return Err(CheckDefinitionError::InvalidDefinition(
                        "`entry` is not a table".into(),
                    ));
                }
                Some(present) => {
                    // todo: check if there is an array in present
                    present.clone()
                }
            },
        };

        Ok(Self {
            file_check,
            present: present_entries,
        })
    }
}

// [[entry_present]]
// file = "test.json"
// present.key = ["item"]
impl Checker for EntryPresent {
    fn checker_type(&self) -> String {
        "entry_present".to_string()
    }

    fn checker_object(&self) -> String {
        self.file_check.check_object()
    }

    fn generic_checker(&self) -> &GenericChecker {
        &self.file_check.generic_check
    }

    fn check_(&self, fix: bool) -> Result<crate::checkers::base::CheckResult, CheckError> {
        let mut doc = self.file_check.get_mapping()?;

        add_entries(doc.as_mut(), &self.present);

        self.file_check.conclude_check_with_new_doc(self, doc, fix)
    }
}

fn add_entries(doc: &mut dyn Mapping, entries_to_add: &toml_edit::Table) {
    for (key_to_add, value_to_add) in entries_to_add {
        if let Some(array_to_add) = value_to_add.as_array() {
            let doc_array = doc
                .get_array(key_to_add, true)
                .expect("expecting key to exist");

            for item in array_to_add {
                doc_array.insert_when_not_present(&toml_edit::Item::Value(item.to_owned()))
            }
            continue;
        }

        let child_doc = doc
            .get_mapping(key_to_add, true)
            .expect("expecting key to exist as mapping");
        if let Some(table_to_add) = value_to_add.as_table() {
            add_entries(child_doc, table_to_add);
        }
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
