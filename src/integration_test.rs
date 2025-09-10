#[cfg(test)]
mod tests {
    use crate::checkers;
    use crate::cli;
    use dircpy::*;

    #[test]
    fn test_example() {
        let _ = std::fs::remove_dir_all("output");

        CopyBuilder::new("example/input", "output").run().unwrap();

        let file_with_checks = cli::parse_path_str_to_uri("example/pyproject.toml").unwrap();
        let checks =
            checkers::read_checks_from_path(&file_with_checks, vec!["tool", "check-config"])
                .into_iter()
                .filter(|c| {
                    cli::filter_checks(
                        &c.generic_checker().tags,
                        &[],
                        &[],
                        &["not_selected".to_string()],
                    )
                })
                .collect();

        assert_eq!(cli::run_checks(&checks, true), cli::ExitStatus::Success);

        assert!(!dir_diff::is_different("output", "example/expected_output").unwrap());

        std::fs::remove_dir_all("output").unwrap();
    }
}
