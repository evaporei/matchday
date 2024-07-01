#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Request(#[from] reqwest::Error),

    #[error("IO error: {0}")]
    Io(#[from] IOError),

    #[error(transparent)]
    EnvVar(#[from] EnvVarError),

    #[error("JSON serialization error: {0}")]
    JSON(#[from] JSONError),
}

#[derive(thiserror::Error, Debug)]
#[error("{source} ({file})")]
pub struct IOError {
    file: std::path::PathBuf,
    #[source]
    source: std::io::Error,
}

impl IOError {
    pub fn new(file: std::path::PathBuf, source: std::io::Error) -> Self {
        Self {
            file: file.into(),
            source,
        }
    }
}

#[derive(thiserror::Error, Debug)]
#[error("{source} ({var})")]
pub struct EnvVarError {
    var: String,
    #[source]
    source: std::env::VarError,
}

impl EnvVarError {
    pub fn new(var: &str, source: std::env::VarError) -> Self {
        Self {
            var: var.into(),
            source,
        }
    }
}

#[derive(thiserror::Error, Debug)]
#[error("{source} {}", match file { Some(f) => f.clone().into_os_string().into_string().unwrap(), None => "".into()})]
pub struct JSONError {
    file: Option<std::path::PathBuf>,
    #[source]
    source: serde_json::Error,
}

impl JSONError {
    pub fn new(file: Option<std::path::PathBuf>, source: serde_json::Error) -> Self {
        Self {
            file: file.map(|s| s.into()),
            source,
        }
    }
}
