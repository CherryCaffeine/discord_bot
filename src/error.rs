#[derive(Debug, thiserror::Error)]
pub(crate) enum Error {
    #[error("Sqlx error: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("Serenity error: {0}")]
    Serenity(#[from] serenity::Error),
}

pub(crate) type Result<T> = core::result::Result<T, Error>;
