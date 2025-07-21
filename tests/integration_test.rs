pub use dircpy::*;

#[test]
fn test_example() {
    // copy_dir("example", "example2");
    CopyBuilder::new("example/input", "output").run().unwrap();

    let file_with_checks =
        check_config::uri::parse_uri("file:://example/pyproject.toml", None).unwrap();

    check_config::cli::run_check_for_file(&file_with_checks, true);

    assert!(dir_diff::is_different("output", "example/expected_output").unwrap());

    std::fs::remove_dir_all("output").unwrap();
}
