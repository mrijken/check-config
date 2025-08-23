use crate::checkers::base::CheckDefinitionError;

pub(crate) fn parse_marker_lines(
    value: &toml_edit::Table,
) -> Result<Option<(String, String)>, CheckDefinitionError> {
    let marker_lines = match value.get("__marker__") {
        None => None,
        Some(marker) => match marker.as_str() {
            None => {
                return Err(CheckDefinitionError::InvalidDefinition(
                    "__marker__ is not a string".to_string(),
                ))
            }
            Some(marker) => {
                let marker = marker.trim_end();
                Some((
                    format!("{marker} (check-config start)\n"),
                    format!("{marker} (check-config end)\n"),
                ))
            }
        },
    };

    Ok(marker_lines)
}

/// Get the __lines__ from value
/// When absent, return an error or return the defalt_value when Some
pub(crate) fn parse_lines(
    value: &toml_edit::Table,
    default_value: Option<String>,
) -> Result<String, CheckDefinitionError> {
    let lines = match value.get("__lines__") {
        None => {
            if let Some(default_value) = default_value {
                return Ok(default_value);
            }
            return Err(CheckDefinitionError::InvalidDefinition(
                "__lines__ is not present".to_string(),
            ));
        }
        Some(lines) => match lines.as_str() {
            None => {
                return Err(CheckDefinitionError::InvalidDefinition(
                    "__lines__ is not a string".to_string(),
                ))
            }
            Some(lines) => lines.to_string(),
        },
    };
    let lines_with_trailing_new_line_when_not_empty = append_str(&lines, "");
    Ok(lines_with_trailing_new_line_when_not_empty)
}

/// Replace the text between markers with replacement
/// The markers re not removed
/// When the markers are not present, the markers and replacement
/// are appended to the contents
pub(crate) fn replace_between_markers(
    contents: &str,
    start_marker: &str,
    end_marker: &str,
    replacement: &str,
) -> String {
    if let (Some(start_pos), Some(end_pos)) =
        (contents.find(start_marker), contents.find(end_marker))
    {
        if start_pos < end_pos {
            let before = &contents[..start_pos + start_marker.len()];
            let after = &contents[end_pos..];
            return format!("{}{}{}", before, replacement, after);
        }
    }
    append_str(
        contents,
        format!("{}{}{}", start_marker, replacement, end_marker).as_str(),
    ) // if markers not found or in wrong order, append the string
}

/// Remove the markers and every between it from contents
/// When the markes are not present, the orginal contents are returned
pub(crate) fn remove_between_markers(
    contents: &str,
    start_marker: &str,
    end_marker: &str,
) -> String {
    if let (Some(start_pos), Some(end_pos)) =
        dbg!((contents.find(start_marker), contents.find(end_marker)))
    {
        if start_pos < end_pos {
            let before = &contents[..start_pos];
            let after = &contents[end_pos + end_marker.len()..];
            return dbg!(format!("{}{}", before, after));
        }
    }
    contents.to_string()
}

pub(crate) fn append_str(contents: &str, lines: &str) -> String {
    let contents = contents.trim_end();
    let optional_new_line = if contents.is_empty() { "" } else { "\n" };
    format!("{}{}{}", contents, optional_new_line, lines)
}
