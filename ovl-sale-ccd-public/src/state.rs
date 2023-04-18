use collections::BTreeMap;
use concordium_std::{SchemaType, Serialize, *};
pub use sale_utils::{
    error::{ContractError, ContractResult, CustomContractError},
    types::*,
};

/// All participants can purchase only 1 unit.
pub const TARGET_UNITS: u8 = 1;

/// The contract state
#[derive(Debug, Serial, DeserialWithState, StateClone)]
#[concordium(state_parameter = "S")]
pub struct State<S: HasStateApi> {
    /// Account of the administrator of the entity running the IDO
    pub(crate) proj_admin: AccountAddress,
    /// Enum for sale status
    pub(crate) status: SaleStatus,
    /// If `true`, some functions will stop working
    pub(crate) paused: bool,
    /// Address of Overlay for receiving sale fee
    pub(crate) addr_ovl: Address,
    /// Address of Overlay for buy back burn
    pub(crate) addr_bbb: Address,
    /// Number of how many fee received
    pub(crate) ovl_claimed_inc: u8,
    /// Number of how many fee for BBB received
    pub(crate) bbb_claimed_inc: u8,
    /// Project token contract address for RIDO
    pub(crate) project_token: Option<ContractAddress>,
    /// Sale schedule
    pub(crate) schedule: SaleSchedule,
    /// Information about sale
    pub(crate) saleinfo: SaleInfo,
    /// Sale participants
    pub(crate) participants: StateMap<Address, UserState, S>,
}

impl<S: HasStateApi> State<S> {
    pub(crate) fn new(
        state_builder: &mut StateBuilder<S>,
        proj_admin: AccountAddress,
        addr_ovl: Address,
        addr_bbb: Address,
        schedule: SaleSchedule,
        saleinfo: SaleInfo,
    ) -> Self {
        State {
            proj_admin,
            paused: false,
            status: SaleStatus::Prepare,
            addr_ovl,
            addr_bbb,
            ovl_claimed_inc: 0,
            bbb_claimed_inc: 0,
            project_token: None,
            schedule,
            saleinfo,
            participants: state_builder.new_map(),
        }
    }

    // TODO should we remove &mut (should not be self mutable function)
    pub(crate) fn calc_vesting_amount(
        &mut self,
        now: Timestamp,
        vesting_start: Timestamp,
        total_units: u64,
        shared: u8,
        cur_inc: u8,
    ) -> ContractResult<(ContractTokenAmount, u8)> {
        let mut amount: u128 = 0;
        let mut inc: u8 = 0;

        for (duration, per) in self.schedule.vesting_period.iter() {
            let ts = match vesting_start.checked_add(*duration) {
                Some(v) => v,
                None => bail!(CustomContractError::InvalidSchedule.into()),
            };

            if now < ts {
                break;
            }

            if cur_inc > inc {
                inc += 1;
                continue;
            }

            let total_amount: u128 = (self.saleinfo.token_per_unit.0 as u128)
                .checked_mul(u128::from(total_units))
                .ok_or(ContractError::from(CustomContractError::OverflowError))?;

            let total_claimable: u128 = total_amount
                .checked_mul(u128::from(shared))
                .ok_or(ContractError::from(CustomContractError::OverflowError))?
                / 100;

            let allocation: u128 = total_claimable
                .checked_mul(u128::from(*per as u128))
                .ok_or(ContractError::from(CustomContractError::OverflowError))?
                / 100;

            amount += allocation;
            inc += 1;
        }

        if amount > u64::MAX as u128 {
            bail!(ContractError::from(CustomContractError::OverflowError))
        } else {
            Ok((ContractTokenAmount::from(amount as u64), inc))
        }
    }

    pub(crate) fn whitelist(&mut self, user: &Address, prior: Prior) {
        // [#Caution] if the user exists, the state is overwritten.
        let _ = self
            .participants
            .entry(*user)
            .or_insert_with(|| UserState::new(prior, Amount::zero(), TARGET_UNITS));
    }

