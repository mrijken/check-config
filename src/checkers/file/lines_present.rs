use crate::checkers::{
    file::FileCheck,
    utils::{
        append_str, get_lines_from_check_table, get_marker_from_check_table,
        replace_between_markers,
    },
};

use super::super::base::CheckConstructor;
pub(crate) use super::super::{
    GenericChecker,
    base::{CheckDefinitionError, CheckError, Checker},
};
use regex::Regex;

#[derive(Debug)]
pub(crate) struct LinesPresent {
    file_check: FileCheck,
    lines: String,
    replacement_regex: Option<Regex>,
    marker_lines: Option<(String, String)>,
}

pub(crate) fn get_replacement_regex_from_check_table(
    check_table: &toml_edit::Table,
) -> Result<Option<Regex>, CheckDefinitionError> {
    match check_table.get("replacement_regex") {
        None => Ok(None),
        Some(regex) => match regex.as_str() {
            None => Err(CheckDefinitionError::InvalidDefinition(format!(
                "replacement_regex ({regex}) is not a string"
            ))),
            Some(s) => match Regex::new(s) {
                Ok(r) => Ok(Some(r)),
                Err(_) => Err(CheckDefinitionError::InvalidDefinition(format!(
                    "__replacement_regex__ ({regex}) is not a valid regex"
                ))),
            },
        },
    }
}

// [[lines_present]]
// file = "file"
// lines = "lines"
// marker = "marker"       # marker or replacement_regex may be present. Both may be absent. Both may not be present
// replacement_regex = "regex"
impl CheckConstructor for LinesPresent {
    type Output = Self;
    fn from_check_table(
        generic_check: GenericChecker,
        value: toml_edit::Table,
    ) -> Result<Self::Output, CheckDefinitionError> {
        let file_check = FileCheck::from_check_table(generic_check, &value)?;
        let lines = get_lines_from_check_table(&value, None)?;
        let marker_lines = get_marker_from_check_table(&value)?;
        let replacement_regex = get_replacement_regex_from_check_table(&value)?;

        if replacement_regex.is_some() && marker_lines.is_some() {
            return Err(CheckDefinitionError::InvalidDefinition(
                "Both `replacement_regex` and `marker` are defined; that is not allowed".into(),
            ));
        }

        Ok(Self {
            file_check,
            lines,
            marker_lines,
            replacement_regex,
        })
    }
}

impl Checker for LinesPresent {
    fn checker_type(&self) -> String {
        "lines_present".to_string()
    }

    fn checker_object(&self) -> String {
        self.file_check.check_object()
    }

    fn generic_checker(&self) -> &GenericChecker {
        &self.file_check.generic_check
    }

