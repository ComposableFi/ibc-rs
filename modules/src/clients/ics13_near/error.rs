use flex_error::{define_error, TraceError};
use near_lite_client::CryptoHash;
use near_primitives_wasm::ConversionError;
/*
       InvalidChainIdentifier
           [ ValidationError ]
           |_| { "invalid chain identifier" },
        InvalidChainId
            { raw_value: String }
            [ ValidationError ]
            |e| { format_args!("invalid chain identifier: {}", e.raw_value) },

*/
define_error! {
    #[derive(Debug, PartialEq, Eq)]
    Error {
        InvalidEpoch
        { epoch_id: CryptoHash }
        | _ | { "invalid epoch id" },
        HeightTooOld
        | _ | { format_args!(
            "height too old")
        },
        InvalidSignature
        | _ | { format_args!(
            "invalid signature")
        },
        InsufficientStakedAmount
        | _ | { format_args!(
            "insufficient staked amount")
        },
        SerializationError
        | _ | { format_args!(
            "serialization error")
        },
        UnavailableBlockProducers
        | _ | { format_args!(
            "unavailable block producers")
        },
        LiteClientError
        | _ | { format_args!(
            "lite client error"
        )},
        InvalidTimestamp
            | _ | { "invalid timestamp" },
        InvalidCommitmentRoot
            |_| { "invalid commitment root" },
        InvalidRawClientState
            { reason: String }
            |e| { format_args!("invalid raw client state: {}", e.reason) },
        MissingFrozenHeight
            |_| { "missing frozen height" },
        InvalidRawHeight
            { raw_height: u64 }
            |e| { format_args!("invalid raw height: {}", e.raw_height) },
        InvalidRawConsensusState
            { reason: String }
            | e | { format_args!("invalid raw client consensus state: {}", e.reason) },
        InvalidRawHeader
            | _ | { "invalid raw header" },
        InvalidRawMisbehaviour
            { reason: String }
            | e | { format_args!("invalid raw misbehaviour: {}", e.reason) },
        ConversionError
            { reason: String }
            | e | { format_args!("type conversion error: {}", e.reason) },
       Decode
            [ TraceError<prost::DecodeError> ]
            | _ | { "decode error" },
    }
}

impl From<ConversionError> for Error {
    fn from(e: ConversionError) -> Self {
        Error::conversion_error(e.to_string())
    }
}
