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

    fn check(&self) -> Result<Action, CheckError> {
        let contents = if !self.generic_check().file_to_check().exists() {
            "".to_string()
        } else {
            let contents = self.generic_check().get_file_contents();
            if let Err(e) = contents {
                log::error!(
                    "Error: {} {} {} {}",
                    e,
                    self.generic_check().file_with_checks().to_string_lossy(),
                    self.generic_check().file_to_check().to_string_lossy(),
                    self.check_type(),
                );
                return Err(e);
            }
            contents.unwrap()
        };

        // Todo: multple actions?
        match self
            .generic_check()
            .file_type()?
            .validate_regex(&contents, &self.value)?
        {
            RegexValidateResult::Invalid(e) => Ok(Action::Manual(e)),
            RegexValidateResult::Valid => {
                self.print_ok();
                Ok(Action::None)
            }
        }
    }
}
