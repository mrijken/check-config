use derive_more::{AsRef, Display, From};
use dirs;
use std::{io::Write, path::PathBuf};
use url::Url;

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

pub(crate) fn read_to_string(url: &Url) -> Result<String, Error> {
    if url.scheme() == "file" {
        Ok(std::fs::read_to_string(url.path())?)
    } else if url.scheme().starts_with("http") {
        Ok(reqwest::blocking::get(url.clone())?.text()?)
    } else {
        Err(Error::UnknownUrlScheme)
    }
}

/// parse uri of files to read to or write from
/// readable:
/// - http(s) uri
/// - python path
/// - relative to config (ie any other readable uri)
/// - + writable path
///
/// writable
/// - local filesystem
///   - relative to home dir
///   - relative to check file dir
///   - absolute pat
///

#[derive(thiserror::Error, Debug)]
pub enum PathError {
    #[error("unsupported scheme: {0}")]
    UnsupportedScheme(String),

    #[error("reqwest error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("url parsing error: {0}")]
    UrlParse(#[from] url::ParseError),
}

#[derive(AsRef, Clone, Debug)]
pub struct ReadablePath(Url);

pub trait ReadPath {
    fn exists(&self) -> Result<bool, PathError>;

    fn read_to_string(&self) -> Result<String, PathError>;

    fn copy(&self, dest: &WritablePath) -> Result<(), PathError>;
}

impl ReadablePath {
    pub fn from_string(input: &str, config_base: &Url) -> Result<ReadablePath, Error> {
        if input.starts_with("config:") {
            let input = input.replacen("config:", "", 1);
            return Ok(ReadablePath(
                config_base
                    .join(input.as_str())
                    .map_err(|_e| Error::InvalidUrl)?,
            ));
        }

        if let Ok(url) = Url::parse(input) {
            if url.scheme() == "py" {
                return Ok(ReadablePath(py_url_to_url(url)?));
            }
            return Ok(ReadablePath(url));
        }

        Ok(ReadablePath(
            Url::from_file_path(WritablePath::from_string(input)?.as_ref())
                .map_err(|_| Error::InvalidUrl)?,
        ))
    }
}

impl ReadPath for ReadablePath {
    fn copy(&self, dest: &WritablePath) -> Result<(), PathError> {
        // todo: create parent dir if needed

        match self.as_ref().scheme() {
            "file" => {
                let path = self
                    .as_ref()
                    .to_file_path()
                    .map_err(|_| PathError::UnsupportedScheme("invalid file path".into()))?;
                std::fs::copy(&path, dest.as_ref())?;
                Ok(())
            }
            "http" | "https" => {
                let resp = reqwest::blocking::get(self.as_ref().clone())?;
                let bytes = resp.bytes()?;
                let mut out = std::fs::File::create(dest.as_ref())?;
                out.write_all(&bytes)?;
                Ok(())
            }
            other => Err(PathError::UnsupportedScheme(other.into())),
        }
    }

    fn read_to_string(&self) -> Result<String, PathError> {
        match self.as_ref().scheme() {
            "file" => Ok(std::fs::read_to_string(
                self.as_ref()
                    .to_file_path()
                    .expect("an url with a file scheme is a valid file path"),
            )?),
            "http" | "https" => Ok(reqwest::blocking::get(self.as_ref().clone())?.text()?),
            other => Err(PathError::UnsupportedScheme(other.into())),
        }
    }

    fn exists(&self) -> Result<bool, PathError> {
        match self.as_ref().scheme() {
            "file" => Ok(self
                .as_ref()
                .to_file_path()
                .map_err(|_| PathError::UnsupportedScheme("invalid file path".into()))?
                .exists()),
            "http" | "https" => {
                let resp = reqwest::blocking::get(self.as_ref().clone())?;
                Ok(resp.status().is_success())
            }
            other => Err(PathError::UnsupportedScheme(other.into())),
        }
    }
}

#[derive(AsRef, Clone, Debug)]
pub struct WritablePath(PathBuf);

impl WritablePath {
    pub fn new(path: PathBuf) -> WritablePath {
        WritablePath(path)
    }

