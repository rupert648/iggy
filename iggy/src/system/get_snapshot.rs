use crate::validatable::Validatable;
use crate::{bytes_serializable::BytesSerializable, error::Error};
use serde::{Deserialize, Serialize};
use std::str;
use std::{fmt::Display, str::FromStr};

use crate::command::CommandPayload;

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct GetSnapshot {
    pub file_save_location: String,
}

impl CommandPayload for GetSnapshot {}

impl Validatable<Error> for GetSnapshot {
    fn validate(&self) -> Result<(), Error> {
        // TODO: check if valid path?
        Ok(())
    }
}

impl FromStr for GetSnapshot {
    type Err = Error;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let parts = input.split('|').collect::<Vec<&str>>();
        if parts.len() != 1 {
            return Err(Error::InvalidCommand);
        }

        let file_save_location = parts[0].to_owned();

        let command = GetSnapshot { file_save_location };
        command.validate()?;
        Ok(command)
    }
}

impl BytesSerializable for GetSnapshot {
    fn as_bytes(&self) -> Vec<u8> {
        self.file_save_location.clone().into_bytes()
    }

    fn from_bytes(bytes: &[u8]) -> Result<GetSnapshot, Error> {
        let file_save_location = str::from_utf8(bytes)?.to_owned();
        let command = GetSnapshot { file_save_location };

        command.validate()?;
        Ok(command)
    }
}

impl Display for GetSnapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.file_save_location)
    }
}
