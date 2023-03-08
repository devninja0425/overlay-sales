use concordium_cis2::{TokenAmountU64, TokenIdUnit};
use concordium_std::{SchemaType, Serialize};

pub type ContractTokenId = TokenIdUnit;
pub type ContractTokenAmount = TokenAmountU64;
pub type OvlCreditAmount = u64;
pub type MicroCcd = u64;
pub type UnitsAmount = u32;

pub type AllowedPercentage = u8;
pub type UsdcAmount = TokenAmountU64;
pub type MicroUsdc = u64;

#[derive(Debug, Serialize, SchemaType, Clone, PartialEq, Eq, PartialOrd)]
pub enum SaleStatus {
    Prepare,
    Ready,
    Fixed,
    Suspend,
}

#[derive(Debug, Serialize, SchemaType, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum Tier {
    T0 = 0,
    T1,
    T2,
    T3,
    T4,
    T5,
}

impl From<u8> for Tier {
    fn from(n: u8) -> Self {
        match n {
            n if n == Tier::T1 as u8 => Tier::T1,
            n if n == Tier::T2 as u8 => Tier::T2,
            n if n == Tier::T3 as u8 => Tier::T3,
            n if n == Tier::T4 as u8 => Tier::T4,
            n if n == Tier::T5 as u8 => Tier::T5,
            _ => Tier::T0,
        }
    }
}

impl From<&str> for Tier {
    fn from(n: &str) -> Self {
        match n {
            "t1" => Tier::T1,
            "t2" => Tier::T2,
            "t3" => Tier::T3,
            "t4" => Tier::T4,
            "t5" => Tier::T5,
            _ => Tier::T0,
        }
    }
}

#[derive(Debug, Serialize, SchemaType, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum Prior {
    TOP = 1,
    SECOND,
    NONE = 99,
}
