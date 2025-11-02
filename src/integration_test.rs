#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::checkers;
    use crate::cli;
    use crate::uri;
    use dircpy::*;

    #[test]
    #[ignore = "needs internet connection"]
    fn test_example() {
        let _ = std::fs::remove_dir_all("output");

        CopyBuilder::new("example/input", "output").run().unwrap();

        let file_with_checks =
            uri::ReadablePath::from_string("example/pyproject.toml", None).unwrap();
        let mut variables = HashMap::new();
        let checks = checkers::read_checks_from_path(
            &file_with_checks,
            &mut variables,
        )
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
