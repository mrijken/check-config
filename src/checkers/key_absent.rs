use super::{
    base::{Action, Check, CheckError},
    GenericCheck,
};
use std::fs;

#[derive(Debug)]
pub(crate) struct KeyAbsent {
    generic_check: GenericCheck,
    value: toml::Table,
}

impl KeyAbsent {
    pub fn new(generic_check: GenericCheck, value: toml::Table) -> Self {
        Self {
            generic_check,
            value,
        }
    }
}

impl Check for KeyAbsent {
    fn check_type(&self) -> String {
        "key_absent".to_string()
    }

    fn generic_check(&self) -> &GenericCheck {
        &self.generic_check
    }

    fn get_action(&self) -> Result<Action, CheckError> {
        if !self.generic_check().file_to_check().exists() {
            return Ok(Action::None);
        }

        let contents = fs::read_to_string(self.generic_check().file_to_check())
            .map_err(CheckError::FileCanNotBeRead)?;

        let new_contents = self
            .generic_check()
            .file_type()?
            .unset(&contents, &self.value)
            .unwrap();

        if contents == new_contents {
            Ok(Action::None)
        } else {
            Ok(Action::SetContents(new_contents))
        }
    }
}
