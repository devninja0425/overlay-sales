use concordium_cis2::Cis2Error;
use concordium_std::{
    num, CallContractError, ParseError, Reject, SchemaType, Serialize, UnwrapAbort, UpgradeError,
};
use core::num::TryFromIntError;

pub type ContractResult<A> = Result<A, ContractError>;

pub type ContractError = Cis2Error<CustomContractError>;

/// The different errors the contract can produce.
#[derive(Serialize, Debug, PartialEq, Eq, Reject, SchemaType)]
pub enum CustomContractError {
    #[from(ParseError)]
    ParseParams, //1
    OverflowError,                         //
    InvokeContractError,                   //
    FailedUpgradeMissingModule,            //
    FailedUpgradeMissingContract,          //5
    FailedUpgradeUnsupportedModuleVersion, //
    AmountTooLarge,                        //
    MissingAccount,                        //
    MissingContract,                       //
    MissingEntrypoint,                     //10
    MessageFailed,                         //
    Trap,                                  //
    TransferError,                         //
    ContractPaused,                        //
    ContractOnly,                          //15
    AccountOnly,                           //
    AlreadySaleStarted,                    //
    AlreadySaleClosed,                     //
    AlreadyDeposited,                      //
    AlreadyRefunded,                       //20
    NotDeposited,                          //
    SaleNotReady,                          //
    NotMatchAmount,                        //
    InvalidSchedule,                       //
    InvalidInput,                          //25
    Inappropriate,                         //
    ColdPeriod,                            //
    DisabledForNow,                        //
}

impl From<CustomContractError> for ContractError {
    fn from(c: CustomContractError) -> Self {
        Cis2Error::Custom(c)
    }
}

impl<T> From<CallContractError<T>> for CustomContractError {
    fn from(cce: CallContractError<T>) -> Self {
        match cce {
            CallContractError::AmountTooLarge => Self::AmountTooLarge,
            CallContractError::MissingAccount => Self::MissingAccount,
            CallContractError::MissingContract => Self::MissingContract,
            CallContractError::MissingEntrypoint => Self::MissingEntrypoint,
            CallContractError::MessageFailed => Self::MessageFailed,
            CallContractError::Trap => Self::Trap,
            CallContractError::LogicReject {
                reason: _,
                return_value: _,
            } => Self::InvokeContractError,
        }
    }
}

impl From<UpgradeError> for CustomContractError {
    #[inline(always)]
    fn from(ue: UpgradeError) -> Self {
        match ue {
            UpgradeError::MissingModule => Self::FailedUpgradeMissingModule,
            UpgradeError::MissingContract => Self::FailedUpgradeMissingContract,
            UpgradeError::UnsupportedModuleVersion => Self::FailedUpgradeUnsupportedModuleVersion,
        }
    }
}

impl From<TryFromIntError> for CustomContractError {
    #[inline(always)]
    fn from(_: TryFromIntError) -> Self {
        Self::OverflowError
    }
}
