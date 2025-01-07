use dirs::home_dir;
use std::{
    fs,
    path::{self, PathBuf},
};
use url::Url;

use derive_more::Display;

#[derive(Debug)]
pub(crate) enum Error {
    NoAbsolutePath,
    InvalidUrl,
    UnknownUrlScheme,
    FileCanNotBeRead,
}

#[derive(Debug, Clone, Display)]
pub(crate) enum Uri {
    #[display("Path {}", _0.display())]
    Path(PathBuf),
    #[display("Uri {}", _0)]
    Http(Url),
}

impl Uri {
    pub(crate) fn new(uri: &str) -> Result<Uri, Error> {
        if uri.starts_with("py") {
            match py_uri_to_path(uri.to_string()) {
                Some(path) => return Ok(Uri::Path(path)),
                None => {
                    log::error!("{} is not a valid python uri", uri);
                    return Err(Error::NoAbsolutePath);
                }
            }
        }
        if uri.starts_with("http") {
            return Ok(Uri::Http(Url::parse(uri).map_err(|_| Error::InvalidUrl)?));
        }
        if uri.starts_with("file://") {
            return Ok(Uri::Path(PathBuf::from(uri.replace("file://", "/"))));
        }
        if uri.starts_with('/') {
            return Ok(Uri::Path(PathBuf::from(uri.to_string())));
        }
        if uri.starts_with('~') {
            return Ok(Uri::Path(
                home_dir().expect("Home directory found").join(&uri[2..]),
            ));
        }
        Ok(Uri::Path(PathBuf::from(uri)))
    }

    // get relative url
    pub(crate) fn join(&self, path: &str) -> Result<Uri, Error> {
        match self {
            Uri::Path(path) => Ok(Uri::Path(path.join(path))),
            Uri::Http(url) => Ok(Uri::Http(url.join(path).map_err(|_| Error::InvalidUrl)?)),
        }
    }

    pub(crate) fn read_to_string(&self) -> Result<String, Error> {
        match self {
            Uri::Path(path) => Ok(fs::read_to_string(path).map_err(|_| Error::FileCanNotBeRead)?),

            Uri::Http(url) => Ok(reqwest::blocking::get(url.clone())
                .map_err(|_| Error::FileCanNotBeRead)?
                .text()
                .map_err(|_| Error::FileCanNotBeRead)?),
        }
    }
}

#[cfg(test)]
fn get_python_module_path(module: &str) -> String {
    format!("/path/to/python/lib/site-packages/{}.py", module)
}

#[cfg(not(test))]
fn get_python_module_path(module: &str) -> String {
    let output = match std::process::Command::new("python")
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
        Err(_) => {
            log::error!("Python can not be called");
            std::process::exit(1);
        }
        Ok(output) => output,
    };

    String::from_utf8(output.stdout)
        .expect("Read output from Python command")
        .trim()
        .to_string()
}

fn py_uri_to_path(package_uri: String) -> Option<path::PathBuf> {
    if !package_uri.starts_with("py://") {
        return None;
    }
    let package_uri = package_uri.replace("py://", "");
    if !package_uri.contains(':') {
        return None;
    }
    let (package_name, path) = package_uri.split_once(':').expect(": in uri");

    Some(
        path::PathBuf::from(get_python_module_path(package_name))
            .parent()
            .expect("parent path is present")
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
            path::PathBuf::from("/path/to/python/lib/site-packages/asset/file.txt")
        );
    }
}
