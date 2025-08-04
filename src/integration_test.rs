#[cfg(test)]
mod tests {
    use crate::checkers;
    use crate::cli;
    use dircpy::*;

    #[test]
    fn test_example() {
        // copy_dir("example", "example2");
        CopyBuilder::new("example/input", "output").run().unwrap();

        let file_with_checks = cli::parse_path("example/pyproject.toml").unwrap();
        let checks = checkers::read_checks_from_path(&file_with_checks);

        let (action_count, success_count) = cli::run_checks(&checks, true);

        assert_eq!(action_count, 0);
        assert_eq!(success_count, 25);
        assert!(!dir_diff::is_different("output", "example/expected_output").unwrap());

        std::fs::remove_dir_all("output").unwrap();
    }
}
