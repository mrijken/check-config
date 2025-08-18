use regex::Regex;

use super::base::CheckConstructor;
pub(crate) use super::{
    base::{Action, Check, CheckDefinitionError, CheckError},
    GenericCheck,
};

#[derive(Debug)]
pub(crate) struct LinesPresent {
    generic_check: GenericCheck,
    lines: String,
    replacement_regex: Option<Regex>,
}

impl CheckConstructor for LinesPresent {
    type Output = Self;
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
        Ok(Self {
            generic_check,
            lines,
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

        if contents.contains(&self.lines) {
            // TODO: check that the content start at the beginning of line
            Ok(Action::None)
        } else {
            let contains_regex = match self.replacement_regex.as_ref() {
                None => false,
                Some(regex) => dbg!(Regex::is_match(regex, contents.as_str())),
            };
            match contains_regex {
                false => {
                    let mut new_contents = contents.clone();
                    if !new_contents.ends_with('\n') && !new_contents.is_empty() {
                        new_contents += "\n";
                    }
                    new_contents += &self.lines.clone();
                    Ok(Action::SetContents(new_contents))
                }
                true => {
                    let regex = self.replacement_regex.as_ref().expect("contains regex");
                    let new_contents = Regex::replace(regex, &contents, &self.lines);
                    Ok(dbg!(Action::SetContents(new_contents.to_string())))
                }
            }
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
        assert_eq!(lines_present_checker.lines, "\n".to_string());

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

        // use replacement_regex
        let mut check_table = toml_edit::Table::new();
        check_table.insert("__lines__", "export EDITOR=hx".into());
        check_table.insert("__replacement_regex__", "(?m)^export EDITOR=.*$".into());

        let lines_present_check =
            LinesPresent::from_check_table(generic_check, check_table).unwrap();

        // file with replacement regex and lines already present
        let mut file = File::create(lines_present_check.generic_check().file_to_check()).unwrap();
        write!(file, "export SHELL=/bin/bash\nexport EDITOR=hx\n").unwrap();
        assert_eq!(lines_present_check.check().unwrap(), Action::None);

        // file with replacement regex and replacement lines present
        let mut file = File::create(lines_present_check.generic_check().file_to_check()).unwrap();
        write!(file, "export SHELL=/bin/bash\nexport EDITOR=vi").unwrap();
        assert_eq!(
            lines_present_check.check().unwrap(),
            Action::SetContents("export SHELL=/bin/bash\nexport EDITOR=hx\n".to_string())
        );

        // file with replacement regex and lines absent
        let mut file = File::create(lines_present_check.generic_check().file_to_check()).unwrap();
        write!(file, "export SHELL=/bin/bash").unwrap();
        assert_eq!(
            lines_present_check.check().unwrap(),
            Action::SetContents("export SHELL=/bin/bash\nexport EDITOR=hx\n".to_string())
        );
    }
}
