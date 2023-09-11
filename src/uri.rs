use dirs::home_dir;
use std::path::{self, Path, PathBuf};

#[warn(unused_imports)]
use std::process::Command;

#[cfg(test)]
fn get_python_module_path(module: &str) -> String {
    format!("/path/to/python/lib/site-packages/{}.py", module)
}

#[cfg(not(test))]
fn get_python_module_path(module: &str) -> String {
    let output = match Command::new("python")
        .args([
            "-c",
            format!(
                "import importlib; print(importlib.import_module('{}').__file__)",
                module
            )
            .as_str(),
        ])
        .output()
    {
        Err(_) => panic!("Python can not be called"),
        Ok(output) => output,
    };

    String::from_utf8(output.stdout).unwrap().trim().to_string()
}

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
    if !package_uri.contains(':') {
        return None;
    }
    let (package_name, path) = package_uri.split_once(':').unwrap();

    Some(
        path::PathBuf::from(get_python_module_path(package_name))
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
        assert!(py_uri_to_path("pathlib".to_string()).is_none(),);

        assert!(py_uri_to_path("py://pathlib".to_string()).is_none(),);

        assert_eq!(
            py_uri_to_path("py://pathlib:asset/file.txt".to_string()).unwrap(),
            path::PathBuf::from("/path/to/python/lib/site-packages\\asset/file.txt")
        );
    }
}
