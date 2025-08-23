use crate::checkers::utils::{
    append_str, parse_lines, parse_marker_lines, replace_between_markers,
};

use super::base::CheckConstructor;
pub(crate) use super::{
    base::{Action, Check, CheckDefinitionError, CheckError},
    GenericCheck,
};
use regex::Regex;
use similar::DiffableStr;

#[derive(Debug)]
pub(crate) struct LinesPresent {
    generic_check: GenericCheck,
    lines: String,
    replacement_regex: Option<Regex>,
    marker_lines: Option<(String, String)>,
}

impl CheckConstructor for LinesPresent {
    type Output = Self;
    fn from_check_table(
        generic_check: GenericCheck,
        value: toml_edit::Table,
    ) -> Result<Self::Output, CheckDefinitionError> {
        let lines = parse_lines(&value, None)?;
        let marker_lines = parse_marker_lines(&value)?;
        let replacement_regex = match value.get("__replacement_regex__") {
            None => None,
            Some(regex) => match regex.as_str() {
                None => {
                    return Err(CheckDefinitionError::InvalidDefinition(format!(
                        "__replacement_regex__ ({regex}) is not a string"
                    )))
                }
                Some(s) => match Regex::new(s) {
                    Ok(r) => Some(r),
                    Err(_) => {
                        return Err(CheckDefinitionError::InvalidDefinition(format!(
                            "__replacement_regex__ ({regex}) is not a valid regex"
                        )))
                    }
                },
            },
        };

        if replacement_regex.is_some() && marker_lines.is_some() {
            return Err(CheckDefinitionError::InvalidDefinition(
                "Both __replacement_regex__ and __marker__ are defined; that is not allowed".into(),
            ));
        }

        Ok(Self {
            generic_check,
            lines,
            marker_lines,
            replacement_regex,
        })
    }
}
impl Check for LinesPresent {
    fn check_type(&self) -> String {
        "lines_present".to_string()
    }

    fn generic_check(&self) -> &GenericCheck {
        &self.generic_check
    }