    pub(crate) fn get_user_any(&mut self, user: &Address) -> ContractResult<UserState> {
        let user = self
            .participants
            .entry(*user)
            .or_insert_with(|| UserState::new(Prior::ANY, Amount::zero(), TARGET_UNITS));
        let user = user.get_ref();
        Ok(user.clone())
    }

    // fn modify_whitelist(
    //     &mut self,
    //     user: &Address,
    //     prior: Prior,
    //     tgt_units: u8,
    // ) -> ContractResult<()> {
    //     let mut user = self
    //         .participants
    //         .get_mut(&user)
    //         .ok_or(ContractError::Custom(CustomContractError::InvalidInput))?;
    //     user.prior = prior;
    //     Ok(())
    // }

    #[allow(dead_code)]
    pub(crate) fn check_listed(&mut self, user: &Address) -> bool {
        self.participants.entry(*user).is_occupied()
    }

    // TODO should we remove &mut (should not be self mutable function)
    pub(crate) fn get_user(&mut self, user: &Address) -> ContractResult<UserState> {
        let user = self
            .participants
            .get(user)
            .ok_or(ContractError::Unauthorized)?;
        Ok(user.clone())
    }

    pub(crate) fn deposit(
        &mut self,
        user: &Address,
        amount: Amount,
        win_units: u8,
    ) -> ContractResult<()> {
        let mut user = self
            .participants
            .get_mut(user)
            .ok_or(ContractError::Unauthorized)?;
        user.deposit_ccd = amount;
        user.win_units = win_units;

        self.saleinfo.applied_units += win_units as UnitsAmount;
        Ok(())
    }

    pub(crate) fn increment_user_claimed(&mut self, user: &Address, n: u8) -> ContractResult<()> {
        let mut user = self
            .participants
            .get_mut(user)
            .ok_or(ContractError::Unauthorized)?;
        user.claimed_inc = n;
        Ok(())
    }

    pub(crate) fn remove_participant(&mut self, user: &Address, tgt_units: u8) {
        self.participants.remove(user);
        self.saleinfo.applied_units -= tgt_units as UnitsAmount;
    }
}

