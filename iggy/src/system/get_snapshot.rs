use crate::validatable::Validatable;
use crate::{bytes_serializable::BytesSerializable, error::Error};
use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::command::CommandPayload;

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct GetSnapshot {}

impl CommandPayload for GetSnapshot {}

impl Validatable<Error> for GetSnapshot {
    fn validate(&self) -> Result<(), Error> {
        Ok(())
    }
}

impl FromStr for GetSnapshot {
    type Err = Error;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        if !input.is_empty() {
            return Err(Error::InvalidCommand);
        }

        let command = GetSnapshot {};
        command.validate()?;
        Ok(GetSnapshot {})
    }
}

impl BytesSerializable for GetSnapshot {
    fn as_bytes(&self) -> Vec<u8> {
        Vec::with_capacity(0)
    }

    fn from_bytes(bytes: &[u8]) -> Result<GetSnapshot, Error> {
        if !bytes.is_empty() {
            return Err(Error::InvalidCommand);
        }

        let command = GetSnapshot {};
        command.validate()?;
        Ok(GetSnapshot {})
    }
}

impl Display for GetSnapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "")
    }
}
