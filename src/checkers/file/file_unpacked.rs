use std::fs::File;

use crate::{
    checkers::{
        base::CheckResult,
        file::{get_option_string_value_from_checktable, get_string_value_from_checktable},
    },
    uri::{ReadPath, WritablePath},
};

use super::super::{
    GenericChecker,
    base::{CheckConstructor, CheckDefinitionError, CheckError, Checker},
};

#[derive(Clone, Debug)]
enum Unpacker {
    TarGz,
    Tar,
    Unzip,
}

#[derive(Debug)]
pub(crate) struct FileUnpacked {
    source: WritablePath,
    destination_dir: WritablePath,
    unpacker: Unpacker,
    generic_check: GenericChecker,
}

// [[file_unpacked]]
// source = "file.zip"
// destination_dir = "path to unpack to"
// unpacker = "zip"  #optional, when not discoverable from extension.
impl CheckConstructor for FileUnpacked {
    type Output = Self;

    fn from_check_table(
        generic_check: GenericChecker,
        check_table: toml_edit::Table,
    ) -> Result<Self::Output, CheckDefinitionError> {
        let source = WritablePath::from_string(
            get_string_value_from_checktable(&check_table, "source")?.as_str(),
        )
        .map_err(|_| CheckDefinitionError::InvalidDefinition("invalid source url".into()))?;

        let destination_dir = WritablePath::from_string(
            get_string_value_from_checktable(&check_table, "destination_dir")?.as_str(),
        )
        .map_err(|_| CheckDefinitionError::InvalidDefinition("invalid destination url".into()))?;

        let unpacker = get_option_string_value_from_checktable(&check_table, "unpacker")?;
        let unpacker = if let Some(unpacker) = unpacker {
            unpacker
        } else {
            let file_name = source
                .as_ref()
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string();
            if file_name.ends_with(".zip") {
                "zip".to_string()
            } else if file_name.ends_with(".tar.gz") {
                "tar.gz".to_string()
            } else if file_name.ends_with(".tar") {
                "tar".to_string()
            } else {
                "unknown".to_string()
            }
        };
        let unpacker = match unpacker.as_str() {
            "tar.gz" => Unpacker::TarGz,
            "tar" => Unpacker::Tar,
            "zip" => Unpacker::Unzip,
            _ => {
                return Err(CheckDefinitionError::InvalidDefinition(format!(
                    "invalid unpacker {}",
                    unpacker
                )));
            }
        };

        Ok(Self {
            destination_dir,
            source,
            unpacker,
            generic_check,
        })
    }
}
impl Checker for FileUnpacked {
    fn checker_type(&self) -> String {
        "file_unpacked".to_string()
    }

    fn generic_checker(&self) -> &GenericChecker {
        &self.generic_check
    }
    fn checker_object(&self) -> String {
        self.source.as_ref().to_string_lossy().to_string()
    }
    fn check_(&self, fix: bool) -> Result<crate::checkers::base::CheckResult, CheckError> {
        let mut action_messages: Vec<String> = vec![];

        match self.source.exists() {
            Ok(false) => return Err(CheckError::String("source file does not exists".into())),
            Ok(true) => (),
            Err(e) => return Err(CheckError::String(e.to_string())),
        };

        let file_unpack = !self.destination_dir.as_ref().exists();

        if file_unpack {
            action_messages.push("unpack file".into());
        }

        let fix_needed = file_unpack;

        let action_message = action_messages.join("\n");

        let check_result = match (fix, fix_needed) {
            (true, true) => {
                match self.unpacker {
                    Unpacker::Tar => {
                        tar::Archive::new(File::open(self.source.as_ref()).expect("file exists"))
                            .unpack(self.destination_dir.as_ref())
                            .map_err(|e| CheckError::String(e.to_string()))?;
                    }

                    Unpacker::TarGz => {
                        let tar = flate2::read::GzDecoder::new(
                            File::open(self.source.as_ref()).expect("file exists"),
                        );
                        tar::Archive::new(tar)
                            .unpack(self.destination_dir.as_ref())
                            .map_err(|e| CheckError::String(e.to_string()))?;
                    }
                    Unpacker::Unzip => {
                        zip::read::ZipArchive::new(
                            File::open(self.source.as_ref()).expect("file exists"),
                        )
                        .map_err(|e| CheckError::String(e.to_string()))?
                        .extract(self.destination_dir.as_ref())
                        .map_err(|e| CheckError::String(e.to_string()))?;
                    }
                }

                CheckResult::FixExecuted(action_message)
            }
            (true, false) => CheckResult::NoFixNeeded,
            (false, false) => CheckResult::NoFixNeeded,
            (false, true) => CheckResult::FixNeeded(action_message),
        };

        Ok(check_result)
    }
}

