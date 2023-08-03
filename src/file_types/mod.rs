// pub mod ini;
// pub mod json;
use ::toml::Table;
pub mod toml;
// pub mod yaml;

pub(crate) trait FileType {
    fn set(&self, contents: &str, table_to_set: &Table) -> Result<String, String>;
    fn unset(&self, contents: &str, table_to_unset: &Table) -> Result<String, String>;
    fn validate_regex(&self, contents: &str, table_to_unset: &Table) -> Result<(), String>;
}
