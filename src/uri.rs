use derive_more::{Display, From};
use dirs;
use std::path::PathBuf;

#[derive(Debug, From, Display)]
pub enum Error {
    InvalidUrl,
    UnknownUrlScheme,
    NoValidPythonURL,
    #[from]
    IO(std::io::Error),
    #[from]
    Parse(url::ParseError),
    #[from]
    Reqwest(reqwest::Error),
}
impl std::error::Error for Error {}

pub(crate) fn read_to_string(url: &url::Url) -> Result<String, Error> {
    if url.scheme() == "file" {
        Ok(std::fs::read_to_string(url.path())?)
    } else if url.scheme().starts_with("http") {
        Ok(reqwest::blocking::get(url.clone())?.text()?)
    } else {
        Err(Error::UnknownUrlScheme)
    }
}

pub(crate) fn parse_uri(uri: &str, base: Option<&url::Url>) -> Result<url::Url, Error> {
    let url = match url::Url::parse(uri) {
        Ok(url) => url,
        Err(_) => match base {
            Some(base) => base.join(uri)?,
            None => return Err(Error::InvalidUrl),
        },
    };

    if url.scheme() != "py" {
        return Ok(url);
    }
    py_url_to_url(url)
}

#[cfg(test)]
fn get_python_package_path(module: &str) -> Option<url::Url> {
    url::Url::parse(format!("file:///path/to/python/lib/site-packages/{module}").as_str()).ok()
}

#[cfg(not(test))]
fn get_python_package_path(module: &str) -> Option<url::Url> {
    let output = match std::process::Command::new("python")
        .args([
            "-c",
            format!("import importlib; print(importlib.import_module('{module}').__file__)")
                .as_str(),
        ])
        .output()
    {
        Err(_) => {
            log::error!("Python can not be called");
            std::process::exit(1);
        }
        Ok(output) => output,
    };
    let path = String::from_utf8(output.stdout)
        .expect("Read output from Python command")
        .trim()
        .to_string();
    let path = path.rsplit_once('/').unwrap().0;

    url::Url::parse(&format!("file://{path}/")).ok()
}

fn py_url_to_url(package_uri: url::Url) -> Result<url::Url, Error> {
    let package_name = package_uri.host().expect("host is present");

    let module_url = match get_python_package_path(package_name.to_string().as_str()) {
        Some(url) => url,
        None => {
            log::error!("{package_name} is not a valid python package");
            return Err(Error::NoValidPythonURL);
        }
    };
    let path_inside_package_without_leading_slash = package_uri
        .path()
        .split_once('/')
        .expect("valid path with a leading slash")
        .1;

    Ok(module_url.join(path_inside_package_without_leading_slash)?)
}

pub fn expand_to_absolute(path_str: &str) -> std::io::Result<PathBuf> {
    if path_str.starts_with("~") {
        if let Some(home) = dirs::home_dir() {
            let without_tilde = path_str.trim_start_matches("~");
            Ok(home.join(without_tilde.strip_prefix('/').unwrap_or(without_tilde)))
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Homedir can not be found",
            ))
        }
    } else {
        Ok(PathBuf::from(path_str))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_uris() {
        assert_eq!(
            parse_uri("file:///path/to/test", None).unwrap().path(),
            "/path/to/test"
        );
        assert_eq!(
            parse_uri("https://path/to/test", None).unwrap().path(),
            "/to/test"
        );
        assert_eq!(
            parse_uri("py://pathlib/to/test", None).unwrap().path(),
            "/path/to/python/lib/site-packages/to/test"
        );
        assert!(parse_uri("pathlib", None).is_err(),);
        assert!(parse_uri("/path/to/test", None).is_err(),);

        assert_eq!(
            parse_uri(
                "https://path/to/test",
                Some(&parse_uri("https://some/other/path", None).unwrap())
            )
            .unwrap()
            .path(),
            "/to/test"
        );

        assert_eq!(
            parse_uri(
                "test",
                Some(&parse_uri("https://some/other/path", None).unwrap())
            )
            .unwrap()
            .path(),
            "/other/test"
        );

        assert_eq!(
            parse_uri(
                "test",
                Some(&parse_uri("https://some/other/path/", None).unwrap())
            )
            .unwrap()
            .path(),
            "/other/path/test"
        );
        // assert!(py_url_to_url("py://pathlib".to_string()).is_none(),);

        // assert_eq!(
        //     py_url_to_url("py://pathlib:asset/file.txt".to_string()).unwrap(),
        //     path::PathBuf::from("/path/to/python/lib/site-packages/asset/file.txt")
        // );
    }
}
