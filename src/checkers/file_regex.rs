use regex::Regex;

use super::{
    base::{Action, Check, CheckError},
    GenericCheck,
};

#[derive(Debug)]
pub(crate) struct FileRegexMatch {
    generic_check: GenericCheck,
    regex: String,
}

impl FileRegexMatch {
    pub fn new(generic_check: GenericCheck, regex: String) -> Self {
        Self {
            generic_check,
            regex,
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
            return Ok(Action::MatchFileRegex {
                regex: self.regex.clone(),
            });
        }

        let contents = self
            .generic_check()
            .get_file_contents(super::DefaultContent::EmptyString)?;

        let regex = match Regex::new(self.regex.as_str()) {
            Ok(regex) => regex,
            Err(s) => return Err(CheckError::InvalidRegex(s.to_string())),
        };
        if regex.is_match(contents.as_str()) {
            Ok(Action::None)
        } else {
            Ok(Action::MatchFileRegex {
                regex: self.regex.clone(),
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
        let file_with_checks = crate::uri::Uri::Path(dir.path().join("file_with_checks"));
        let generic_check = GenericCheck {
            file_to_check,
            file_type_override: None,
            file_with_checks,
        };

        let regex_check = FileRegexMatch::new(generic_check, "export KEY=.*".to_string());

        // not existing file
        let mut file = File::create(regex_check.generic_check().file_to_check()).unwrap();
        assert_eq!(
            regex_check.check().unwrap(),
            Action::MatchFileRegex {
                regex: "export KEY=.*".to_string()
            }
        );

        // file with correct contents
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
