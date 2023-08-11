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
        if !self.generic_check().file_to_check().exists() {
            return Ok(Action::SetContents(self.lines.clone()));
        }
        let contents = self
            .generic_check()
            .get_file_contents()
            .map_err(CheckError::FileCanNotBeRead)?;
        if contents.contains(&self.lines) {
            Ok(Action::None)
        } else {
            let mut new_contents = contents.clone();
            if !new_contents.ends_with('\n') {
                new_contents += "\n";
            }
            new_contents += &self.lines.clone();
            Ok(Action::SetContents(new_contents))
        }
    }
}
