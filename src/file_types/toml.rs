use crate::checkers::base::CheckError;

use super::{generic, FileType, RegexValidateResult};
use toml_edit::Document;
pub(crate) struct Toml {}

impl Toml {
    pub fn new() -> Toml {
        Toml {}
    }
}

impl FileType for Toml {
    fn to_mapping(
        &self,
        contents: &str,
    ) -> Result<Box<dyn crate::mapping::generic::Mapping>, CheckError> {
        crate::mapping::toml::from_string(contents)
    }
    fn set(&self, contents: &str, table_to_set: &toml::Table) -> Result<String, CheckError> {
        let mut doc = convert_string(contents)?;

        generic::set_key_value(doc.as_table_mut(), table_to_set);

        Ok(doc.to_string())
    }

    fn unset(&self, contents: &str, table_to_unset: &toml::Table) -> Result<String, CheckError> {
        let mut doc = convert_string(contents)?;

        generic::unset_key(doc.as_table_mut(), table_to_unset);

        Ok(doc.to_string())
    }

    fn validate_key_value_regex(
        &self,
        contents: &str,
        table_with_regex: &toml::Table,
    ) -> Result<RegexValidateResult, CheckError> {
        let mut doc = convert_string(contents)?;
        generic::validate_key_value_regex(doc.as_table_mut(), table_with_regex, "".to_string())
    }
}

fn convert_string(contents: &str) -> Result<toml_edit::Document, CheckError> {
    let doc = contents
        .parse::<Document>()
        .map_err(|e| CheckError::InvalidFileFormat(e.to_string()))?;
    Ok(doc.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::file_types::RegexValidateResult;

    #[test]
    fn test_unset() {
        let contents = r#"[package]
name = "foo"
version = "1.0"

[dependencies]
bar = { version = "1.0" }
foo = "1.0"
"#;

        let contents_to_unset = r#"
[dependencies.bar]
"#;

        let contents_to_unset = toml::from_str::<toml::Value>(contents_to_unset).unwrap();
        let contents_to_unset = contents_to_unset.as_table().unwrap();

        let new_contents = super::Toml::new()
            .unset(contents, contents_to_unset)
            .unwrap();

        assert_eq!(
            new_contents,
            r#"[package]
name = "foo"
version = "1.0"

[dependencies]
foo = "1.0"
"#
        );
    }

    #[test]
    fn test_set_simple() {
        let contents = r#"[package]
name = "foo"
version = "1.0"
"#;

        let contents_to_set = r#"[package]
name = "bar"
"#;

        let contents_to_set = toml::from_str::<toml::Value>(contents_to_set).unwrap();
        let contents_to_set = contents_to_set.as_table().unwrap();

        let new_contents = super::Toml::new().set(contents, contents_to_set).unwrap();

        assert_eq!(
            new_contents,
            r#"[package]
name = "bar"
version = "1.0"
"#
        );
    }

    #[test]
    fn test_set_nested() {
        let contents = r#"[package]
name = "foo"
version = "1.0"

[dependencies]
toml ="1.0"

[dependencies.bar]
version = "1.0"
features = ["foo"]
"#;

        let contents_to_set = r#"[dependencies.bar]
version = "2.0"
features = ["bar"]
"#;

        let contents_to_set = toml::from_str::<toml::Value>(contents_to_set).unwrap();
        let contents_to_set = contents_to_set.as_table().unwrap();

        let new_contents = super::Toml::new().set(contents, contents_to_set).unwrap();

        assert_eq!(
            new_contents,
            r#"[package]
name = "foo"
version = "1.0"

[dependencies]
toml ="1.0"

[dependencies.bar]
version = "2.0"
features = ["bar"]
"#
        );
    }

    #[test]
    fn test_regex() {
        let contents = r#"[package]
name = "foo"
version = "1.0"

[dependencies]
toml ="1.0"

[dependencies.bar]
version = "1.0"
features = ["foo"]
"#;

        let contents_with_matched_regex = r#"[dependencies.bar]
version = "[0-9]\\.[0-9]"
"#;

        let contents_with_matched_regex =
            toml::from_str::<toml::Value>(contents_with_matched_regex).unwrap();
        let contents_with_matched_regex = contents_with_matched_regex.as_table().unwrap();

        assert_eq!(
            super::Toml::new()
                .validate_key_value_regex(contents, contents_with_matched_regex)
                .unwrap(),
            RegexValidateResult::Valid
        );

        let contents_with_unmatched_regex = r#"[dependencies.bar]
version = "[0-9][0-9]"
"#;

        let contents_with_unmatched_regex =
            toml::from_str::<toml::Value>(contents_with_unmatched_regex).unwrap();
        let contents_with_unmatched_regex = contents_with_unmatched_regex.as_table().unwrap();

        assert_eq!(
            super::Toml::new()
                .validate_key_value_regex(contents, contents_with_unmatched_regex)
                .unwrap(),
            RegexValidateResult::Invalid {
                key: "dependencies.bar.version".to_string(),
                regex: "[0-9][0-9]".to_string(),
                found: "1.0".to_string()
            }
        );
    }
}
