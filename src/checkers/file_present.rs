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

#[cfg(test)]
mod tests {
    use std::fs::File;

    use super::*;

    use tempfile::tempdir;

    #[test]
    fn test_file_present() {
        let dir = tempdir().unwrap();
        let file_to_check = dir.path().join("file_to_check");
        let file_with_checks = crate::uri::Uri::Path(dir.path().join("file_with_checks"));
        let generic_check = GenericCheck {
            file_to_check,
            file_type_override: None,
            file_with_checks,
        };

        let file_present_check = FilePresent::new(generic_check);

        assert_eq!(
            file_present_check.check().unwrap(),
            Action::SetContents("".to_string())
        );

        File::create(file_present_check.generic_check().file_to_check()).unwrap();
        assert_eq!(file_present_check.check().unwrap(), Action::None);
    }
}
