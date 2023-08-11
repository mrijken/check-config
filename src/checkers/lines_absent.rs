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
            return Ok(Action::RemoveFile);
        }

        let contents = self
            .generic_check()
            .get_file_contents()
            .map_err(CheckError::FileCanNotBeRead)?;
        if contents.contains(&self.lines) {
            let new_contents = contents.replace(&self.lines, "");
            Ok(Action::SetContents(new_contents))
        } else {
            Ok(Action::None)
        }
    }
}
