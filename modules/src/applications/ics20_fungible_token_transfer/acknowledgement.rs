use alloc::string::String;
use core::fmt::{Display, Formatter};
use core::write;
use serde::{Deserialize, Serialize};

/// Ics20 Acknowledgement
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ICS20Acknowledgement {
    /// Equivalent to b"AQ=="
    Success,
    /// Error Acknowledgement
    Error(String),
}

impl Display for ICS20Acknowledgement {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let raw_string = match self {
            Self::Success => "AQ==",
            Self::Error(err) => err.as_str(),
        };
        write!(f, "{}", raw_string)
    }
}
