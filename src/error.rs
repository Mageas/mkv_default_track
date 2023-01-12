use thiserror::Error;

#[derive(Error, Debug)]
pub enum TempError {
    #[error("Unable to deserialize")]
    Deserialize(#[source] serde_json::Error),

    #[error("Unable to serialize")]
    Serialize(#[source] serde_json::Error),
}

pub type TempResult<T = ()> = Result<T, TempError>;
