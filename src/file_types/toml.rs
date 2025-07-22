use crate::checkers::base::CheckError;

use super::FileType;

pub(crate) struct Toml {}

impl Toml {
    pub(crate) fn new() -> Toml {
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
}
