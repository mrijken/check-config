use std::{fs, path::PathBuf, str::FromStr};

use crate::mapping::{generic::Mapping, json};

type TestFiles = Vec<(String, Box<dyn Mapping>, String, toml_edit::Table)>;

#[allow(dead_code)]
pub(crate) fn read_test_files(check_type: &str) -> TestFiles {
    let mut test_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    test_dir.push("tests/resources/checkers/".to_string() + check_type);

    let mut tests = vec![];

    for test in test_dir.read_dir().expect("read_dir call failed") {
        let test = test.unwrap().path();
        let file_checker_content = fs::read_to_string(test.join("checker.toml")).unwrap();
        let file_checker = toml_edit::DocumentMut::from_str(file_checker_content.as_str())
            .unwrap()
            .as_table()
            .clone();

        let json_input = json::from_path(test.join("input.json")).unwrap();
        let json_expected_output =
            fs::read_to_string(test.join("expected_output.json")).unwrap() + "\n";

        tests.push((
            test.join("input.json").to_string_lossy().to_string(),
            json_input,
            json_expected_output,
            file_checker.clone(),
        ));

        let toml_input = crate::mapping::toml::from_path(test.join("input.toml")).unwrap();
        let toml_expected_output = fs::read_to_string(test.join("expected_output.toml")).unwrap();

        tests.push((
            test.join("input.toml").to_string_lossy().to_string(),
            toml_input,
            toml_expected_output,
            file_checker.clone(),
        ));

        let yaml_input = crate::mapping::yaml::from_path(test.join("input.yaml")).unwrap();
        let yaml_expected_output = fs::read_to_string(test.join("expected_output.yaml")).unwrap();

        tests.push((
            test.join("input.yaml").to_string_lossy().to_string(),
            yaml_input,
            yaml_expected_output,
            file_checker.clone(),
        ));
    }
    tests
}
