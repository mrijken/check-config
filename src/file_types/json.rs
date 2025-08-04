use crate::checkers::base::CheckError;

use super::FileType;

pub(crate) struct Json {}

impl Json {
    pub(crate) fn new() -> Json {
        Json {}
    }
}

impl FileType for Json {
    fn to_mapping(
        &self,
        contents: &str,
    ) -> Result<Box<dyn crate::mapping::generic::Mapping>, CheckError> {
        crate::mapping::json::from_string(contents)
    }
}
