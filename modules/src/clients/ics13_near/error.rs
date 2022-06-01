use super::types::CryptoHash;
use flex_error::define_error;

define_error! {
    #[derive(Debug, PartialEq, Eq)]
    Error {
        InvalidEpoch
        { epoch_id: CryptoHash }
        | _ | { "invalid epoch id" },
        InvalidRawConsensusState
            { reason: String }
            | e | { format_args!("invalid raw client consensus state: {}", e.reason) },
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
    }
}
