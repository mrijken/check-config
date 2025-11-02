use derive_more::{AsRef, Display, From};
use dirs;
use std::{
    hash::{Hash, Hasher},
    io::Write,
    path::PathBuf,
};
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

    #[error("content is not a string")]
    ContentIsNoString,
}

#[derive(AsRef, Clone, Debug, Display)]
pub struct ReadablePath(Url);

pub trait ReadPath {
    fn exists(&self) -> Result<bool, PathError>;

    fn read_to_string(&self) -> Result<String, PathError> {
        let bytes = self.read_to_bytes()?;
        String::from_utf8(bytes).map_err(|_| PathError::ContentIsNoString)
    }

    fn is_utf8(&self) -> Result<bool, PathError> {
        match self.read_to_string() {
            Err(PathError::ContentIsNoString) => Ok(false),
            Ok(_) => Ok(true),
            Err(e) => Err(e),
        }
    }

    fn read_to_bytes(&self) -> Result<Vec<u8>, PathError>;

    fn copy(&self, dest: &WritablePath) -> Result<(), PathError>;

    fn hash(&self) -> Result<u64, PathError> {
        let mut hasher = std::hash::DefaultHasher::new();
        let content = self.read_to_bytes()?;
        content.hash(&mut hasher);
        Ok(hasher.finish())
    }
}

impl ReadablePath {
    pub fn from_url(url: Url) -> ReadablePath {
        ReadablePath(url)
    }
    /// get a readable path
    /// - config:<path>  - relative to current_config_path
    /// - https://<path> - uri
    /// - file://<path>  - absolute path
    /// - py://<package>/<path>
    /// - ~/<path>       - relative to home dir
    /// - <path>         - relative to cwd
    /// - /<path>        - absolute path
    pub fn from_string(
        input: &str,
        current_config_path: Option<&ReadablePath>,
    ) -> Result<ReadablePath, Error> {
        // Case: config:<path>
        if let Some(config_file_path) = current_config_path
            && input.starts_with("config:")
        {
            let input = input.replacen("config:", "", 1);
            return Ok(ReadablePath::from_url(
                config_file_path
                    .as_ref()
                    .join(input.as_str())
                    .map_err(|_e| Error::InvalidUrl)?,
            ));
        }

        // Case file / http(s) / py
        if let Ok(url) = Url::parse(input) {
            if url.scheme() == "py" {
                return py_url_to_url(url).map(ReadablePath::from_url);
            }
            return Ok(ReadablePath::from_url(url));
        }

        // case: absolute dir or relative to cwd /home dir
        Ok(ReadablePath::from_url(
            Url::from_file_path(WritablePath::from_string(input)?.as_ref())
                .map_err(|_| Error::InvalidUrl)?,
        ))
    }

    pub fn join(&self, file: &str) -> ReadablePath {
        ReadablePath::from_url(self.as_ref().join(file).unwrap())
    }
}

impl ReadPath for ReadablePath {
    fn copy(&self, dest: &WritablePath) -> Result<(), PathError> {
        // TODO: create parent dir if needed

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

    fn read_to_bytes(&self) -> Result<Vec<u8>, PathError> {
        match self.as_ref().scheme() {
            "file" => Ok(std::fs::read(
                self.as_ref()
                    .to_file_path()
                    .expect("an url with a file scheme is a valid file path"),
            )?),
            "http" | "https" => Ok(reqwest::blocking::get(self.as_ref().clone())?
                .bytes()?
                .into()),
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
        // case: relative to home dir
        if input.starts_with("~") {
            if let Some(home) = dirs::home_dir() {
                let expanded = input.replacen("~", home.to_str().unwrap(), 1);
                return Ok(WritablePath::new(PathBuf::from(expanded)));
            }
            return Err(Error::InvalidUrl);
        }

        // case: absolute
        if input.starts_with("/") {
            return Ok(WritablePath::new(PathBuf::from(input)));
        }

        // case: relative to cwd
        let cwd = std::env::current_dir()
            .map_err(|e| e.to_string())
            .map_err(|_| Error::InvalidUrl)?;
        let full_path = cwd.join(input);
        Ok(WritablePath::new(full_path))
    }

    pub fn write_from_string(&self, content: &str) -> Result<(), Error> {
        Ok(std::fs::write(self.as_ref(), content)?)
    }

    pub fn exists(&self) -> bool {
        self.as_ref().exists()
    }
}

impl std::fmt::Display for WritablePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_ref().to_string_lossy())
    }
}

