use super::{
    base::{Action, Check, CheckConstructor, CheckDefinitionError, CheckError},
    GenericCheck,
};

#[derive(Debug)]
pub(crate) struct FilePresent {
    generic_check: GenericCheck,
    placeholder: String,
}

impl CheckConstructor for FilePresent {
    type Output = Self;

    fn from_check_table(
        generic_check: GenericCheck,
        value: toml_edit::Table,
    ) -> Result<Self::Output, CheckDefinitionError> {
        let placeholder = match value.get("__placeholder__") {
            None => "",
            Some(s) => match s.as_str() {
                None => {
                    return Err(CheckDefinitionError::InvalidDefinition(
                        "__placeholder__ is not a string".to_string(),
                    ))
                }
                Some(x) => x,
            },
        };

        Ok(Self {
            generic_check,
            placeholder: placeholder.to_string(),
        })
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
            false => Ok(Action::SetContents(self.placeholder.clone())),
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
    fn test_file_absent() {
        let dir = tempdir().unwrap();
        let file_to_check = dir.path().join("file_to_check");
        let file_with_checks =
            url::Url::from_file_path(dir.path().join("file_with_checks")).unwrap();
        let generic_check = GenericCheck {
            file_to_check,
            file_type_override: None,
            file_with_checks,
        };

        let file_present_check =
            FilePresent::from_check_table(generic_check, toml_edit::Table::new()).unwrap();

        assert_eq!(
            file_present_check.check().unwrap(),
            Action::SetContents("".to_string())
        );
    }

    #[test]
    fn test_file_present() {
        let dir = tempdir().unwrap();
        let file_to_check = dir.path().join("file_to_check");
        File::create(&file_to_check).unwrap();
        let file_with_checks =
            url::Url::from_file_path(dir.path().join("file_with_checks")).unwrap();
        let generic_check = GenericCheck {
            file_to_check,
            file_type_override: None,
            file_with_checks,
        };

        let file_present_check =
            FilePresent::from_check_table(generic_check, toml_edit::Table::new()).unwrap();

        assert_eq!(file_present_check.check().unwrap(), Action::None);
    }

    #[test]
    fn test_file_absent_with_placeholder() {
        let dir = tempdir().unwrap();
        let file_to_check = dir.path().join("file_to_check");
        let file_with_checks =
            url::Url::from_file_path(dir.path().join("file_with_checks")).unwrap();
        let generic_check = GenericCheck {
            file_to_check,
            file_type_override: None,
            file_with_checks,
        };

        let mut placeholder_table = toml_edit::Table::new();
        placeholder_table.insert("__placeholder__", "placeholder".into());

        let file_present_check =
            FilePresent::from_check_table(generic_check, placeholder_table).unwrap();

        assert_eq!(
            file_present_check.check().unwrap(),
            Action::SetContents("placeholder".to_string())
        );
    }

    #[test]
    fn test_file_present_with_placeholder() {
        let dir = tempdir().unwrap();
        let file_to_check = dir.path().join("file_to_check");
        File::create(&file_to_check).unwrap();
        let file_with_checks =
            url::Url::from_file_path(dir.path().join("file_with_checks")).unwrap();
        let generic_check = GenericCheck {
            file_to_check,
            file_type_override: None,
            file_with_checks,
        };

        let mut placeholder_table = toml_edit::Table::new();
        placeholder_table.insert("__placeholder__", "placeholder".into());

        let file_present_check =
            FilePresent::from_check_table(generic_check, placeholder_table).unwrap();

        assert_eq!(file_present_check.check().unwrap(), Action::None);
    }
}
