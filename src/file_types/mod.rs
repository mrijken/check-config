pub mod json;
use ::toml::Table;

use crate::{checkers::base::CheckError, mapping::generic::Mapping};
mod generic;
pub mod toml;
pub mod yaml;

#[derive(PartialEq, Debug)]
pub enum RegexValidateResult {
    Valid,
    Invalid {
        key: String,
        regex: String,
        found: String,
    },
}

pub(crate) trait FileType {
    fn to_mapping(&self, contents: &str) -> Result<Box<dyn Mapping>, CheckError>;

    fn set(&self, contents: &str, table_to_set: &Table) -> Result<String, CheckError>;
    fn unset(&self, contents: &str, table_to_unset: &Table) -> Result<String, CheckError>;
    fn validate_key_value_regex(
        &self,
        contents: &str,
        table_to_unset: &Table,
    ) -> Result<RegexValidateResult, CheckError>;
}