#[cfg(test)]
mod tests {

    use crate::checkers::test_helpers;

    use super::*;

    use tempfile::tempdir;

    fn get_file_unpacked_check_with_result(
        dir: String,
        unpacker: Option<String>,
    ) -> (
        Result<FileUnpacked, CheckDefinitionError>,
        tempfile::TempDir,
    ) {
        let generic_check = test_helpers::get_generic_check();

        let mut check_table = toml_edit::Table::new();
        let tmp_dir = tempdir().unwrap();
        let file_to_check = tmp_dir.path().join("file_to_check");
        check_table.insert("file", file_to_check.to_string_lossy().to_string().into());
        check_table.insert("dir", dir.into());

        if let Some(unpacker) = unpacker {
            check_table.insert("unpacker", unpacker.into());
        }

        (
            FileUnpacked::from_check_table(generic_check, check_table),
            tmp_dir,
        )
    }

    fn get_file_unpacked_check(
        dir: String,
        unpacker: Option<String>,
    ) -> (FileUnpacked, tempfile::TempDir) {
        let (file_unpacked_with_result, tempdir) =
            get_file_unpacked_check_with_result(dir, unpacker);

        (
            file_unpacked_with_result.expect("check without issues"),
            tempdir,
        )
    }

    // #[test]
    // fn test_file_present() {
    //     let (file_present_check, _tempdir) = get_file_present_check(None, None, None);
    //
    //     assert_eq!(
    //         file_present_check.check(false).unwrap(),
    //         CheckResult::FixNeeded("create file".into())
    //     );
    //
    //     assert_eq!(
    //         file_present_check.check(true).unwrap(),
    //         CheckResult::FixExecuted("create file".into())
    //     );
    //     assert_eq!(
    //         file_present_check.check(false).unwrap(),
    //         CheckResult::NoFixNeeded
    //     );
    // }
    //
    // #[test]
    // fn test_file_present_with_placeholder() {
    //     let (file_present_check, _tempdir) =
    //         get_file_present_check(Some("placeholder".into()), None, None);
    //
    //     assert_eq!(
    //         file_present_check.check(false).unwrap(),
    //         CheckResult::FixNeeded("create file\nset contents to placeholder".into())
    //     );
    //
    //     assert_eq!(
    //         file_present_check.check(true).unwrap(),
    //         CheckResult::FixExecuted("create file\nset contents to placeholder".into())
    //     );
    //     assert_eq!(
    //         file_present_check.check(false).unwrap(),
    //         CheckResult::NoFixNeeded
    //     );
    // }
    //
    // #[test]
    // fn test_file_present_with_permissions() {
    //     let (file_present_check, _tempdir) = get_file_present_check(None, Some("666".into()), None);
    //
    //     assert_eq!(
    //         file_present_check.check(false).unwrap(),
    //         CheckResult::FixNeeded("create file\nfix permissions to 666".into())
    //     );
    //
    //     assert_eq!(
    //         file_present_check.check(true).unwrap(),
    //         CheckResult::FixExecuted("create file\nfix permissions to 666".into())
    //     );
    //
    //     assert_eq!(
    //         file_present_check.check(false).unwrap(),
    //         CheckResult::NoFixNeeded
    //     );
    // }
    //
    // #[test]
    // fn test_file_present_with_regex() {
    //     let file_present_error =
    //         get_file_present_check_with_result(None, None, Some("^[0-9]{1,3$".into()))
    //             .0
    //             .expect_err("must give error");
    //
    //     assert_eq!(
    //         file_present_error,
    //         CheckDefinitionError::InvalidDefinition(
    //             "regex (\"^[0-9]{1,3$\") is not a valid regex".into()
    //         )
    //     );
    //     let (file_present_check, _tempdir) =
    //         get_file_present_check(None, None, Some("[0-9]{1,3}".into()));
    //
    //     assert_eq!(
    //         file_present_check.check(false).unwrap(),
    //         CheckResult::FixNeeded("create file\nfix content to match regex \"[0-9]{1,3}\"".into())
    //     );
    //
    //     let _ = write(file_present_check.file_check.file_to_check.clone(), "bla");
    //
    //     assert_eq!(
    //         file_present_check.check(false).unwrap(),
    //         CheckResult::FixNeeded("fix content to match regex \"[0-9]{1,3}\"".into())
    //     );
    //
    //     let _ = write(file_present_check.file_check.file_to_check.clone(), "129");
    //
    //     assert_eq!(
    //         file_present_check.check(false).unwrap(),
    //         CheckResult::NoFixNeeded
    //     );
    // }
}
