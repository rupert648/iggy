use std::fmt::{Display, Formatter, Result};
use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum CmdToolError {
    MissingCredentials,
}

impl Display for CmdToolError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::MissingCredentials => {
                write!(f, "Missing iggy server credentials")
            }
        }
    }
}

#[derive(Error, Debug)]
pub(crate) enum IggyCmdError {
    #[error("Iggy client error")]
    IggyClient(#[from] iggy::client_error::ClientError),

    #[error("Iggy sdk or command error")]
    CommandError(#[from] anyhow::Error),

    #[error("Iggy password prompt error")]
    PasswordPrompt(#[from] passterm::PromptError),

    #[error("Iggy command line tool error")]
    CmdToolError(#[from] CmdToolError),
}
