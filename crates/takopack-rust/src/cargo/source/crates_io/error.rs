use takopack_core::error::Error as CoreError;

#[derive(Debug)]
pub enum Error {
    CoreFunc(CoreError),
    VersionNotFound { name: String, version: String },
    CrateNotFound { name: String },
    VersionNotSet,
    ChecksumMismatch { got: String, expected: String },
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::CoreFunc(error) => error.fmt(f),
            Error::VersionNotFound { name, version } => write!(f, "{}-{} not found", name, version),
            Error::CrateNotFound { name } => write!(f, "crate {} not found", name),
            Error::VersionNotSet => write!(f, "version not set"),
            Error::ChecksumMismatch { got, expected } => {
                write!(f, "checksum mismatch, expected {}, got {}", expected, got)
            }
        }
    }
}

impl From<takopack_core::error::Error> for Error {
    fn from(value: takopack_core::error::Error) -> Self {
        Self::CoreFunc(value)
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::CoreFunc(value.into())
    }
}

impl From<crate::cargo::source::Error> for Error {
    fn from(value: crate::cargo::source::Error) -> Self {
        match value {
            crate::cargo::source::Error::Io(error) => error.into(),
            crate::cargo::source::Error::ChecksumMismatch { got, expected } => {
                Self::ChecksumMismatch { got, expected }
            }
        }
    }
}
