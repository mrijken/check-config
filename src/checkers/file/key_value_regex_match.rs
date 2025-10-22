use regex::Regex;

use crate::{
    checkers::{
        base::{CheckDefinitionError, CheckResult},
        file::{FileCheck, get_option_string_value_from_checktable},
    },
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
    key_regex: toml_edit::Table,
    placeholder: Option<String>,
}

// [key_value_regex_matched]
// file = "file"
// key.key = "regex"
// placeholder = "optional value to be set when key is absent"
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
            Some(key_value_regex) => match key_value_regex.as_table() {
                None => {
                    return Err(CheckDefinitionError::InvalidDefinition(
                        "`key` is not a table".into(),
                    ));
                }
                Some(key_value_regex) => key_value_regex.clone(),
            },
        };

        let placeholder = get_option_string_value_from_checktable(&check_table, "placeholder")?;

        Ok(Self {
            file_check,
            key_regex: key_value_regex,
            placeholder,
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

        let fix_needed = match validate_key_value_regex(
            doc.as_mut(),
            &self.key_regex,
            "".to_string(),
            self.placeholder.clone(),
        ) {
            Ok(RegexValidateResult::Valid) => false,
            Ok(RegexValidateResult::Invalid {
                key: _,
                regex: _,
                found: _,
            }) => true,
            Err(e) => return Err(CheckError::InvalidRegex(e.to_string())),
        };

        let action_message = match fix_needed {
            false => "".to_string(),
            true => {
                if self.placeholder.is_none() {
                    "content of key does not match regex".to_string()
                } else {
                    format!(
                        "content of key does not match regex (setting placeholder to {})",
                        self.placeholder.clone().unwrap()
                    )
                }
            }
        };

        match (fix_needed, fix) {
            (false, _) => Ok(crate::checkers::base::CheckResult::NoFixNeeded),
            (true, false) => Ok(CheckResult::FixNeeded(action_message)),
            (true, true) => {
                if self.placeholder.is_some() {
                    self.file_check.conclude_check_with_new_doc(doc, fix)?;
                    Ok(CheckResult::FixExecuted(action_message))
                } else {
                    Ok(CheckResult::FixNeeded(action_message))
                }
            }
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
    placeholder: Option<String>,
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
                        if let Some(placeholder) = placeholder {
                            doc.insert(
                                &key.to_string().into(),
                                &toml_edit::Item::Value(placeholder.into()),
                            );
                        }
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
                    return validate_key_value_regex(
                        child_doc,
                        t,
                        make_key_path(&key_path, key),
                        placeholder,
                    );
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
                validate_key_value_regex(test_input.as_mut(), &checker, "".to_string(), None)
                    .unwrap();

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
