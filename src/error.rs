#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("TryFrom failed: {0}")]
    TryFrom(String),
}