    fn check_(&self, fix: bool) -> Result<crate::checkers::base::CheckResult, CheckError> {
        let contents = self.file_check.get_file_contents()?;

        let new_contents = match (self.replacement_regex.as_ref(), self.marker_lines.as_ref()) {
            (None, None) => {
                if contents.contains(&self.lines) {
                    contents.clone()
                } else {
                    append_str(&contents, &self.lines)
                }
            }
            (Some(regex), None) => {
                if contents.contains(&self.lines) {
                    contents.clone()
                } else if regex.is_match(&contents) {
                    Regex::replace(regex, &contents, self.lines.trim_end()).to_string()
                } else {
                    append_str(&contents, &self.lines)
                }
            }
            (None, Some((start_marker, end_marker))) => {
                replace_between_markers(&contents, start_marker, end_marker, &self.lines)
            }
            _ => panic!(),
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

    fn get_lines_present_check(
        lines: String,
        marker: Option<String>,
        replacement_regex: Option<String>,
    ) -> (LinesPresent, tempfile::TempDir) {
        let generic_check = test_helpers::get_generic_check();

        let mut check_table = toml_edit::Table::new();
        let dir = tempdir().unwrap();
        let file_to_check = dir.path().join("file_to_check");
        check_table.insert("file", file_to_check.to_string_lossy().to_string().into());
        check_table.insert("lines", lines.into());

        if let Some(marker) = marker {
            check_table.insert("marker", marker.into());
        }

        if let Some(replacement_regex) = replacement_regex {
            check_table.insert("replacement_regex", replacement_regex.into());
        }

        (
            LinesPresent::from_check_table(generic_check, check_table).unwrap(),
            dir,
        )
    }

    #[test]
    fn test_lines_present() {
        let (lines_present_check, _tempdir) = get_lines_present_check("1\n2\n".into(), None, None);

        // not existing file
        assert_eq!(
            lines_present_check.check_(false).unwrap(),
            CheckResult::FixNeeded("Set file contents to: \n@@ -0,0 +1,2 @@\n+1\n+2\n".into())
        );

        // empty file
        File::create(lines_present_check.file_check.file_to_check.as_ref()).unwrap();
        assert_eq!(
            lines_present_check.check_(false).unwrap(),
            CheckResult::FixNeeded("Set file contents to: \n@@ -1 +1,2 @@\n-\n+1\n+2\n".into())
        );

        // file with other contents
        let mut file = File::create(lines_present_check.file_check.file_to_check.as_ref()).unwrap();
        writeln!(file, "a").unwrap();
        assert_eq!(
            lines_present_check.check_(false).unwrap(),
            CheckResult::FixNeeded("Set file contents to: \n@@ -1 +1,3 @@\n a\n+1\n+2\n".into())
        );

        // file with correct contents
        assert_eq!(
            lines_present_check.check_(true).unwrap(),
            CheckResult::FixExecuted("Set file contents to: \n@@ -1 +1,3 @@\n a\n+1\n+2\n".into())
        );

        assert_eq!(
            lines_present_check.check_(false).unwrap(),
            CheckResult::NoFixNeeded
        );
    }

    // #[test]
    // fn test_lines_present_with_regex() {
    //     let dir = tempdir().unwrap();
    //     let file_to_check = dir.path().join("file_to_check");
    //     let file_with_checks =
    //         url::Url::from_file_path(dir.path().join("file_with_checks")).unwrap();
    //     let generic_check = GenericCheck {
    //         file_with_checks,
    //         tags: Vec::new(),
    //     };

    //     let mut check_table = toml_edit::Table::new();
    //     check_table.insert("__lines__", "export EDITOR=hx".into());
    //     check_table.insert("__replacement_regex__", "(?m)^export EDITOR=.*$".into());

    //     let lines_present_check =
    //         LinesPresent::from_check_table(generic_check, check_table).unwrap();

    //     // file with lines present
    //     let mut file = File::create(lines_present_check.generic_check().file_to_check()).unwrap();
    //     write!(file, "export SHELL=/bin/bash\nexport EDITOR=hx\n").unwrap();
    //     assert_eq!(lines_present_check.check().unwrap(), Action::None);

    //     // file with regex present
    //     let mut file = File::create(lines_present_check.generic_check().file_to_check()).unwrap();
    //     write!(file, "export SHELL=/bin/bash\nexport EDITOR=vi").unwrap();
    //     assert_eq!(
    //         lines_present_check.check().unwrap(),
    //         Action::SetContents("export SHELL=/bin/bash\nexport EDITOR=hx\n".to_string())
    //     );

    //     // file with lines absent
    //     let mut file = File::create(lines_present_check.generic_check().file_to_check()).unwrap();
    //     write!(file, "export SHELL=/bin/bash").unwrap();
    //     assert_eq!(
    //         lines_present_check.check().unwrap(),
    //         Action::SetContents("export SHELL=/bin/bash\nexport EDITOR=hx\n".to_string())
    //     );
    // }

    // #[test]
    // fn test_lines_present_with_regex_and_markers() {
    //     let dir = tempdir().unwrap();
    //     let file_to_check = dir.path().join("file_to_check");
    //     let file_with_checks =
    //         url::Url::from_file_path(dir.path().join("file_with_checks")).unwrap();
    //     let generic_check = GenericCheck {
    //         file_with_checks,
    //         tags: Vec::new(),
    //     };

    //     let mut check_table = toml_edit::Table::new();
    //     check_table.insert("__lines__", "export EDITOR=hx".into());
    //     check_table.insert("__replacement_regex__", "(?m)^export EDITOR=.*$".into());
    //     check_table.insert("__marker__", "# marker".into());

    //     assert!(LinesPresent::from_check_table(generic_check, check_table).is_err());
    // }

    // #[test]
    // fn test_lines_present_with_markers() {
    //     let dir = tempdir().unwrap();
    //     let file_to_check = dir.path().join("file_to_check");
    //     let file_with_checks =
    //         url::Url::from_file_path(dir.path().join("file_with_checks")).unwrap();
    //     let generic_check = GenericCheck {
    //         file_with_checks,
    //         tags: Vec::new(),
    //     };

    //     let mut check_table = toml_edit::Table::new();
    //     check_table.insert("__lines__", "export EDITOR=hx".into());
    //     check_table.insert("__marker__", "# marker".into());

    //     let lines_present_check =
    //         LinesPresent::from_check_table(generic_check, check_table).unwrap();

    //     // file with lines already present
    //     let mut file = File::create(lines_present_check.generic_check().file_to_check()).unwrap();
    //     write!(file, "export SHELL=/bin/bash\n# marker (check-config start)\nexport EDITOR=hx\n# marker (check-config end)\n").unwrap();
    //     assert_eq!(lines_present_check.check().unwrap(), Action::None);

    //     // file with marker present
    //     let mut file = File::create(lines_present_check.generic_check().file_to_check()).unwrap();
    //     write!(file, "export SHELL=/bin/bash\n# marker (check-config start)\nexport EDITOR=vi\n# marker (check-config end)\n").unwrap();
    //     assert_eq!(
    //         lines_present_check.check().unwrap(),
    //         Action::SetContents("export SHELL=/bin/bash\n# marker (check-config start)\nexport EDITOR=hx\n# marker (check-config end)\n".to_string()));

    //     // file with lines absent
    //     let mut file = File::create(lines_present_check.generic_check().file_to_check()).unwrap();
    //     write!(file, "export SHELL=/bin/bash").unwrap();
    //     assert_eq!(
    //         lines_present_check.check().unwrap(),
    //         Action::SetContents("export SHELL=/bin/bash\n# marker (check-config start)\nexport EDITOR=hx\n# marker (check-config end)\n".to_string()));
    // }
}
