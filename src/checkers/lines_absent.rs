use super::{
    base::{Action, Check, CheckError},
    GenericCheck,
};

#[derive(Debug)]
pub(crate) struct LinesAbsent {
    generic_check: GenericCheck,
    lines: String,
}

impl LinesAbsent {
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

impl Check for LinesAbsent {
    fn check_type(&self) -> String {
        "lines_absent".to_string()
    }

    fn generic_check(&self) -> &GenericCheck {
        &self.generic_check
    }

    fn get_action(&self) -> Result<Action, CheckError> {
        if !self.generic_check.file_to_check().exists() {
            return Ok(Action::None);
        }

        let contents = self
            .generic_check()
            .get_file_contents(super::DefaultContent::None)?;
        if contents.contains(&self.lines) {
            // TODO: check that the content start at the beginning of line
            let new_contents = contents.replace(&self.lines, "");
            Ok(Action::SetContents(new_contents))
        } else {
            Ok(Action::None)
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

        let lines_absent_checker = LinesAbsent::new(generic_check.clone(), "".to_string());
        assert_eq!(lines_absent_checker.lines, "\n".to_string());

        let lines_absent_checker = LinesAbsent::new(generic_check.clone(), "1".to_string());
        assert_eq!(lines_absent_checker.lines, "1\n".to_string());

        let lines_absent_checker = LinesAbsent::new(generic_check.clone(), "2\n".to_string());
        assert_eq!(lines_absent_checker.lines, "2\n".to_string());
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

        let lines_absent_check = LinesAbsent::new(generic_check, "1\n2\n".to_string());

        // not existing file
        assert_eq!(lines_absent_check.check().unwrap(), Action::None);

        // empty file
        File::create(lines_absent_check.generic_check().file_to_check()).unwrap();
        assert_eq!(lines_absent_check.check().unwrap(), Action::None);

        // file with other contents
        let mut file: File =
            File::create(lines_absent_check.generic_check().file_to_check()).unwrap();
        writeln!(file, "a").unwrap();
        assert_eq!(lines_absent_check.check().unwrap(), Action::None);

        // file with incorrect contents
        let mut file: File =
            File::create(lines_absent_check.generic_check().file_to_check()).unwrap();
        write!(file, "1\n2\nb\n").unwrap();
        assert_eq!(
            lines_absent_check.check().unwrap(),
            Action::SetContents("b\n".to_string())
        );
    }
}
