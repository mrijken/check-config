use crate::file_types::RegexValidateResult;

use super::{
    base::{Action, Check, CheckError},
    GenericCheck,
};

#[derive(Debug)]
pub(crate) struct EntryRegexMatch {
    generic_check: GenericCheck,
    value: toml::Table,
}

impl EntryRegexMatch {
    pub fn new(generic_check: GenericCheck, value: toml::Table) -> Self {
        Self {
            generic_check,
            value,
        }
    }
}

impl Check for EntryRegexMatch {
    fn check_type(&self) -> String {
        "key_value_regex_match".to_string()
    }

    fn generic_check(&self) -> &GenericCheck {
        &self.generic_check
    }

    fn get_action(&self) -> Result<Action, CheckError> {
        let contents = self
            .generic_check()
            .get_file_contents(Some("".to_string()))?;

        // Todo: multple actions?
        match self
            .generic_check()
            .file_type()?
            .validate_regex(&contents, &self.value)?
        {
            RegexValidateResult::Invalid {
                key,
                regex,
                found: _,
            } => Ok(Action::MatchRegex { key, regex }),
            RegexValidateResult::Valid => Ok(Action::None),
        }
    }
}
