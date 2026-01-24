use crate::ConversionArg;
use crate::dashboard::DisplayAmount;
use crate::nns_types::NeuronId;
use candid::{CandidType, Decode, Deserialize};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString};

// Maximum number of bytes for a WaterNeuron call argument passed to the ICRC-21 endpoint.
pub const MAX_CONSENT_MESSAGE_ARG_SIZE_BYTES: u16 = 500;

#[derive(Debug, EnumString, EnumIter, Display)]
pub enum Icrc21Function {
    #[strum(serialize = "icp_to_nicp")]
    Stake,
    #[strum(serialize = "nicp_to_icp")]
    Unstake,
    #[strum(serialize = "cancel_withdrawal")]
    CancelWithdrawal,
    #[strum(serialize = "claim_airdrop")]
    ClaimAirdrop,
}

#[derive(CandidType, Deserialize)]
pub struct StandardRecord {
    pub url: String,
    pub name: String,
}

#[derive(CandidType, Deserialize)]
pub struct ConsentMessageRequest {
    pub arg: Vec<u8>,
    pub method: String,
    pub user_preferences: ConsentMessageSpec,
}

#[derive(CandidType, Deserialize)]
pub struct ConsentMessageSpec {
    pub metadata: ConsentMessageMetadata,
    pub device_spec: Option<DisplayMessageType>,
}

#[derive(CandidType, Deserialize)]
pub struct ConsentMessageMetadata {
    pub utc_offset_minutes: Option<u16>,
    pub language: String,
}

#[derive(CandidType, Deserialize)]
pub enum DisplayMessageType {
    GenericDisplay,
    LineDisplay {
        characters_per_line: u16,
        lines_per_page: u16,
    },
}

#[derive(CandidType, Deserialize)]
pub struct ConsentInfo {
    pub metadata: ConsentMessageMetadata,
    pub consent_message: ConsentMessage,
}

#[derive(CandidType, Deserialize)]
pub enum ConsentMessage {
    LineDisplayMessage { pages: Vec<LineDisplayPage> },
    GenericDisplayMessage(String),
}

#[derive(CandidType, Deserialize)]
pub struct LineDisplayPage {
    lines: Vec<String>,
}

#[derive(CandidType, Deserialize)]
pub enum Icrc21Error {
    GenericError {
        description: String,
        error_code: u64,
    },
    InsufficientPayment(ErrorInfo),
    UnsupportedCanisterCall(ErrorInfo),
    ConsentMessageUnavailable(ErrorInfo),
}

#[derive(CandidType, Deserialize)]
pub struct ErrorInfo {
    pub description: String,
}

pub fn icrc10_supported_standards() -> Vec<StandardRecord> {
    vec![
        StandardRecord {
            name: "ICRC-21".to_string(),
            url: "https://github.com/dfinity/wg-identity-authentication/blob/main/topics/ICRC-21/icrc_21_consent_msg.md".to_string(),
        }
    ]
}

pub fn icrc21_canister_call_consent_message(
    request: ConsentMessageRequest,
) -> Result<ConsentInfo, Icrc21Error> {
    if request.arg.len() > MAX_CONSENT_MESSAGE_ARG_SIZE_BYTES as usize {
        return Err(Icrc21Error::UnsupportedCanisterCall(ErrorInfo {
            description: format!(
                "The argument size is too large. The maximum allowed size is {} bytes.",
                MAX_CONSENT_MESSAGE_ARG_SIZE_BYTES
            ),
        }));
    }

    let metadata = ConsentMessageMetadata {
        language: "en".to_string(),
        utc_offset_minutes: request.user_preferences.metadata.utc_offset_minutes,
    };

    let message = match request.method.parse::<Icrc21Function>().map_err(|err| {
        Icrc21Error::UnsupportedCanisterCall(ErrorInfo {
            description: format!(
                "The call provided is not supported: {}.\n Supported calls for ICRC-21 are {:?}. \n Error: {:?}", 
                request.method,
                Icrc21Function::iter().map(|f|f.to_string()).collect::<Vec<String>>(),
                err
            ),
        })
    })? {
        Icrc21Function::Stake =>  {
            let arg = Decode!(&request.arg, ConversionArg).map_err(|e| Icrc21Error::UnsupportedCanisterCall(ErrorInfo {
                description: format!("Failed to decode ConversionArg: {}", e)
            }))?;
            match arg.maybe_subaccount {
                Some(subaccount) => format!("Convert {} ICP to nICP at the current exchange rate. 
                    Specified subaccount: {}.", 
                    DisplayAmount(arg.amount_e8s),
                    hex::encode(subaccount)
                ),
                None => format!("Convert {} ICP to nICP at the current exchange rate.", DisplayAmount(arg.amount_e8s))
            }
        },
        Icrc21Function::Unstake =>  {
            let arg = Decode!(&request.arg, ConversionArg).map_err(|e| Icrc21Error::UnsupportedCanisterCall(ErrorInfo {
                description: format!("Failed to decode ConversionArg: {}", e),
            }))?;
            match arg.maybe_subaccount {
                Some(subaccount) => format!(
                    "Convert {} nICP to ICP at the current exchange rate after a 6 months dissolve delay. 
                    Specified subaccount: {}.", 
                    DisplayAmount(arg.amount_e8s),
                    hex::encode(subaccount)
                ),
                None => format!(
                    "Convert {} nICP to ICP at the current exchange rate after a 6 months dissolve delay.",
                    DisplayAmount(arg.amount_e8s)
                )
            }
        },
        Icrc21Function::CancelWithdrawal =>  {
            let arg = Decode!(&request.arg, NeuronId).map_err(|e| Icrc21Error::UnsupportedCanisterCall(ErrorInfo {
                description: format!("Failed to decode NeuronId: {}", e),
            }))?;
            format!("Cancel the withdrawal associated to the neuron id: {}.", arg.id)
        },
        Icrc21Function::ClaimAirdrop =>  {
            "Claim WTN tokens associated to your airdrop allocation.".to_string()
        }
    };

    Ok(ConsentInfo {
        metadata,
        consent_message: ConsentMessage::GenericDisplayMessage(message),
    })
}