impl ReadPath for WritablePath {
    fn read_to_bytes(&self) -> Result<Vec<u8>, PathError> {
        Ok(std::fs::read(self.as_ref())?)
    }

    fn exists(&self) -> Result<bool, PathError> {
        Ok(self.as_ref().exists())
    }

    fn copy(&self, dest: &WritablePath) -> Result<(), PathError> {
        std::fs::copy(self.as_ref(), dest.as_ref())?;
        Ok(())
    }
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
    fn test_config_readable_path() {
        let path = ReadablePath::from_string(
            "config:test.toml",
            Some(&ReadablePath::from_url(
                Url::from_file_path("/some/base/dir/config.toml").unwrap(),
            )),
        )
        .expect("path is ok");

        assert_eq!(path.as_ref().path(), "/some/base/dir/test.toml");
    }

    #[test]
    fn test_config_readable_path_with_url_base() {
        let path = ReadablePath::from_string(
            "config:test.toml",
            Some(
                &ReadablePath::from_string("https://test.nl/some/base/dir/config.toml", None)
                    .unwrap(),
            ),
        )
        .expect("path is ok");

        assert_eq!(path.as_ref().path(), "/some/base/dir/test.toml");

        let path = ReadablePath::from_string(
            "config:sub/test.toml",
            Some(
                &ReadablePath::from_string("https://test.nl/some/base/dir/config.toml", None)
                    .unwrap(),
            ),
        )
        .expect("path is ok");

        assert_eq!(path.as_ref().path(), "/some/base/dir/sub/test.toml");
    }

    #[test]
    fn test_paths_copy_and_read() {
        let dir = tempdir().unwrap();
        let destination = dir.path().join("destination");
        let destination = WritablePath::new(destination);

        let source = ReadablePath::from_string(
            "https://rust-lang.org/static/images/rust-logo-blk.svg",
            Some(&ReadablePath::from_url(
                Url::from_file_path(dir.path()).expect("valid path"),
            )),
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
                Some(&ReadablePath::from_url(
                    Url::from_file_path(dir.path()).expect("valid path")
                )),
            )
            .unwrap()
            .exists()
            .unwrap()
        );

        assert!(
            !ReadablePath::from_string(
                "https://rust-lang.org/non_existing",
                Some(&ReadablePath::from_url(
                    Url::from_file_path(dir.path()).expect("valid path")
                )),
            )
            .unwrap()
            .exists()
            .unwrap()
        );

        let tmp_path = dir.path().join("tmp_file");

        assert!(!WritablePath::new(tmp_path.clone()).exists());

        std::fs::File::create(&tmp_path).unwrap();

        assert!(WritablePath::new(tmp_path).exists());
    }

    #[test]
    fn test_uris() {
        assert_eq!(
            ReadablePath::from_string("file:///path/to/test", None)
                .unwrap()
                .as_ref()
                .path(),
            "/path/to/test"
        );
        assert_eq!(
            ReadablePath::from_string("https://path/to/test", None)
                .unwrap()
                .as_ref()
                .path(),
            "/to/test"
        );
        assert_eq!(
            ReadablePath::from_string("py://pathlib/to/test", None)
                .unwrap()
                .as_ref()
                .path(),
            "/path/to/python/lib/site-packages/to/test"
        );
        assert!(
            ReadablePath::from_string("pathlib", None)
                .unwrap()
                .as_ref()
                .path()
                .ends_with("check-config/pathlib")
        );
        assert_eq!(
            ReadablePath::from_string("/path/to/test", None)
                .unwrap()
                .as_ref()
                .path(),
            "/path/to/test"
        );

        assert_eq!(
            ReadablePath::from_string(
                "https://domain/to/test",
                Some(&ReadablePath::from_string("https://domain/other/path", None).unwrap())
            )
            .unwrap()
            .as_ref()
            .path(),
            "/to/test"
        );

        assert_eq!(
            ReadablePath::from_string(
                "config:test",
                Some(&ReadablePath::from_string("https://domain/other/path", None).unwrap())
            )
            .unwrap()
            .as_ref()
            .path(),
            "/other/test"
        );

        assert_eq!(
            ReadablePath::from_string(
                "config:test",
                Some(&ReadablePath::from_string("https://domain/other/path/", None).unwrap())
            )
            .unwrap()
            .as_ref()
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
