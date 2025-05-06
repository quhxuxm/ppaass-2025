use thiserror::Error;
#[derive(Error, Debug)]
pub enum UserError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