#[cfg(any(feature = "wasm-test", test))]
/// implements PartialEq for `claim_eq` inside test functions.
/// this implementation will be build only when `concordium-std/wasm-test` feature is active.
/// (e.g. when launched by `cargo concordium test`)
impl<S: HasStateApi> PartialEq for State<S> {
    fn eq(&self, other: &Self) -> bool {
        if self.proj_admin != other.proj_admin {
            return false;
        }
        if self.status != other.status {
            return false;
        }
        if self.paused != other.paused {
            return false;
        }
        if self.addr_ovl != other.addr_ovl {
            return false;
        }
        if self.addr_bbb != other.addr_bbb {
            return false;
        }
        if self.ovl_claimed_inc != other.ovl_claimed_inc {
            return false;
        }
        if self.bbb_claimed_inc != other.bbb_claimed_inc {
            return false;
        }
        if self.project_token != other.project_token {
            return false;
        }
        if self.schedule != other.schedule {
            return false;
        }
        if self.saleinfo != other.saleinfo {
            return false;
        }
        if self.participants.iter().count() != other.participants.iter().count() {
            return false;
        }
        for (my_user_address, my_user_state) in self.participants.iter() {
            let other_user_state = other.participants.get(&my_user_address);
            if other_user_state.is_none() {
                return false;
            }
            let other_user_state = other_user_state.unwrap();
            if my_user_state.clone() != other_user_state.clone() {
                return false;
            }
        }
        true
    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

/// Sale Schedule
#[derive(Debug, Serialize, SchemaType, Clone)]
#[cfg_attr(any(feature = "wasm-test", test), derive(PartialEq))]
pub struct SaleSchedule {
    /// IDO schedule(The process is split into some phases)
    pub(crate) open_at: BTreeMap<Timestamp, Prior>,
    /// Sale End Time
    pub(crate) close_at: Timestamp,
    /// Actual vesting_period is calculated based on this start time
    pub(crate) vesting_start: Option<Timestamp>,
    /// User(sale particicants) can withdraw assets according to the vesting period
    pub(crate) vesting_period: BTreeMap<Duration, AllowedPercentage>,
}

impl SaleSchedule {
    pub fn new(
        now: Timestamp,
        open_at: BTreeMap<Timestamp, Prior>,
        close_at: Timestamp,
        vesting_period: BTreeMap<Duration, AllowedPercentage>,
    ) -> Result<Self, CustomContractError> {
        ensure!(!open_at.is_empty(), CustomContractError::InvalidSchedule);

        ensure!(
            now < *open_at.first_key_value().unwrap().0,
            CustomContractError::InvalidSchedule
        );

        ensure!(
            *open_at.last_key_value().unwrap().0 < close_at,
            CustomContractError::InvalidSchedule
        );

        // check vesting_period
        let mut total_per = 0;
        for (_, per) in vesting_period.iter() {
            total_per += *per;
        }
        ensure!(total_per == 100, CustomContractError::Inappropriate);

        Ok(SaleSchedule {
            open_at,
            close_at,
            vesting_start: None,
            vesting_period,
        })
    }

    #[allow(dead_code)]
    pub(crate) fn is_sale_opened(&self, now: Timestamp) -> bool {
        if now < *self.open_at.first_key_value().unwrap().0 {
            true
        } else {
            false
        }
    }

    pub(crate) fn is_sale_closed(&self, now: Timestamp) -> bool {
        if self.close_at < now {
            true
        } else {
            false
        }
    }

    pub(crate) fn is_on_sale(&self, now: Timestamp) -> bool {
        if *self.open_at.first_key_value().unwrap().0 <= now && now < self.close_at {
            true
        } else {
            false
        }
    }

    pub(crate) fn check_sale_priority(&self, now: Timestamp) -> Option<Prior> {
        if !self.is_on_sale(now) {
            return None;
        }

        let mut current = Prior::TOP;
        for (ts, priority) in self.open_at.iter() {
            if priority == &current {
                continue;
            }
            if now < *ts {
                return Some(current);
            } else {
                current = priority.clone();
            }
        }
        return Some(current);
    }
}

/// Information about sale
#[derive(Debug, Serialize, SchemaType, Clone)]
#[cfg_attr(any(feature = "wasm-test", test), derive(PartialEq))]
pub struct SaleInfo {
    /// Price in ccd per a project token
    pub(crate) price_per_token: MicroCcd,
    /// Amount of tokens contained in a unit
    pub(crate) token_per_unit: ContractTokenAmount,
    /// Maximum quantity to be issued in this sale
    pub(crate) max_units: UnitsAmount,
    /// Minimum quantity to be issued for this sale
    pub(crate) min_units: UnitsAmount,
    /// Amount of sales completed at that point
    pub(crate) applied_units: UnitsAmount,
}

impl SaleInfo {
    pub(crate) fn new(
        price_per_token: MicroCcd,
        token_per_unit: ContractTokenAmount,
        max_units: UnitsAmount,
        min_units: UnitsAmount,
    ) -> Result<Self, CustomContractError> {
        ensure!(min_units < max_units, CustomContractError::Inappropriate);

        // Price_per_unit must not exceed 18_446_744_073_709_551_615
        if price_per_token.checked_mul(token_per_unit.0).is_none() {
            bail!(CustomContractError::OverflowError);
        }

        Ok(SaleInfo {
            price_per_token,
            token_per_unit,
            max_units,
            min_units,
            applied_units: 0,
        })
    }

    pub(crate) fn check_room_to_apply(&self) -> UnitsAmount {
        if self.applied_units < self.max_units {
            self.max_units - self.applied_units
        } else {
            0
        }
    }

    pub(crate) fn is_reached_sc(&self) -> bool {
        if self.min_units <= self.applied_units {
            true
        } else {
            false
        }
    }

    pub(crate) fn amount_of_pjtoken(&self) -> Result<ContractTokenAmount, CustomContractError> {
        let token_amount = self.token_per_unit.0.checked_mul(self.applied_units as u64);
        if token_amount.is_none() {
            bail!(CustomContractError::OverflowError);
        }
        Ok(ContractTokenAmount::from(token_amount.unwrap()))
    }

    pub(crate) fn calc_price_per_unit(&self) -> Result<Amount, CustomContractError> {
        // Price_per_unit must not exceed 18_446_744_073_709_551_615
        let price = self.price_per_token.checked_mul(self.token_per_unit.0);
        if price.is_none() {
            bail!(CustomContractError::OverflowError);
        }
        let price = price.unwrap();
        Ok(Amount::from_micro_ccd(price))
    }
}

/// About sale participants
#[derive(Debug, Serialize, SchemaType, Clone, PartialEq, Eq)]
pub struct UserState {
    /// Priority to participate in the sale
    pub(crate) prior: Prior,
    /// If deposited, their right to receive tokens will be confirmed.
    pub(crate) deposit_ccd: Amount,
    /// Number of unit desired(or available) to be purchased
    pub(crate) tgt_units: u8,
    /// Number actually determined to be purchased
    pub(crate) win_units: u8,
    /// Number of tokens received during the vesting period(neither Amount or number of claim)
    pub(crate) claimed_inc: u8,
}

impl UserState {
    pub fn new(prior: Prior, deposit_ccd: Amount, tgt_units: u8) -> Self {
        UserState {
            prior,
            deposit_ccd,
            tgt_units,
            win_units: 0,
            claimed_inc: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_infrastructure::*;
    use crate::InitParams;
    #[allow(unused)]
    use sale_utils::{PUBLIC_RIDO_FEE, PUBLIC_RIDO_FEE_BBB, PUBLIC_RIDO_FEE_OVL};

    const PJ_ADMIN_ACC: AccountAddress = AccountAddress([1u8; 32]);
    const ADDR_OVL: Address = Address::Account(AccountAddress([2u8; 32]));
    const ADDR_BBB: Address = Address::Contract(ContractAddress {
        index: 100,
        subindex: 0,
    });
    const USER1_ACC: AccountAddress = AccountAddress([10u8; 32]);
    const USER1_ADDR: Address = Address::Account(USER1_ACC);
    const USER2_ACC: AccountAddress = AccountAddress([11u8; 32]);
    const USER2_ADDR: Address = Address::Account(USER2_ACC);
    const USER3_ACC: AccountAddress = AccountAddress([12u8; 32]);
    const USER3_ADDR: Address = Address::Account(USER3_ACC);

    fn init_parameter(vesting_period: BTreeMap<Duration, AllowedPercentage>) -> InitParams {
        InitParams {
            proj_admin: PJ_ADMIN_ACC,
            addr_ovl: ADDR_OVL,
            addr_bbb: ADDR_BBB,
            open_at: BTreeMap::from([
                (Timestamp::from_timestamp_millis(10), Prior::TOP),
                (Timestamp::from_timestamp_millis(20), Prior::SECOND),
            ]),
            close_at: Timestamp::from_timestamp_millis(30),
            max_units: 100,
            min_units: 50,
            price_per_token: 5_000_000,
            token_per_unit: 200.into(),
            vesting_period: if vesting_period.is_empty() {
                BTreeMap::from([
                    (Duration::from_days(1), 25),
                    (Duration::from_days(2), 40),
                    (Duration::from_days(3), 35),
                ])
            } else {
                vesting_period
            },
        }
    }

    #[test]
    fn test_invalid_schedule() {
        let open_at = BTreeMap::from([
            (Timestamp::from_timestamp_millis(10), Prior::TOP),
            (Timestamp::from_timestamp_millis(20), Prior::SECOND),
        ]);
        let close_at = Timestamp::from_timestamp_millis(30);
        let vesting_period = BTreeMap::from([
            (Duration::from_days(30), 30),
            (Duration::from_days(60), 40),
            (Duration::from_days(90), 30),
        ]);
        let schedule = SaleSchedule::new(
            Timestamp::from_timestamp_millis(15),
            open_at,
            close_at,
            vesting_period,
        );

        // let schedule = match schedule {
        //     Ok(v) => v,
        //     Err(e) => bail!(e.into()),
        // };

        assert_eq!(
            schedule.expect_err_report("error"),
            CustomContractError::InvalidSchedule
        );
    }

    #[test]
    fn test_state() {
        // initialize
        let mut state_builder = TestStateBuilder::new();
        let params = init_parameter(BTreeMap::new());
        let schedule = SaleSchedule::new(
            Timestamp::from_timestamp_millis(1),
            params.open_at,
            params.close_at,
            params.vesting_period,
        )
        .unwrap_abort();

        let saleinfo = SaleInfo::new(
            params.price_per_token,
            params.token_per_unit,
            params.max_units,
            params.min_units,
        )
        .unwrap_abort();
        let mut state = State::new(
            &mut state_builder,
            params.proj_admin,
            params.addr_ovl,
            params.addr_bbb,
            schedule,
            saleinfo,
        );

        state
            .schedule
            .check_sale_priority(Timestamp::from_timestamp_millis(30));

        // whitelisted
        let users = vec![
            (&USER1_ADDR, Prior::TOP),
            (&USER2_ADDR, Prior::SECOND),
            (&USER2_ADDR, Prior::TOP),
        ];
        for v in users.into_iter() {
            state.whitelist(v.0, v.1);
        }

        assert_eq!(
            state.get_user(&USER1_ADDR).unwrap(),
            UserState {
                prior: Prior::TOP,
                deposit_ccd: Amount::zero(),
                tgt_units: 1,
                win_units: 0,
                claimed_inc: 0
            },
            "something wrong with user1 before deposit!"
        );

        // deposit = allocation fixed
        let _ = state.deposit(&USER1_ADDR, Amount::from_ccd(100), 1);

        // vesting
        assert!(state.check_listed(&USER1_ADDR), "user1 should exist!");
        assert!(!state.check_listed(&USER3_ADDR), "user3 should not on list");
        assert_eq!(
            state.get_user(&USER1_ADDR).unwrap(),
            UserState {
                prior: Prior::TOP,
                deposit_ccd: Amount::from_micro_ccd(100_000_000),
                tgt_units: 1,
                win_units: 1,
                claimed_inc: 0
            },
            "something wrong with user1 after deposit!"
        );
    }

    #[test]
    fn test_sale_info_overflow() {
        let price_per_token = 2000_000_000; //2000ccd
        let token_per_unit = 9_300_000_000;
        SaleInfo::new(price_per_token, token_per_unit.into(), 1000, 100)
            .expect_err("should overflow!");
    }

    #[test]
    fn test_divide() {
        let price_per_token: u64 = 2_000_000; //2000ccd
        let token_per_unit: u64 = 900;
        let sale = SaleInfo::new(price_per_token, token_per_unit.into(), 1000, 100).unwrap();
        let price = sale.calc_price_per_unit().unwrap();
        claim_eq!(
            price,
            Amount::from_micro_ccd(price_per_token * token_per_unit),
            "Something wrong with calc price!"
        );
        claim_eq!(
            price,
            Amount::from_ccd(price_per_token / 10_u64.pow(6) * token_per_unit),
            "Something wrong with calc price!"
        );
    }

    #[test]
    fn test_get_user_ary() {
        // initialize
        let mut state_builder = TestStateBuilder::new();
        let params = init_parameter(BTreeMap::new());
        let schedule = SaleSchedule::new(
            Timestamp::from_timestamp_millis(1),
            params.open_at,
            params.close_at,
            params.vesting_period,
        )
        .unwrap_abort();

        let saleinfo = SaleInfo::new(
            params.price_per_token,
            params.token_per_unit,
            params.max_units,
            params.min_units,
        )
        .unwrap_abort();
        let mut state = State::new(
            &mut state_builder,
            params.proj_admin,
            params.addr_ovl,
            params.addr_bbb,
            schedule,
            saleinfo,
        );

        state
            .schedule
            .check_sale_priority(Timestamp::from_timestamp_millis(30));

        // whitelisted
        let users = vec![(&USER1_ADDR, Prior::TOP), (&USER2_ADDR, Prior::SECOND)];
        for v in users.into_iter() {
            state.whitelist(v.0, v.1);
        }

        assert!(state.check_listed(&USER1_ADDR), "user1 should exist!");
        assert!(!state.check_listed(&USER3_ADDR), "user3 should not on list");

        assert_eq!(
            state.get_user(&USER1_ADDR),
            Ok(UserState {
                prior: Prior::TOP,
                deposit_ccd: Amount::zero(),
                tgt_units: 1,
                win_units: 0,
                claimed_inc: 0
            }),
            "something wrong with user1 before deposit!"
        );

        assert_eq!(
            state.get_user(&USER3_ADDR),
            Err(ContractError::Unauthorized),
            "something wrong with user1 after deposit!"
        );

        assert_eq!(
            state.get_user_any(&USER3_ADDR),
            Ok(UserState {
                prior: Prior::ANY,
                deposit_ccd: Amount::zero(),
                tgt_units: 1,
                win_units: 0,
                claimed_inc: 0
            }),
            "something wrong with user1 before deposit!"
        );
    }

    #[test]
    fn test_vesting_first() {
        let first_per = 25;
        let max = 5_000_000;
        let applied = 6_000_000;
        let token_per_unit = 200_000_000;

        let mut state_builder = TestStateBuilder::new();
        let params = init_parameter(BTreeMap::new());
        let schedule = SaleSchedule::new(
            Timestamp::from_timestamp_millis(1),
            BTreeMap::from([
                (Timestamp::from_timestamp_millis(10), Prior::TOP),
                (Timestamp::from_timestamp_millis(20), Prior::SECOND),
            ]),
            Timestamp::from_timestamp_millis(30),
            BTreeMap::from([
                (Duration::from_millis(10), first_per),
                (Duration::from_millis(20), 40),
                (Duration::from_millis(30), 35),
            ]),
        )
        .unwrap();
        let saleinfo = SaleInfo::new(15_000_000, token_per_unit.into(), max, 100).unwrap();

        let mut state = State::new(
            &mut state_builder,
            params.proj_admin,
            params.addr_ovl,
            params.addr_bbb,
            schedule,
            saleinfo,
        );
        let cur_inc = 0;
        let total_units = cmp::min(state.saleinfo.max_units, applied);
        let ret = state
            .calc_vesting_amount(
                Timestamp::from_timestamp_millis(61),
                Timestamp::from_timestamp_millis(50),
                total_units as u64,
                PUBLIC_RIDO_FEE_OVL,
                cur_inc,
            )
            .unwrap();

        claim_eq!(
            ret.0 .0,
            total_units as u64 * token_per_unit * PUBLIC_RIDO_FEE_OVL as u64 / 100
                * first_per as u64
                / 100,
            "Something wrong with vesting calcuration!"
        );
        claim_eq!(ret.1, 1, "Something wrong with claimed_inc!");
    }

    #[test]
    fn test_vesting_too_early() {
        let first_per = 25;
        let max = 5_000_000;
        let applied = 6_000_000;
        let token_per_unit = 200_000_000;

        let mut state_builder = TestStateBuilder::new();
        let params = init_parameter(BTreeMap::new());
        let schedule = SaleSchedule::new(
            Timestamp::from_timestamp_millis(1),
            BTreeMap::from([
                (Timestamp::from_timestamp_millis(10), Prior::TOP),
                (Timestamp::from_timestamp_millis(20), Prior::SECOND),
            ]),
            Timestamp::from_timestamp_millis(30),
            BTreeMap::from([
                (Duration::from_millis(10), first_per),
                (Duration::from_millis(20), 40),
                (Duration::from_millis(30), 35),
            ]),
        )
        .unwrap();
        let saleinfo = SaleInfo::new(15_000_000, token_per_unit.into(), max, 100).unwrap();

        let mut state = State::new(
            &mut state_builder,
            params.proj_admin,
            params.addr_ovl,
            params.addr_bbb,
            schedule,
            saleinfo,
        );
        let cur_inc = 0;
        let total_units = cmp::min(state.saleinfo.max_units, applied);
        let ret = state
            .calc_vesting_amount(
                Timestamp::from_timestamp_millis(40),
                Timestamp::from_timestamp_millis(50),
                total_units as u64,
                PUBLIC_RIDO_FEE_OVL,
                cur_inc,
            )
            .unwrap();
        claim_eq!(ret.0 .0, 0, "Something wrong with vesting calcuration!");
        claim_eq!(ret.1, 0, "Something wrong with claimed_inc!");
    }

    #[test]
    fn test_vesting_second_all_at_once() {
        let first_per = 25;
        let second_per = 40;
        let max = 5_000_000;
        let applied = 6_000_000;
        let token_per_unit = 200_000_000;

        let mut state_builder = TestStateBuilder::new();
        let params = init_parameter(BTreeMap::new());
        let schedule = SaleSchedule::new(
            Timestamp::from_timestamp_millis(1),
            BTreeMap::from([
                (Timestamp::from_timestamp_millis(10), Prior::TOP),
                (Timestamp::from_timestamp_millis(20), Prior::SECOND),
            ]),
            Timestamp::from_timestamp_millis(30),
            BTreeMap::from([
                (Duration::from_millis(10), first_per),
                (Duration::from_millis(20), second_per),
                (Duration::from_millis(30), 35),
            ]),
        )
        .unwrap();
        let saleinfo = SaleInfo::new(15_000_000, token_per_unit.into(), max, 100).unwrap();

        let mut state = State::new(
            &mut state_builder,
            params.proj_admin,
            params.addr_ovl,
            params.addr_bbb,
            schedule,
            saleinfo,
        );
        let cur_inc = 0;
        let total_units = cmp::min(state.saleinfo.max_units, applied);
        let ret = state
            .calc_vesting_amount(
                Timestamp::from_timestamp_millis(70),
                Timestamp::from_timestamp_millis(50),
                total_units as u64,
                PUBLIC_RIDO_FEE_OVL,
                cur_inc,
            )
            .unwrap();

        claim_eq!(
            ret.0 .0,
            total_units as u64 * token_per_unit * PUBLIC_RIDO_FEE_OVL as u64 / 100
                * (first_per + second_per) as u64
                / 100,
            "Something wrong with vesting calcuration!"
        );
        claim_eq!(ret.1, 2, "Something wrong with claimed_inc!");
    }

    #[test]
    fn test_vesting_second_separate() {
        let first_per = 25;
        let second_per = 40;
        let max = 5_000_000;
        let applied = 6_000_000;
        let token_per_unit = 200_000_000;

        let mut state_builder = TestStateBuilder::new();
        let params = init_parameter(BTreeMap::new());
        let schedule = SaleSchedule::new(
            Timestamp::from_timestamp_millis(1),
            BTreeMap::from([
                (Timestamp::from_timestamp_millis(10), Prior::TOP),
                (Timestamp::from_timestamp_millis(20), Prior::SECOND),
            ]),
            Timestamp::from_timestamp_millis(30),
            BTreeMap::from([
                (Duration::from_millis(10), first_per),
                (Duration::from_millis(20), second_per),
                (Duration::from_millis(30), 35),
            ]),
        )
        .unwrap();
        let saleinfo = SaleInfo::new(15_000_000, token_per_unit.into(), max, 100).unwrap();

        let mut state = State::new(
            &mut state_builder,
            params.proj_admin,
            params.addr_ovl,
            params.addr_bbb,
            schedule,
            saleinfo,
        );
        let cur_inc = 1;
        let total_units = cmp::min(state.saleinfo.max_units, applied);
        let ret = state
            .calc_vesting_amount(
                Timestamp::from_timestamp_millis(70),
                Timestamp::from_timestamp_millis(50),
                total_units as u64,
                PUBLIC_RIDO_FEE_OVL,
                cur_inc,
            )
            .unwrap();

        claim_eq!(
            ret.0 .0,
            total_units as u64 * token_per_unit * PUBLIC_RIDO_FEE_OVL as u64 / 100
                * second_per as u64
                / 100,
            "Something wrong with vesting calcuration!"
        );
        claim_eq!(ret.1, 2, "Something wrong with claimed_inc!");
    }

    #[test]
    fn test_vesting_check_u64() {
        let first_per = 25;
        let max = 1000;
        let applied = 1000;
        let price_per_token = 100;
        let token_per_unit = 18_446_744_073_709_551;

        let mut state_builder = TestStateBuilder::new();
        let params = init_parameter(BTreeMap::new());
        let schedule = SaleSchedule::new(
            Timestamp::from_timestamp_millis(1),
            BTreeMap::from([
                (Timestamp::from_timestamp_millis(10), Prior::TOP),
                (Timestamp::from_timestamp_millis(20), Prior::SECOND),
            ]),
            Timestamp::from_timestamp_millis(30),
            BTreeMap::from([
                (Duration::from_millis(10), first_per),
                (Duration::from_millis(20), 40),
                (Duration::from_millis(30), 35),
            ]),
        )
        .unwrap();
        let saleinfo = SaleInfo::new(price_per_token, token_per_unit.into(), max, 10).unwrap();

        let mut state = State::new(
            &mut state_builder,
            params.proj_admin,
            params.addr_ovl,
            params.addr_bbb,
            schedule,
            saleinfo,
        );
        let cur_inc = 0;
        let total_units = cmp::min(state.saleinfo.max_units, applied);
        let ret = state
            .calc_vesting_amount(
                Timestamp::from_timestamp_millis(61),
                Timestamp::from_timestamp_millis(50),
                total_units as u64,
                PUBLIC_RIDO_FEE_OVL,
                cur_inc,
            )
            .unwrap();

        // overflow
        // let ans = total_units as u64 * token_per_unit * PUBLIC_RIDO_FEE_OVL as u64 / 100
        //     * first_per as u64
        //     / 100;

        claim_eq!(
            ret.0 .0,
            230584300921369387,
            "Something wrong with vesting calcuration!"
        );
    }

    #[test]
    fn test_vesting_overflow() {
        let first_per = 25;
        let max = 100_000;
        let applied = 100_000;
        let price_per_token = 100;
        let token_per_unit = 18_446_744_073_709_551;

        let mut state_builder = TestStateBuilder::new();
        let params = init_parameter(BTreeMap::new());
        let schedule = SaleSchedule::new(
            Timestamp::from_timestamp_millis(1),
            BTreeMap::from([
                (Timestamp::from_timestamp_millis(10), Prior::TOP),
                (Timestamp::from_timestamp_millis(20), Prior::SECOND),
            ]),
            Timestamp::from_timestamp_millis(30),
            BTreeMap::from([
                (Duration::from_millis(10), first_per),
                (Duration::from_millis(20), 40),
                (Duration::from_millis(30), 35),
            ]),
        )
        .unwrap();
        let saleinfo = SaleInfo::new(price_per_token, token_per_unit.into(), max, 100).unwrap();

        let mut state = State::new(
            &mut state_builder,
            params.proj_admin,
            params.addr_ovl,
            params.addr_bbb,
            schedule,
            saleinfo,
        );
        let cur_inc = 0;
        let total_units = cmp::min(state.saleinfo.max_units, applied);
        let ret = state.calc_vesting_amount(
            Timestamp::from_timestamp_millis(61),
            Timestamp::from_timestamp_millis(50),
            total_units as u64,
            PUBLIC_RIDO_FEE_OVL,
            cur_inc,
        );
        claim_eq!(
            ret,
            Err(ContractError::from(CustomContractError::OverflowError)),
            "Should overflow!"
        );
    }
}
