use crate::{checkers::file::FileCheck, mapping::generic::Mapping};

use super::super::{
    GenericChecker,
    base::{CheckConstructor, CheckDefinitionError, CheckError, Checker},
};

#[derive(Debug)]
pub(crate) struct EntryAbsent {
    file_check: FileCheck,
    absent: toml_edit::Table,
}

// [[entry_absent]]
// file = "file"
// entry.key = ["item1"]
impl CheckConstructor for EntryAbsent {
    type Output = Self;
    fn from_check_table(
        generic_check: GenericChecker,
        check_table: toml_edit::Table,
    ) -> Result<Self::Output, CheckDefinitionError> {
        let file_check = FileCheck::from_check_table(generic_check, &check_table)?;
        let absent_entries = match check_table.get("entry") {
            None => {
                return Err(CheckDefinitionError::InvalidDefinition(
                    "`entry` key is not present".into(),
                ));
            }
            Some(absent) => match absent.as_table() {
                None => {
                    return Err(CheckDefinitionError::InvalidDefinition(
                        "`entry` is not a table".into(),
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
            absent: absent_entries,
        })
    }
}

// [[entry_absent]]
// file = "test.json"
// entry.key = ["item"]
impl Checker for EntryAbsent {
    fn checker_type(&self) -> String {
        "entry_absent".to_string()
    }

    fn checker_object(&self) -> String {
        self.file_check.check_object()
    }

    fn generic_checker(&self) -> &GenericChecker {
        &self.file_check.generic_check
    }

    fn check_(&self, fix: bool) -> Result<crate::checkers::base::CheckResult, CheckError> {
        let mut doc = self.file_check.get_mapping()?;

        remove_entries(doc.as_mut(), &self.absent);

        self.file_check.conclude_check_with_new_doc(self, doc, fix)
    }
}

fn remove_entries(doc: &mut dyn Mapping, entries_to_remove: &toml_edit::Table) {
    for (key_to_remove, value_to_remove) in entries_to_remove {
        if !doc.contains_key(key_to_remove) {
            // key_to_remove does not exists, so no need to remove value_to_remove
            continue;
        }
        if let Some(value_to_remove) = value_to_remove.as_array() {
            let doc_array = match doc.get_array(key_to_remove, false) {
                Ok(a) => a,
                Err(_) => {
                    log::error!("expecting key to exist");
                    std::process::exit(1);
                }
            };

            for item in value_to_remove.iter() {
                doc_array.remove(&toml_edit::Item::Value(item.to_owned()))
            }
            continue;
        }
        let child_doc = doc
            .get_mapping(key_to_remove, false)
            .expect("key exists from which value is removed");
        if let Some(value_to_remove) = value_to_remove.as_table() {
            remove_entries(child_doc, value_to_remove);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::checkers::test_helpers::read_test_files;
    use crate::file_types::{self, FileType};

    use super::*;

    #[test]
    fn test_test_files() {
        for (test_path, test_input, test_expected_output, checker) in
            read_test_files("entry_absent")
        {
            let mut test_input = test_input;
            remove_entries(test_input.as_mut(), &checker);

            assert_eq!(
                *test_expected_output,
                test_input.to_string().unwrap(),
                "test_path {test_path} failed"
            );
        }
    }

    #[test]
    fn test_remove_entries_with_tables() {
        let entries_to_remove = r#"
[key]
list = [{key = "3"}, {key = "2"}, {key = "4"}]
"#;
        let entries_to_remove = toml_edit::DocumentMut::from_str(entries_to_remove).unwrap();
        let entries_to_remove = entries_to_remove.as_table();

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
