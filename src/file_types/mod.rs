pub mod json;

use crate::{checkers::base::CheckError, mapping::generic::Mapping};
pub mod toml;
pub mod yaml;

#[derive(PartialEq, Clone, Debug)]
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
}
