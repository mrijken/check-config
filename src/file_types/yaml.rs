use crate::checkers::base::CheckError;

use super::FileType;

pub(crate) struct Yaml {}

impl Yaml {
    pub(crate) fn new() -> Yaml {
        Yaml {}
    }
}

impl FileType for Yaml {
    fn to_mapping(
        &self,
        contents: &str,
    ) -> Result<Box<dyn crate::mapping::generic::Mapping>, CheckError> {
        crate::mapping::yaml::from_string(contents)
    }
}