    pub fn from_string(input: &str) -> Result<WritablePath, Error> {
        if input.starts_with("~") {
            if let Some(home) = dirs::home_dir() {
                let expanded = input.replacen("~", home.to_str().unwrap(), 1);
                return Ok(WritablePath(PathBuf::from(expanded)));
            }
            return Err(Error::InvalidUrl);
        }

        if input.starts_with("/") {
            return Ok(WritablePath(PathBuf::from(input)));
        }

        let cwd = std::env::current_dir()
            .map_err(|e| e.to_string())
            .map_err(|_| Error::InvalidUrl)?;
        let full_path = cwd.join(input);
        Ok(WritablePath(full_path))
    }
}

impl ReadPath for WritablePath {
    fn read_to_string(&self) -> Result<String, PathError> {
        Ok(std::fs::read_to_string(self.as_ref())?)
    }

    fn exists(&self) -> Result<bool, PathError> {
        Ok(self.as_ref().exists())
    }

    fn copy(&self, dest: &WritablePath) -> Result<(), PathError> {
        std::fs::copy(self.as_ref(), dest.as_ref())?;
        Ok(())
    }
}

pub(crate) fn parse_uri(input: &str, base: Option<&Url>) -> Result<Url, Error> {
    // Case 1: Try parsing directly as a URL
    if let Ok(url) = Url::parse(input) {
        if url.scheme() == "py" {
            return py_url_to_url(url);
        }
        return Ok(url);
    }

    // Case 2: Handle "~" expansion
    if input.starts_with("~") {
        if let Some(home) = dirs::home_dir() {
            let expanded = input.replacen("~", home.to_str().unwrap(), 1);
            return Url::from_file_path(PathBuf::from(expanded)).map_err(|_| Error::InvalidUrl);
        }
        return Err(Error::InvalidUrl);
    }

    // Case 3: absolute
    if input.starts_with("/") {
        return Url::from_file_path(PathBuf::from(input)).map_err(|_| Error::InvalidUrl);
    }

    // Case 4: relative with given base
    if let Some(base) = base {
        return Ok(base.join(input)?);
    }

    // Case 5: relative without given base, so use cwd
    let cwd = std::env::current_dir()
        .map_err(|e| e.to_string())
        .map_err(|_| Error::InvalidUrl)?;
    let full_path = cwd.join(input);
    Url::from_file_path(&full_path).map_err(|_| Error::InvalidUrl)
}

#[cfg(test)]
fn get_python_package_path(module: &str) -> Option<Url> {
    Url::parse(format!("file:///path/to/python/lib/site-packages/{module}").as_str()).ok()
}

#[cfg(not(test))]
fn get_python_package_path(module: &str) -> Option<Url> {
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

    Url::parse(&format!("file://{path}/")).ok()
}

fn py_url_to_url(package_uri: Url) -> Result<Url, Error> {
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

#[cfg(test)]
mod test {
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn test_paths_copy_and_read() {
        let dir = tempdir().unwrap();
        let destination = dir.path().join("destination");
        let destination = WritablePath(destination);

        let source = ReadablePath::from_string(
            "https://rust-lang.org/static/images/rust-logo-blk.svg",
            &Url::from_file_path(dir.path()).expect("valid path"),
        )
        .expect("valid url");

        source.copy(&destination).unwrap();

        assert_eq!(
            destination.read_to_string().unwrap(),
            source.read_to_string().unwrap()
        )
    }

    #[test]
    fn test_exists() {
        let dir = tempdir().unwrap();
        assert!(
            ReadablePath::from_string(
                "https://rust-lang.org/static/images/rust-logo-blk.svg",
                &Url::from_file_path(dir.path()).expect("valid path"),
            )
            .unwrap()
            .exists()
            .unwrap()
        );

        assert!(
            !ReadablePath::from_string(
                "https://rust-lang.org/non_existing",
                &Url::from_file_path(dir.path()).expect("valid path"),
            )
            .unwrap()
            .exists()
            .unwrap()
        );

        let tmp_path = dir.path().join("tmp_file");

        assert!(!WritablePath(tmp_path.clone()).exists().unwrap());

        std::fs::File::create(&tmp_path).unwrap();

        assert!(WritablePath(tmp_path).exists().unwrap());
    }

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
        assert!(
            parse_uri("pathlib", None)
                .unwrap()
                .path()
                .ends_with("check-config/pathlib")
        );
        assert_eq!(
            parse_uri("/path/to/test", None).unwrap().path(),
            "/path/to/test"
        );

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
