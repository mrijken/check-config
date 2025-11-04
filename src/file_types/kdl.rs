use kdl::KdlDocument;

use crate::checkers::base::CheckError;

use super::FileType;

pub(crate) struct Kdl {}

impl Kdl {
    pub(crate) fn new() -> Kdl {
        Kdl {}
    }
}

impl FileType for Kdl {
    fn to_mapping(
        &self,
        contents: &str,
    ) -> Result<Box<dyn crate::mapping::generic::Mapping>, CheckError> {
        let contents = if contents.trim().is_empty() {
            "root {}".to_string()
        } else {
            format!("root {test}", contents)
        };
        let parsed_doc = kdl::KdlDocument::parse(contents.as_str())
            .map_err(|e| CheckError::InvalidFileFormat(e.to_string()))?;
        let root_node = parsed_doc.get("root").unwrap();
        let json_contents = crate::mapping::kdl::kdl_to_json(root_node);
        dbg!(json_contents.to_string());
        let doc = json_contents
            .as_object()
            .ok_or(CheckError::InvalidFileFormat("No object".to_string()))?;
        Ok(Box::new(doc.clone()))
    }

    fn from_mapping(
        &self,
        mapping: impl crate::mapping::generic::Mapping,
        indent: usize,
    ) -> Result<String, CheckError> {
        let json_str = mapping.to_string(indent)?;

        let json_value: serde_json::Value = serde_json::from_str(json_str.as_str()).unwrap();
        let kdl_root = crate::mapping::kdl::json_to_kdl(&json_value, Some("root"));
        let mut kdl_doc = KdlDocument::new();
        kdl_doc.nodes_mut().push(kdl_root);
        Ok(kdl_doc.to_string())
    }
}
