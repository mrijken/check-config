use super::{
    base::{Action, Check, CheckError},
    GenericCheck,
};

#[derive(Debug)]
pub(crate) struct FileAbsent {
    generic_check: GenericCheck,
}

impl FileAbsent {
    pub fn new(generic_check: GenericCheck) -> Self {
        Self { generic_check }
    }
}

impl Check for FileAbsent {
    fn check_type(&self) -> String {
        "file_absent".to_string()
    }

    fn generic_check(&self) -> &GenericCheck {
        &self.generic_check
    }

    fn get_action(&self) -> Result<Action, CheckError> {
        match self.generic_check().file_to_check().exists() {
            true => Ok(Action::RemoveFile),
            false => Ok(Action::None),
        }
    }
}
