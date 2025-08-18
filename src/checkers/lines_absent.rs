use super::base::CheckConstructor;
pub(super) use super::{
    base::{Action, Check, CheckDefinitionError, CheckError},
    GenericCheck,
};

#[derive(Debug)]
pub(crate) struct LinesAbsent {
    generic_check: GenericCheck,
    lines: String,
}

impl CheckConstructor for LinesAbsent {
    type Output = LinesAbsent;
    fn from_check_table(
        generic_check: GenericCheck,
        value: toml_edit::Table,
    ) -> Result<Self::Output, CheckDefinitionError> {
        let lines = match value.get("__lines__") {
            None => {
                return Err(CheckDefinitionError::InvalidDefinition(
                    "__lines__ is not present".to_string(),
                ))
            }
            Some(lines) => match lines.as_str() {
                None => {
                    return Err(CheckDefinitionError::InvalidDefinition(
                        "__lines__ is not a string".to_string(),
                    ))
                }
                Some(lines) => lines.to_string(),
            },
        };
        let lines = if !lines.ends_with('\n') {
            lines + "\n"
        } else {
            lines
        };
        Ok(Self {
            generic_check,
            lines,
        })
    }
}
impl Check for LinesAbsent {
    fn check_type(&self) -> String {
        "lines_absent".to_string()
    }

    fn generic_check(&self) -> &GenericCheck {
        &self.generic_check
    }

    fn get_action(&self) -> Result<Action, CheckError> {
        if !self.generic_check.file_to_check().exists() {
            return Ok(Action::None);
        }

        let contents = self
            .generic_check()
            .get_file_contents(super::DefaultContent::None)?;
        if contents.contains(&self.lines) {
            // TODO: check that the content start at the beginning of line
            let new_contents = contents.replace(&self.lines, "");
            Ok(Action::SetContents(new_contents))
        } else {
            Ok(Action::None)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Write;

    use super::*;

    use tempfile::tempdir;

    #[test]
    fn test_add_line_ending_when_needed() {
        let dir = tempdir().unwrap();
        let file_to_check = dir.path().join("file_to_check");
        let file_with_checks =
            url::Url::from_file_path(dir.path().join("file_with_checks")).unwrap();
        let generic_check = GenericCheck {
            file_to_check,
            file_type_override: None,
            file_with_checks,
        };

        let mut check_table = toml_edit::Table::new();
        check_table.insert("__lines__", "".into());

        let lines_absent_checker =
            LinesAbsent::from_check_table(generic_check.clone(), check_table).unwrap();
        assert_eq!(lines_absent_checker.lines, "\n".to_string());

        let mut check_table = toml_edit::Table::new();
        check_table.insert("__lines__", "1".into());

        let lines_absent_checker =
            LinesAbsent::from_check_table(generic_check.clone(), check_table).unwrap();
        assert_eq!(lines_absent_checker.lines, "1\n".to_string());

        let mut check_table = toml_edit::Table::new();
        check_table.insert("__lines__", "2\n".into());

        let lines_absent_checker =
            LinesAbsent::from_check_table(generic_check.clone(), check_table).unwrap();
        assert_eq!(lines_absent_checker.lines, "2\n".to_string());
    }

    #[test]
    fn test_lines_present() {
        let dir = tempdir().unwrap();
        let file_to_check = dir.path().join("file_to_check");
        let file_with_checks =
            url::Url::from_file_path(dir.path().join("file_with_checks")).unwrap();
        let generic_check = GenericCheck {
            file_to_check,
            file_type_override: None,
            file_with_checks,
        };

        let mut check_table = toml_edit::Table::new();
        check_table.insert("__lines__", "1\n2\n".into());

        let lines_absent_check = LinesAbsent::from_check_table(generic_check, check_table).unwrap();

        // not existing file
        assert_eq!(lines_absent_check.check().unwrap(), Action::None);

        // empty file
        File::create(lines_absent_check.generic_check().file_to_check()).unwrap();
        assert_eq!(lines_absent_check.check().unwrap(), Action::None);

        // file with other contents
        let mut file: File =
            File::create(lines_absent_check.generic_check().file_to_check()).unwrap();
        writeln!(file, "a").unwrap();
        assert_eq!(lines_absent_check.check().unwrap(), Action::None);

        // file with incorrect contents
        let mut file: File =
            File::create(lines_absent_check.generic_check().file_to_check()).unwrap();
        write!(file, "1\n2\nb\n").unwrap();
        assert_eq!(
            lines_absent_check.check().unwrap(),
            Action::SetContents("b\n".to_string())
        );
    }
}
