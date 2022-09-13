use crate::prelude::*;
use core::fmt;
use core::fmt::Debug;

use serde_derive::{Deserialize, Serialize};

use super::error::Error;

/// Type of the client, depending on the specific consensus algorithm.
#[derive(
    Copy,
    Clone,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    derive::ClientType,
)]
pub enum ClientType {
    Tendermint = 7,
    #[cfg(any(test, feature = "ics11_beefy"))]
    Beefy = 11,
    #[cfg(any(test, feature = "ics11_beefy"))]
    Near = 13,
    #[cfg(any(test, feature = "mocks"))]
    Mock = 9999,
}

impl fmt::Display for ClientType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ClientType({})", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use core::str::FromStr;
    use test_log::test;

    use super::ClientType;
    use crate::core::ics02_client::error::{Error, ErrorDetail};

    #[test]
    fn parse_tendermint_client_type() {
        let client_type = ClientType::from_str("7-tendermint");

        match client_type {
            Ok(ClientType::Tendermint) => (),
            _ => panic!("parse failed"),
        }
    }

    #[test]
    fn parse_mock_client_type() {
        let client_type = ClientType::from_str("9999-mock");

        match client_type {
            Ok(ClientType::Mock) => (),
            _ => panic!("parse failed"),
        }
    }

    #[test]
    fn parse_unknown_client_type() {
        let client_type_str = "some-random-client-type";
        let result = ClientType::from_str(client_type_str);

        match result {
            Err(Error(ErrorDetail::UnknownClientType(e), _)) => {
                assert_eq!(&e.client_type, client_type_str)
            }
            _ => {
                panic!("Expected ClientType::from_str to fail with UnknownClientType, instead got",)
            }
        }
    }

    #[test]
    fn parse_mock_as_string_result() {
        let client_type = ClientType::Mock;
        let type_string = client_type.as_str();
        let client_type_from_str = ClientType::from_str(type_string).unwrap();
        assert_eq!(client_type_from_str, client_type);
    }

    #[test]
    fn parse_tendermint_as_string_result() {
        let client_type = ClientType::Tendermint;
        let type_string = client_type.as_str();
        let client_type_from_str = ClientType::from_str(type_string).unwrap();
        assert_eq!(client_type_from_str, client_type);
    }
}
