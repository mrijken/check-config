use regex::Regex;

use crate::{
    checkers::{base::CheckDefinitionError, file::FileCheck},
    file_types::RegexValidateResult,
    mapping::generic::Mapping,
};

use super::super::{
    GenericChecker,
    base::{CheckConstructor, CheckError, Checker},
};

#[derive(Debug)]
pub(crate) struct EntryRegexMatched {
    file_check: FileCheck,
    value: toml_edit::Table,
}

// [key_value_regex_matched]
// file = "file"
// key.key = "regex"
impl CheckConstructor for EntryRegexMatched {
    type Output = Self;

    fn from_check_table(
        generic_check: GenericChecker,
        check_table: toml_edit::Table,
    ) -> Result<Self::Output, CheckDefinitionError> {
        let file_check = FileCheck::from_check_table(generic_check, &check_table)?;

        let key_value_regex = match check_table.get("key") {
            None => {
                return Err(CheckDefinitionError::InvalidDefinition(
                    "`key` key is not present".into(),
                ));
            }
            Some(absent) => match absent.as_table() {
                None => {
                    return Err(CheckDefinitionError::InvalidDefinition(
                        "`key` is not a table".into(),
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
            value: key_value_regex,
        })
    }
}
impl Checker for EntryRegexMatched {
    fn checker_type(&self) -> String {
        "key_value_regex_matched".to_string()
    }

    fn checker_object(&self) -> String {
        self.file_check.check_object()
    }

    fn generic_checker(&self) -> &GenericChecker {
        &self.file_check.generic_check
    }

    fn check_(&self, fix: bool) -> Result<crate::checkers::base::CheckResult, CheckError> {
        let mut doc = self.file_check.get_mapping()?;

        match validate_key_value_regex(doc.as_mut(), &self.value, "".to_string()) {
            Ok(RegexValidateResult::Valid) => Ok(crate::checkers::base::CheckResult::NoFixNeeded),
            Ok(RegexValidateResult::Invalid {
                key,
                regex,
                found: _,
            }) => {
                let mut action_message =
                    format!("content of {} does not match regex {:?}", key, regex);
                if fix {
                    action_message += " (not fixable via --fix)";
                }
                Ok(crate::checkers::base::CheckResult::FixNeeded(
                    action_message,
                ))
            }
            Err(e) => Err(CheckError::InvalidRegex(e.to_string())),
        }
    }
}

fn make_key_path(parent: &str, key: &str) -> String {
    if parent.is_empty() {
        key.to_string()
    } else {
        parent.to_string() + "." + key
    }
}

fn validate_key_value_regex(
    doc: &mut dyn Mapping,
    table_with_regex: &toml_edit::Table,
    key_path: String,
) -> Result<RegexValidateResult, CheckError> {
    for (key, value) in table_with_regex {
        match value {
            toml_edit::Item::Value(toml_edit::Value::String(raw_regex)) => {
                match doc.get_string(key) {
                    Ok(string_to_match) => {
                        let regex = match Regex::new(raw_regex.value()) {
                            Ok(regex) => regex,
                            Err(s) => return Err(CheckError::InvalidRegex(s.to_string())),
                        };
                        if regex.is_match(string_to_match.as_str()) {
                            return Ok(RegexValidateResult::Valid);
                        } else {
                            return Ok(RegexValidateResult::Invalid {
                                key: make_key_path(&key_path, key),
                                regex: raw_regex.value().to_owned(),
                                found: string_to_match.clone(),
                            });
                        }
                    }
                    _ => {
                        return Ok(RegexValidateResult::Invalid {
                            key: make_key_path(&key_path, key),
                            regex: raw_regex.value().to_owned(),
                            found: "".to_string(),
                        });
                    }
                }
            }
            toml_edit::Item::Table(t) => match doc.get_mapping(key, false) {
                Ok(child_doc) => {
                    return validate_key_value_regex(child_doc, t, make_key_path(&key_path, key));
                }
                _ => {
                    return Ok(RegexValidateResult::Invalid {
                        key: make_key_path(&key_path, key),
                        regex: "".to_string(),
                        found: "".to_string(),
                    });
                }
            },

            _ => {}
        }
    }
    Ok(RegexValidateResult::Valid)
}

#[cfg(test)]
mod tests {
    use crate::checkers::test_helpers::read_test_files;

    use super::*;

    #[test]
    fn test_test_files() {
        for (test_path, test_input, test_expected_output, checker) in
            read_test_files("key_value_regex_matched")
        {
            let mut test_input = test_input;
            let result =
                validate_key_value_regex(test_input.as_mut(), &checker, "".to_string()).unwrap();

            if test_expected_output.contains("true") {
                assert_eq!(
                    result,
                    RegexValidateResult::Valid,
                    "test_path {test_path} failed"
                );
            } else {
                assert_ne!(
                    result,
                    RegexValidateResult::Valid,
                    "test_path {test_path} failed"
                );
            }
        }
    }
}
