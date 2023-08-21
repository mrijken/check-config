use super::{
    base::{Action, Check, CheckError},
    GenericCheck,
};

#[derive(Debug)]
pub(crate) struct EntryPresent {
    generic_check: GenericCheck,
    value: toml::Table,
}

impl EntryPresent {
    pub fn new(generic_check: GenericCheck, value: toml::Table) -> Self {
        Self {
            generic_check,
            value,
        }
    }
}

impl Check for EntryPresent {
    fn check_type(&self) -> String {
        "entry_present".to_string()
    }

    fn generic_check(&self) -> &GenericCheck {
        &self.generic_check
    }

    fn get_action(&self) -> Result<Action, CheckError> {
        let contents = if !self.generic_check().file_to_check().exists() {
            "".to_string()
        } else {
            self.generic_check().get_file_contents()?
        };

        let new_contents = self
            .generic_check()
            .file_type()?
            .add_entries(&contents, &self.value)
            .unwrap();

        if contents == new_contents {
            Ok(Action::None)
        } else {
            Ok(Action::SetContents(new_contents))
        }
    }
}
