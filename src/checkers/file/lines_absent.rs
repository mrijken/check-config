use crate::checkers::{
    file::FileCheck,
    utils::{get_lines_from_check_table, get_marker_from_check_table, remove_between_markers},
};

use super::super::base::CheckConstructor;
pub(super) use super::super::{
    GenericChecker,
    base::{CheckDefinitionError, CheckError, Checker},
};

#[derive(Debug)]
pub(crate) struct LinesAbsent {
    file_check: FileCheck,
    lines: String,
    marker_lines: Option<(String, String)>,
}

// [[lines_absent]]
// file = "file"
// lines = "lines"    # lines or marker must be given
// marker = "marker"
impl CheckConstructor for LinesAbsent {
    type Output = LinesAbsent;
    fn from_check_table(
        generic_check: GenericChecker,
        check_table: toml_edit::Table,
    ) -> Result<Self::Output, CheckDefinitionError> {
        let file_check = FileCheck::from_check_table(generic_check, &check_table)?;
        let marker_lines = get_marker_from_check_table(&check_table)?;
        let lines = get_lines_from_check_table(
            &check_table,
            if marker_lines.is_none() {
                None
            } else {
                Some("".to_string())
            },
        )?;
        Ok(Self {
            file_check,
            lines,
            marker_lines,
        })
    }
}

impl Checker for LinesAbsent {
    fn checker_type(&self) -> String {
        "lines_absent".to_string()
    }

    fn checker_object(&self) -> String {
        self.file_check.check_object()
    }

    fn generic_checker(&self) -> &GenericChecker {
        &self.file_check.generic_check
    }

    fn check_(&self, fix: bool) -> Result<crate::checkers::base::CheckResult, CheckError> {
        let contents = self.file_check.get_file_contents()?;

        let new_contents = if let Some((start_marker, end_marker)) = self.marker_lines.as_ref() {
            remove_between_markers(&contents, start_marker, end_marker)
        } else {
            contents.replace(&self.lines, "")
        };

        self.file_check
            .conclude_check_new_contents(self, new_contents, fix)
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Write;

    use crate::checkers::base::CheckResult;
    use crate::checkers::test_helpers;

    use super::*;

    use tempfile::tempdir;

    fn get_lines_absent_check(
        lines: String,
        marker: Option<String>,
    ) -> (LinesAbsent, tempfile::TempDir) {
        let generic_check = test_helpers::get_generic_check();

        let mut check_table = toml_edit::Table::new();
        let dir = tempdir().unwrap();
        let file_to_check = dir.path().join("file_to_check");
        check_table.insert("file", file_to_check.to_string_lossy().to_string().into());
        check_table.insert("lines", lines.into());

        if let Some(marker) = marker {
            check_table.insert("marker", marker.into());
        }

        (
            LinesAbsent::from_check_table(generic_check, check_table).unwrap(),
            dir,
        )
    }

    // #[test]
    // fn test_add_line_ending_when_needed() {
    //     let lines_absent_check = get_lines_absent_check(lines, marker);

    //     let mut check_table = toml_edit::Table::new();
    //     check_table.insert("__lines__", "".into());

    //     let lines_absent_checker =
    //         LinesAbsent::from_check_table(generic_check.clone(), check_table).unwrap();
    //     assert_eq!(lines_absent_checker.lines, "".to_string());

    //     let mut check_table = toml_edit::Table::new();
    //     check_table.insert("__lines__", "1".into());

    //     let lines_absent_checker =
    //         LinesAbsent::from_check_table(generic_check.clone(), check_table).unwrap();
    //     assert_eq!(lines_absent_checker.lines, "1\n".to_string());

    //     let mut check_table = toml_edit::Table::new();
    //     check_table.insert("__lines__", "2\n".into());

    //     let lines_absent_checker =
    //         LinesAbsent::from_check_table(generic_check.clone(), check_table).unwrap();
    //     assert_eq!(lines_absent_checker.lines, "2\n".to_string());
    // }

    #[test]
    fn test_lines_absent() {
        let (lines_absent_check, _tmpdir) = get_lines_absent_check("1\n2\n".into(), None);

        // not existing file
        assert_eq!(
            lines_absent_check.check_(false).unwrap(),
            CheckResult::NoFixNeeded
        );

        // empty file
        File::create(lines_absent_check.file_check.file_to_check.as_ref().clone()).unwrap();
        assert_eq!(
            lines_absent_check.check_(false).unwrap(),
            CheckResult::NoFixNeeded
        );

        // file with other contents
        let mut file: File =
            File::create(lines_absent_check.file_check.file_to_check.as_ref().clone()).unwrap();
        writeln!(file, "a").unwrap();
        assert_eq!(
            lines_absent_check.check_(false).unwrap(),
            CheckResult::NoFixNeeded
        );

        // file with incorrect contents
        let mut file: File =
            File::create(lines_absent_check.file_check.file_to_check.as_ref().clone()).unwrap();
        write!(file, "1\n2\nb\n").unwrap();
        assert_eq!(
            lines_absent_check.check_(false).unwrap(),
            CheckResult::FixNeeded(
                "Set file contents to: \n@@ -1,3 +1 @@\n-1\n-2\n b\n".to_string()
            )
        );

        assert_eq!(
            lines_absent_check.check_(true).unwrap(),
            CheckResult::FixExecuted(
                "Set file contents to: \n@@ -1,3 +1 @@\n-1\n-2\n b\n".to_string()
            )
        );

        assert_eq!(
            lines_absent_check.check_(false).unwrap(),
            CheckResult::NoFixNeeded,
        );
    }

    // todo: add test with marker
}
