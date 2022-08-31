use flex_error::define_error;
use near_lite_client::CryptoHash;

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
    }
}
