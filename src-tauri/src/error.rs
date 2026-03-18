use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("ADB error: {0}")]
    Adb(String),

    #[error("AirPlay error: {0}")]
    AirPlay(String),

    #[error("Video pipeline error: {0}")]
    Video(String),

    #[error("Device not found: {0}")]
    DeviceNotFound(String),

    #[error("Connection error: {0}")]
    Connection(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Other(String),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;
