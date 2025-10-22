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
                    // TODO: check if there is an array in present
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

        self.file_check.conclude_check_with_new_doc(doc, fix)
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
    use std::{fs::read_to_string, io::Write, str::FromStr};

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
                test_input.to_string(4).unwrap(),
                "test_path {test_path} failed"
            );
        }
    }
    fn get_file_check_with_result(
        entry: i64,
        indent: Option<i64>,
    ) -> (
        Result<EntryPresent, CheckDefinitionError>,
        tempfile::TempDir,
    ) {
        let generic_check = crate::checkers::test_helpers::get_generic_check();
        let tmp_dir = tempfile::tempdir().unwrap();
        let path_to_check = tmp_dir.path().join("file_to_check.json");
        let mut file_to_check = std::fs::File::create(&path_to_check).unwrap();

        let indent = if let Some(indent) = indent {
            format!("indent = {}", indent)
        } else {
            "".to_string()
        };

        let check_doc = toml_edit::DocumentMut::from_str(
            format!(
                r#"
                file="{}"
                entry.list = [ {} ]
                {}"#,
                path_to_check.to_string_lossy(),
                entry,
                indent
            )
            .as_str(),
        )
        .unwrap();
        let check_table = check_doc.as_table().clone();

        writeln!(file_to_check, "{{\"list\": [1,2] }}").unwrap();

        (
            EntryPresent::from_check_table(generic_check, check_table),
            tmp_dir,
        )
    }

    fn get_file_unpacked_check(
        entry: i64,
        indent: Option<i64>,
    ) -> (EntryPresent, tempfile::TempDir) {
        let (file_check_with_result, tempdir) = get_file_check_with_result(entry, indent);

        (
            file_check_with_result.expect("check without issues"),
            tempdir,
        )
    }

    #[test]
    fn test_indent() {
        let (result, _dir) = get_file_check_with_result(12, Some(-2));
        assert_eq!(
            result.err().unwrap(),
            CheckDefinitionError::InvalidDefinition("indent must be >= 0".into())
        );

        let (result, dir) = get_file_check_with_result(12, Some(2));
        result.unwrap().check(true);
        let file_to_check = dir.path().join("file_to_check.json");
        let contents = read_to_string(file_to_check).unwrap();
        assert_eq!(
            contents,
            "{\n  \"list\": [\n    1,\n    2,\n    12\n  ]\n}\n"
        );

        let (result, dir) = get_file_check_with_result(12, Some(4));
        result.unwrap().check(true);
        let file_to_check = dir.path().join("file_to_check.json");
        let contents = read_to_string(file_to_check).unwrap();
        assert_eq!(
            contents,
            "{\n    \"list\": [\n        1,\n        2,\n        12\n    ]\n}\n"
        );
    }
}
