use crate::checkers::{base::CheckDefinitionError, file::get_option_string_value_from_checktable};

pub(crate) fn get_marker_from_check_table(
    value: &toml_edit::Table,
) -> Result<Option<(String, String)>, CheckDefinitionError> {
    let marker_lines = match value.get("marker") {
        None => None,
        Some(marker) => match marker.as_str() {
            None => {
                return Err(CheckDefinitionError::InvalidDefinition(
                    "`marker` is not a string".to_string(),
                ));
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

/// Get the lines from value
/// When absent, return an error or return the default_value when Some
pub(crate) fn get_lines_from_check_table(
    check_table: &toml_edit::Table,
    default_value: Option<String>,
) -> Result<String, CheckDefinitionError> {
    match get_option_string_value_from_checktable(check_table, "lines") {
        Ok(None) => Ok(default_value.unwrap_or("".to_string()).to_string()),
        Ok(Some(lines)) => {
            let lines_with_trailing_new_line_when_not_empty = append_str(&lines, "");
            Ok(lines_with_trailing_new_line_when_not_empty)
        }
        Err(err) => Err(err),
    }
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
        && start_pos < end_pos
    {
        let before = &contents[..start_pos + start_marker.len()];
        let after = &contents[end_pos..];
        return format!("{}{}{}", before, replacement, after);
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
        (contents.find(start_marker), contents.find(end_marker))
        && start_pos < end_pos
    {
        let before = &contents[..start_pos];
        let after = &contents[end_pos + end_marker.len()..];
        return format!("{}{}", before, after);
    }

    contents.to_string()
}

pub(crate) fn append_str(contents: &str, lines: &str) -> String {
    let contents = contents.trim_end();
    let optional_new_line = if contents.is_empty() { "" } else { "\n" };
    format!("{}{}{}", contents, optional_new_line, lines)
}