    fn get_action(&self) -> Result<Action, CheckError> {
        let contents = self
            .generic_check()
            .get_file_contents(super::DefaultContent::EmptyString)?;

        match (self.replacement_regex.as_ref(), self.marker_lines.as_ref()) {
            (None, None) => {
                if contents.contains(&self.lines) {
                    Ok(Action::None)
                } else {
                    let new_contents = append_str(&contents, &self.lines);
                    Ok(Action::SetContents(new_contents))
                }
            }
            (Some(regex), None) => {
                if contents.contains(&self.lines) {
                    Ok(Action::None)
                } else if regex.is_match(&contents) {
                    let new_contents = Regex::replace(regex, &contents, self.lines.trim_end());
                    Ok(Action::SetContents(new_contents.to_string()))
                } else {
                    let new_contents = append_str(&contents, &self.lines);
                    Ok(Action::SetContents(new_contents))
                }
            }
            (None, Some((start_marker, end_marker))) => {
                let new_contents =
                    replace_between_markers(&contents, start_marker, end_marker, &self.lines);
                if new_contents == contents {
                    Ok(Action::None)
                } else {
                    Ok(Action::SetContents(new_contents))
                }
            }
            _ => panic!(),
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

        let lines_present_checker =
            LinesPresent::from_check_table(generic_check.clone(), check_table).unwrap();
        assert_eq!(lines_present_checker.lines, "".to_string());

        let mut check_table = toml_edit::Table::new();
        check_table.insert("__lines__", "1".into());

        let lines_present_checker =
            LinesPresent::from_check_table(generic_check.clone(), check_table).unwrap();
        assert_eq!(lines_present_checker.lines, "1\n".to_string());

        let mut check_table = toml_edit::Table::new();
        check_table.insert("__lines__", "2\n".into());

        let lines_present_checker =
            LinesPresent::from_check_table(generic_check.clone(), check_table).unwrap();
        assert_eq!(lines_present_checker.lines, "2\n".to_string());
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

        let lines_present_check =
            LinesPresent::from_check_table(generic_check.clone(), check_table).unwrap();

        // not existing file
        assert_eq!(
            lines_present_check.check().unwrap(),
            Action::SetContents("1\n2\n".to_string())
        );

        // empty file
        File::create(lines_present_check.generic_check().file_to_check()).unwrap();
        assert_eq!(
            lines_present_check.check().unwrap(),
            Action::SetContents("1\n2\n".to_string())
        );

        // file with other contents
        let mut file = File::create(lines_present_check.generic_check().file_to_check()).unwrap();
        writeln!(file, "a").unwrap();
        assert_eq!(
            lines_present_check.check().unwrap(),
            Action::SetContents("a\n1\n2\n".to_string())
        );

        // file with correct contents
        let mut file = File::create(lines_present_check.generic_check().file_to_check()).unwrap();
        write!(file, "1\n2\nb\n").unwrap();
        assert_eq!(lines_present_check.check().unwrap(), Action::None);
    }

    #[test]
    fn test_lines_present_with_regex() {
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
        check_table.insert("__lines__", "export EDITOR=hx".into());
        check_table.insert("__replacement_regex__", "(?m)^export EDITOR=.*$".into());

        let lines_present_check =
            LinesPresent::from_check_table(generic_check, check_table).unwrap();

        // file with lines present
        let mut file = File::create(lines_present_check.generic_check().file_to_check()).unwrap();
        write!(file, "export SHELL=/bin/bash\nexport EDITOR=hx\n").unwrap();
        assert_eq!(lines_present_check.check().unwrap(), Action::None);

        // file with regex present
        let mut file = File::create(lines_present_check.generic_check().file_to_check()).unwrap();
        write!(file, "export SHELL=/bin/bash\nexport EDITOR=vi").unwrap();
        assert_eq!(
            lines_present_check.check().unwrap(),
            Action::SetContents("export SHELL=/bin/bash\nexport EDITOR=hx\n".to_string())
        );

        // file with lines absent
        let mut file = File::create(lines_present_check.generic_check().file_to_check()).unwrap();
        write!(file, "export SHELL=/bin/bash").unwrap();
        assert_eq!(
            lines_present_check.check().unwrap(),
            Action::SetContents("export SHELL=/bin/bash\nexport EDITOR=hx\n".to_string())
        );
    }

    #[test]
    fn test_lines_present_with_regex_and_markers() {
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
        check_table.insert("__lines__", "export EDITOR=hx".into());
        check_table.insert("__replacement_regex__", "(?m)^export EDITOR=.*$".into());
        check_table.insert("__marker__", "# marker".into());

        assert!(LinesPresent::from_check_table(generic_check, check_table).is_err());
    }

    #[test]
    fn test_lines_present_with_markers() {
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
        check_table.insert("__lines__", "export EDITOR=hx".into());
        check_table.insert("__marker__", "# marker".into());

        let lines_present_check =
            LinesPresent::from_check_table(generic_check, check_table).unwrap();

        // file with lines already present
        let mut file = File::create(lines_present_check.generic_check().file_to_check()).unwrap();
        write!(file, "export SHELL=/bin/bash\n# marker (check-config start)\nexport EDITOR=hx\n# marker (check-config end)\n").unwrap();
        assert_eq!(lines_present_check.check().unwrap(), Action::None);

        // file with marker present
        let mut file = File::create(lines_present_check.generic_check().file_to_check()).unwrap();
        write!(file, "export SHELL=/bin/bash\n# marker (check-config start)\nexport EDITOR=vi\n# marker (check-config end)\n").unwrap();
        assert_eq!(
            lines_present_check.check().unwrap(),
            Action::SetContents("export SHELL=/bin/bash\n# marker (check-config start)\nexport EDITOR=hx\n# marker (check-config end)\n".to_string()));

        // file with lines absent
        let mut file = File::create(lines_present_check.generic_check().file_to_check()).unwrap();
        write!(file, "export SHELL=/bin/bash").unwrap();
        assert_eq!(
            lines_present_check.check().unwrap(),
            Action::SetContents("export SHELL=/bin/bash\n# marker (check-config start)\nexport EDITOR=hx\n# marker (check-config end)\n".to_string()));
    }
}
