use dirs::home_dir;
use std::path::{self, Path, PathBuf};
use std::process::Command;

pub(crate) fn uri_to_path(working_path: &Path, include_uri: &str) -> PathBuf {
    if include_uri.starts_with("py") {
        match py_uri_to_path(include_uri.to_string()) {
            Some(path) => return path,
            None => panic!("{} is not a valid python uri", include_uri),
        }
    }
    if include_uri.starts_with('/') {
        return PathBuf::from(include_uri);
    }
    if include_uri.starts_with('~') {
        return home_dir().unwrap().join(&include_uri[2..]);
    }
    working_path.join(include_uri)
}

fn py_uri_to_path(package_uri: String) -> Option<path::PathBuf> {
    if !package_uri.starts_with("py://") {
        return None;
    }
    let package_uri = package_uri.replace("py://", "");
    let (package_name, path) = package_uri.split_once(':').unwrap();
    let output = match Command::new("python")
        .args([
            "-c",
            format!(
                "import importlib; print(importlib.import_module('{}').__file__)",
                package_name
            )
            .as_str(),
        ])
        .output()
    {
        Err(_) => return None,
        Ok(output) => output,
    };

    Some(
        path::PathBuf::from(String::from_utf8(output.stdout).unwrap().trim().to_string())
            .parent()
            .unwrap()
            .to_path_buf()
            .join(path),
    )
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_read_py_packages() {
        assert_eq!(
            py_uri_to_path("pathlib".to_string()).unwrap(),
            path::PathBuf::from("ss")
        );
    }
}
