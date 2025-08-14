pub(crate) use regex::Regex;

use super::{
    base::{Action, Check, CheckConstructor, CheckDefinitionError, CheckError},
    GenericCheck,
};

#[derive(Debug)]
pub(crate) struct FileRegexMatch {
    generic_check: GenericCheck,
    regex: Regex,
    placeholder: Option<String>,
}

impl CheckConstructor for FileRegexMatch {
    type Output = Self;

    fn from_check_table(
        generic_check: GenericCheck,
        value: toml_edit::Table,
    ) -> Result<Self::Output, CheckDefinitionError> {
        let placeholder = match value.get("__placeholder__") {
            None => None,
            Some(r) => match r.as_str() {
                Some(r) => Some(r.to_string()),
                None => {
                    return Err(CheckDefinitionError::InvalidDefinition(
                        "__placeholder__ is not a string".to_string(),
                    ))
                }
            },
        };

        match value.get("__regex__") {
            None => {
                log::error!("No __regex__ found in {value}");
                Err(CheckDefinitionError::InvalidDefinition(
                    "no __regex__ found".to_string(),
                ))
            }
            Some(regex) => match regex.as_str() {
                None => Err(CheckDefinitionError::InvalidDefinition(format!(
                    "__regex__ ({regex}) is not a string"
                ))),
                Some(s) => match Regex::new(s) {
                    Ok(r) => Ok(Self {
                        generic_check,
                        regex: r,
                        placeholder,
                    }),
                    Err(_) => Err(CheckDefinitionError::InvalidDefinition(format!(
                        "__regex__ ({regex}) is not a valid regex"
                    ))),
                },
            },
        }
    }
}
impl Check for FileRegexMatch {
    fn check_type(&self) -> String {
        "file_regex_match".to_string()
    }

    fn generic_check(&self) -> &GenericCheck {
        &self.generic_check
    }

    fn get_action(&self) -> Result<Action, CheckError> {
        if !self.generic_check().file_to_check().exists() {
            return if let Some(placeholder) = &self.placeholder {
                Ok(Action::SetContents(placeholder.clone()))
            } else {
                Ok(Action::MatchFileRegex {
                    regex: self.regex.as_str().to_string(),
                })
            };
        }

        let contents = self
            .generic_check()
            .get_file_contents(super::DefaultContent::EmptyString)?;

        if self.regex.is_match(contents.as_str()) {
            Ok(Action::None)
        } else {
            Ok(Action::MatchFileRegex {
                regex: self.regex.as_str().to_string(),
            })
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
    fn test_regex_present() {
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
        check_table.insert("__regex__", "export KEY=.*".into());

        let regex_check = FileRegexMatch::from_check_table(generic_check, check_table).unwrap();

        // not existing file
        assert_eq!(
            regex_check.check().unwrap(),
            Action::MatchFileRegex {
                regex: "export KEY=.*".to_string()
            }
        );

        // file with correct contents
        let mut file = File::create(regex_check.generic_check().file_to_check()).unwrap();
        writeln!(file, "export KEY=test").unwrap();
        assert_eq!(regex_check.check().unwrap(), Action::None);

        // file with incorrect contents
        let mut file = File::create(regex_check.generic_check().file_to_check()).unwrap();
        writeln!(file, "export WRONG_KEY=test").unwrap();
        assert_eq!(
            regex_check.check().unwrap(),
            Action::MatchFileRegex {
                regex: "export KEY=.*".to_string()
            }
        );
    }

    #[test]
    fn test_regex_present_with_placeholder() {
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
        check_table.insert("__regex__", "export KEY=.*".into());
        check_table.insert("__placeholder__", "export KEY=value".into());

        let regex_check = FileRegexMatch::from_check_table(generic_check, check_table).unwrap();

        // not existing file
        assert_eq!(
            regex_check.check().unwrap(),
            Action::SetContents("export KEY=value".to_string())
        );

        // file with correct contents
        let mut file = File::create(regex_check.generic_check().file_to_check()).unwrap();
        writeln!(file, "export KEY=test").unwrap();
        assert_eq!(regex_check.check().unwrap(), Action::None);

        // file with incorrect contents
        let mut file = File::create(regex_check.generic_check().file_to_check()).unwrap();
        writeln!(file, "export WRONG_KEY=test").unwrap();
        assert_eq!(
            regex_check.check().unwrap(),
            Action::MatchFileRegex {
                regex: "export KEY=.*".to_string()
            }
        );
    }
}
