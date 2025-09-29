use core::fmt::Debug as DebugTrait;
use std::io;
use thiserror::Error;

use crate::{checkers::RelativeUrl, uri::PathError};

use super::GenericChecker;

#[derive(Clone, Debug, PartialEq)]
pub enum CheckResult {
    NoFixNeeded,
    FixNeeded(String),
    FixExecuted(String),
    Error(String),
}

#[derive(Error, Debug, PartialEq)]
pub(crate) enum CheckDefinitionError {
    #[error("invalid check definition ({0})")]
    InvalidDefinition(String),
    #[error("Unknown checktype ({0})")]
    UnknownCheckType(String),
}

#[derive(Error, Debug)]
pub(crate) enum CheckError {
    #[error("file can not be read")]
    FileCanNotBeRead(#[from] io::Error),
    #[error("unknown file type ({0}); do not know how to handle")]
    UnknownFileType(String),
    #[error("file can not be removed")]
    FileCanNotBeRemoved,
    #[error("file can not be written")]
    FileCanNotBeWritten,
    #[error("invalid file format ({0})")]
    InvalidFileFormat(String),
    #[error("invalid regex format ({0})")]
    InvalidRegex(String),
    #[error("permission not available on this system")]
    PermissionsNotAccessable,
    #[error("git error ({0})")]
    GitError(String),
    #[error("file can not be fetched")]
    FetchError(#[from] PathError),
    #[error("{0}")]
    String(String),
}

pub(crate) trait CheckConstructor {
    type Output;
    fn from_check_table(
        generic_check: GenericChecker,
        value: toml_edit::Table,
    ) -> Result<Self::Output, CheckDefinitionError>;
}
pub(crate) trait Checker: DebugTrait {
    fn checker_type(&self) -> String;
    fn generic_checker(&self) -> &GenericChecker;
    fn checker_object(&self) -> String;

    fn list_checker(&self, enabled: bool) {
        log::error!(
            "{} {} - {} - {} - {:?}",
            if enabled { "⬜" } else { " ✖️" },
            self.generic_checker().file_with_checks.short_url_str(),
            self.checker_type(),
            self.checker_object(),
            self.generic_checker().tags
        )
    }
    fn print(&self, check_result: &CheckResult) {
        let (check_result_str, action_message) = match check_result {
            CheckResult::NoFixNeeded => ("✅", "".to_string()),
            CheckResult::FixNeeded(action) => ("❌", format!(" - {action}")),
            CheckResult::FixExecuted(action) => ("🔧", format!(" - {action}")),
            CheckResult::Error(e) => ("🚨", format!(" - {e}")),
        };
        let msg = format!(
            "{} {} - {} - {}{}",
            check_result_str,
            self.generic_checker().file_with_checks().short_url_str(),
            self.checker_type(),
            self.checker_object(),
            action_message
        );
        match check_result {
            CheckResult::NoFixNeeded => log::info!("{msg}"),
            CheckResult::FixExecuted(_) => log::info!("{msg}"),
            CheckResult::FixNeeded(_) => log::warn!("{msg}"),
            CheckResult::Error(_) => log::error!("{msg}"),
        }
    }

    fn check_(&self, fix: bool) -> Result<CheckResult, CheckError>;

    fn check(&self, fix: bool) -> CheckResult {
        let check_result = match self.check_(fix) {
            Ok(check_result) => check_result,
            Err(e) => CheckResult::Error(e.to_string()),
        };

        self.print(&check_result);

        check_result
    }
}
