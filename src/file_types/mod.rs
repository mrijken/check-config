pub mod json;
use ::toml::Table;

use crate::checkers::base::CheckError;
pub mod toml;
pub mod yaml;

#[derive(PartialEq, Debug)]
pub enum RegexValidateResult {
    Valid,
    Invalid(String),
}

pub(crate) trait FileType {
    fn set(&self, contents: &str, table_to_set: &Table) -> Result<String, CheckError>;
    fn unset(&self, contents: &str, table_to_unset: &Table) -> Result<String, CheckError>;
    fn validate_regex(
        &self,
        contents: &str,
        table_to_unset: &Table,
    ) -> Result<RegexValidateResult, CheckError>;
}
