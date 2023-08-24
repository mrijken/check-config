use super::{
    base::{Action, Check, CheckError},
    GenericCheck,
};

#[derive(Debug)]
pub(crate) struct KeyValuePresent {
    generic_check: GenericCheck,
    value: toml::Table,
}

impl KeyValuePresent {
    pub fn new(generic_check: GenericCheck, value: toml::Table) -> Self {
        Self {
            generic_check,
            value,
        }
    }
}

impl Check for KeyValuePresent {
    fn check_type(&self) -> String {
        "key_value_present".to_string()
    }

    fn generic_check(&self) -> &GenericCheck {
        &self.generic_check
    }

    fn get_action(&self) -> Result<Action, CheckError> {
        let contents = self
            .generic_check()
            .get_file_contents(Some("".to_string()))?;

        let new_contents = self
            .generic_check()
            .file_type()?
            .set(&contents, &self.value)
            .unwrap();

        if contents == new_contents {
            Ok(Action::None)
        } else {
            Ok(Action::SetContents(new_contents))
        }
    }
}
