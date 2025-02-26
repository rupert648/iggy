use crate::bytes_serializable::BytesSerializable;
use crate::command::CommandPayload;
use crate::error::Error;
use crate::identifier::Identifier;
use crate::validatable::Validatable;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct GetStream {
    #[serde(skip)]
    pub stream_id: Identifier,
}

impl CommandPayload for GetStream {}

impl Validatable<Error> for GetStream {
    fn validate(&self) -> Result<(), Error> {
        Ok(())
    }
}

impl FromStr for GetStream {
    type Err = Error;
    fn from_str(input: &str) -> std::result::Result<Self, Self::Err> {
        let parts = input.split('|').collect::<Vec<&str>>();
        if parts.len() != 1 {
            return Err(Error::InvalidCommand);
        }

        let stream_id = parts[0].parse::<Identifier>()?;
        let command = GetStream { stream_id };
        command.validate()?;
        Ok(command)
    }
}

impl BytesSerializable for GetStream {
    fn as_bytes(&self) -> Vec<u8> {
        self.stream_id.as_bytes()
    }

    fn from_bytes(bytes: &[u8]) -> std::result::Result<GetStream, Error> {
        if bytes.len() < 3 {
            return Err(Error::InvalidCommand);
        }

        let stream_id = Identifier::from_bytes(bytes)?;
        let command = GetStream { stream_id };
        command.validate()?;
        Ok(command)
    }
}

impl Display for GetStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.stream_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_be_serialized_as_bytes() {
        let command = GetStream {
            stream_id: Identifier::numeric(1).unwrap(),
        };

        let bytes = command.as_bytes();
        let stream_id = Identifier::from_bytes(&bytes).unwrap();

        assert!(!bytes.is_empty());
        assert_eq!(stream_id, command.stream_id);
    }

    #[test]
    fn should_be_deserialized_from_bytes() {
        let stream_id = Identifier::numeric(1).unwrap();
        let bytes = stream_id.as_bytes();
        let command = GetStream::from_bytes(&bytes);
        assert!(command.is_ok());

        let command = command.unwrap();
        assert_eq!(command.stream_id, stream_id);
    }

    #[test]
    fn should_be_read_from_string() {
        let stream_id = Identifier::numeric(1).unwrap();
        let input = stream_id.to_string();
        let command = GetStream::from_str(&input);
        assert!(command.is_ok());

        let command = command.unwrap();
        assert_eq!(command.stream_id, stream_id);
    }
}
