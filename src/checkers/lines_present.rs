use super::{
    base::{Action, Check, CheckError},
    GenericCheck,
};

#[derive(Debug)]
pub(crate) struct LinesPresent {
    generic_check: GenericCheck,
    lines: String,
}

impl LinesPresent {
    pub fn new(generic_check: GenericCheck, lines: String) -> Self {
        let lines = if !lines.ends_with('\n') {
            lines + "\n"
        } else {
            lines
        };
        Self {
            generic_check,
            lines,
        }
    }
}

impl Check for LinesPresent {
    fn check_type(&self) -> String {
        "lines_present".to_string()
    }

    fn generic_check(&self) -> &GenericCheck {
        &self.generic_check
    }

    fn get_action(&self) -> Result<Action, CheckError> {
        let contents = self
            .generic_check()
            .get_file_contents(super::DefaultContent::EmptyString)?;

        if contents.contains(&self.lines) {
            // TODO: check that the content start at the beginning of line
            Ok(Action::None)
        } else {
            let mut new_contents = contents.clone();
            if !new_contents.ends_with('\n') && !new_contents.is_empty() {
                new_contents += "\n";
            }
            new_contents += &self.lines.clone();
            Ok(Action::SetContents(new_contents))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Write;

    use super::*;

    use tempfile::tempdir;

    #[test]
    fn test_add_line_ending_when_needed() {
        let dir = tempdir().unwrap();
        let file_to_check = dir.path().join("file_to_check");
        let file_with_checks = dir.path().join("file_with_checks");
        let generic_check = GenericCheck {
            file_to_check,
            file_type_override: None,
            file_with_checks,
        };

        let lines_present_checker = LinesPresent::new(generic_check.clone(), "".to_string());
        assert_eq!(lines_present_checker.lines, "\n".to_string());

        let lines_present_checker = LinesPresent::new(generic_check.clone(), "1".to_string());
        assert_eq!(lines_present_checker.lines, "1\n".to_string());

        let lines_present_checker = LinesPresent::new(generic_check.clone(), "2\n".to_string());
        assert_eq!(lines_present_checker.lines, "2\n".to_string());
    }

    #[test]
    fn test_lines_present() {
        let dir = tempdir().unwrap();
        let file_to_check = dir.path().join("file_to_check");
        let file_with_checks = dir.path().join("file_with_checks");
        let generic_check = GenericCheck {
            file_to_check,
            file_type_override: None,
            file_with_checks,
        };

        let lines_present_check = LinesPresent::new(generic_check, "1\n2\n".to_string());

        // not existing file
        assert_eq!(
            lines_present_check.check().unwrap(),
            Action::SetContents("1\n2\n".to_string())
        );

        // empty file
        File::create(lines_present_check.generic_check().file_to_check()).unwrap();
        assert_eq!(
            lines_present_check.check().unwrap(),
            Action::SetContents("1\n2\n".to_string())
        );

        // file with other contents
        let mut file = File::create(lines_present_check.generic_check().file_to_check()).unwrap();
        writeln!(file, "a").unwrap();
        assert_eq!(
            lines_present_check.check().unwrap(),
            Action::SetContents("a\n1\n2\n".to_string())
        );

        // file with correct contents
        let mut file = File::create(lines_present_check.generic_check().file_to_check()).unwrap();
        write!(file, "1\n2\nb\n").unwrap();
        assert_eq!(lines_present_check.check().unwrap(), Action::None);
    }
}
