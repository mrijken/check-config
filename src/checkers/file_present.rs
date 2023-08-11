use super::{
    base::{Action, Check, CheckError},
    GenericCheck,
};

#[derive(Debug)]
pub(crate) struct FilePresent {
    generic_check: GenericCheck,
}

impl FilePresent {
    pub fn new(generic_check: GenericCheck) -> Self {
        Self { generic_check }
    }
}

impl Check for FilePresent {
    fn check_type(&self) -> String {
        "file_present".to_string()
    }

    fn generic_check(&self) -> &GenericCheck {
        &self.generic_check
    }

    fn get_action(&self) -> Result<Action, CheckError> {
        match self.generic_check().file_to_check().exists() {
            false => Ok(Action::SetContents("".to_string())),
            true => Ok(Action::None),
        }
    }
}
