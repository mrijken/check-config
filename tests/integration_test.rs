use check_config::cli;
use check_config::uri;
use dircpy::*;

#[test]
fn test_example() {
    // copy_dir("example", "example2");
    CopyBuilder::new("example/input", "output").run().unwrap();

    let file_with_checks = cli::parse_path("example/pyproject.toml").unwrap();

    let (action_count, success_count) = cli::run_check_for_file(&file_with_checks, true);

    assert_eq!(action_count, 0);
    assert_eq!(success_count, 21);
    assert!(!dir_diff::is_different("output", "example/expected_output").unwrap());

    std::fs::remove_dir_all("output").unwrap();
}
