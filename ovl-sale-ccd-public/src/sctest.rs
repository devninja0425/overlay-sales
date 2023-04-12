use crate::{test_infrastructure::*, *};
use core::fmt::Debug;
use std::sync::atomic::{AtomicU8, Ordering};

static ADDRESS_COUNTER: AtomicU8 = AtomicU8::new(10);
const OVL_TEAM_ACC: AccountAddress = AccountAddress([0u8; 32]);
const PJ_ADMIN_ACC: AccountAddress = AccountAddress([1u8; 32]);
const ADDR_OVL: Address = Address::Account(AccountAddress([2u8; 32]));
const ADDR_BBB: Address = Address::Contract(ContractAddress {
    index: 100,
    subindex: 0,
});

fn new_account() -> AccountAddress {
    let account = AccountAddress([ADDRESS_COUNTER.load(Ordering::SeqCst); 32]);
    ADDRESS_COUNTER.fetch_add(1, Ordering::SeqCst);
    account
}

pub(crate) fn init_parameter(vesting_period: BTreeMap<Duration, AllowedPercentage>) -> InitParams {
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

fn init_ctx(
    sender: AccountAddress,
    slot_time: SlotTime,
    parameter_bytes: &[u8],
) -> TestInitContext {
    let mut ctx = TestInitContext::empty();
    ctx.set_init_origin(sender);
    ctx.set_metadata_slot_time(slot_time);
    ctx.set_parameter(parameter_bytes);
    ctx
}

fn receive_ctx(
    owner: AccountAddress,
    sender: AccountAddress,
    slot_time: SlotTime,
    parameter_bytes: &[u8],
) -> TestReceiveContext {
    let mut ctx = TestReceiveContext::empty();
    ctx.set_self_address(ContractAddress::new(10, 0));
    ctx.set_sender(Address::Account(sender));
    ctx.set_invoker(sender);
    ctx.set_owner(owner);
    ctx.set_metadata_slot_time(slot_time);
    ctx.set_parameter(parameter_bytes);
    ctx
}

fn initial_state<S: HasStateApi>(
    state_builder: &mut StateBuilder<S>,
    schedule: Option<SaleSchedule>,
    saleinfo: Option<SaleInfo>,
) -> State<S> {
    let params = init_parameter(BTreeMap::new());

    let schedule = if schedule.is_none() {
        SaleSchedule::new(
            Timestamp::from_timestamp_millis(1),
            params.open_at,
            params.close_at,
            params.vesting_period,
        )
        .unwrap_abort()
    } else {
        schedule.unwrap()
    };

    let saleinfo = if saleinfo.is_none() {
        SaleInfo::new(
            params.price_per_token,
            params.token_per_unit,
            params.max_units,
            params.min_units,
        )
        .unwrap_abort()
    } else {
        saleinfo.unwrap()
    };

    let state = State::new(
        state_builder,
        params.proj_admin,
        params.addr_ovl,
        params.addr_bbb,
        schedule,
        saleinfo,
    );
    state
}

fn expect_error<E, T>(expr: Result<T, E>, err: E, msg: &str)
where
    E: Eq + Debug,
    T: Debug,
{
    let actual = expr.expect_err_report(msg);
    claim_eq!(actual, err);
}

mod overlay_team;
mod participant;
mod project_admin;

#[concordium_cfg_test]
mod test_user {
    use super::*;

    #[concordium_test]
    fn test_deposit_before_ready() {
        let parameter_bytes: Vec<u8> = to_bytes(&init_parameter(BTreeMap::new()));
        let ctx = init_ctx(
            OVL_TEAM_ACC,
            Timestamp::from_timestamp_millis(1),
            &parameter_bytes,
        );
        let mut state_builder = TestStateBuilder::new();
        let state = contract_init(&ctx, &mut state_builder).unwrap();
        // state.status = SaleStatus::Prepare;
        let mut host = TestHost::new(state, state_builder);

        // let ctx = receive_ctx(
        //     OVL_TEAM_ACC,
        //     OVL_TEAM_ACC,
        //     Timestamp::from_timestamp_millis(12),
        //     &[],
        // );
        // let _ = contract_set_prepare(&ctx, &mut host);

        let ctx = receive_ctx(
            OVL_TEAM_ACC,
            new_account(),
            Timestamp::from_timestamp_millis(15),
            &[],
        );
        let error: ContractResult<()> =
            contract_user_deposit(&ctx, &mut host, Amount::from_micro_ccd(100));
        expect_error(
            error,
            CustomContractError::SaleNotReady.into(),
            "this call should fail when sale is not ready",
        );
        // claim_eq!(
        //     error,
        //     Err(CustomContractError::SaleNotReady.into()),
        //     "Function should throw an error."
        // );
    }

    #[concordium_test]
    fn test_deposit_before_open() {
        let mut state_builder = TestStateBuilder::new();
        let mut state = initial_state(&mut state_builder, None, None);
        state.status = SaleStatus::Ready;

        let mut host = TestHost::new(state, state_builder);

        let ctx = receive_ctx(
            OVL_TEAM_ACC,
            new_account(),
            Timestamp::from_timestamp_millis(5),
            &[],
        );
        let amount = Amount::from_ccd(100);
        let result: ContractResult<()> = contract_user_deposit(&ctx, &mut host, amount);
        expect_error(
            result,
            CustomContractError::InvalidSchedule.into(),
            "this call should fail when sale is not open",
        );
    }

    #[concordium_test]
    fn test_quit() {
        let acc1 = new_account();
        let acc2 = new_account();

        let mut state_builder = TestStateBuilder::new();
        let state = initial_state(&mut state_builder, None, None);
        let mut host = TestHost::new(state, state_builder);
        let balance = Amount::from_micro_ccd(5_000_000 * 200 * 10);
        host.set_self_balance(balance);

        let params = vec![
            AllowedUserParams {
                user: Address::Account(acc1),
                prior: Prior::TOP,
            },
            AllowedUserParams {
                user: Address::Account(acc2),
                prior: Prior::SECOND,
            },
        ];
        let params_bytes: Vec<u8> = to_bytes(&params);
        let ctx = receive_ctx(
            OVL_TEAM_ACC,
            OVL_TEAM_ACC,
            Timestamp::from_timestamp_millis(5),
            &params_bytes,
        );
        let _ = contract_whitelisting(&ctx, &mut host);

        let ctx = receive_ctx(
            OVL_TEAM_ACC,
            acc1,
            Timestamp::from_timestamp_millis(15),
            &[],
        );
        let amount = Amount::from_micro_ccd(5_000_000 * 200 * 1);
        let _ = contract_user_deposit(&ctx, &mut host, amount);

        let ctx = receive_ctx(
            OVL_TEAM_ACC,
            acc1,
            Timestamp::from_timestamp_millis(25),
            &[],
        );
        let ret = contract_user_quit(&ctx, &mut host);
        expect_error(
            ret,
            CustomContractError::DisabledForNow.into(),
            "this call should fail because disabled",
        );
        // claim_eq!(
        //     host.get_transfers(),
        //     [(acc1, amount)],
        //     "Something wrong with user claim."
        // );
        // for (addr, user_state) in host.state_mut().participants.iter() {
        //     println!("{:?}", *addr);
        //     println!("{:?}", *user_state);
        // }
    }

    #[concordium_test]
    fn test_user_claim() {
        let accounts = vec![new_account(), new_account(), new_account()];

        let schedule = SaleSchedule::new(
            Timestamp::from_timestamp_millis(1),
            BTreeMap::from([
                (Timestamp::from_timestamp_millis(10), Prior::TOP),
                (Timestamp::from_timestamp_millis(20), Prior::SECOND),
            ]),
            Timestamp::from_timestamp_millis(30),
            BTreeMap::from([
                (Duration::from_millis(10), 25),
                (Duration::from_millis(20), 40),
                (Duration::from_millis(30), 35),
            ]),
        )
        .ok();
        let saleinfo = SaleInfo::new(5_000_000, 200.into(), 100, 50).ok();

        let mut state_builder = TestStateBuilder::new();
        let mut state = initial_state(&mut state_builder, schedule, saleinfo);
        state.status = SaleStatus::Fixed;
        state.project_token = Some(ContractAddress::new(1, 0));
        state.schedule.vesting_start = Some(Timestamp::from_timestamp_millis(50));
        state.saleinfo.applied_units = 80;

        for acc in accounts.iter() {
            state.participants.insert(
                Address::from(*acc),
                UserState {
                    prior: Prior::TOP,
                    deposit_ccd: Amount::from_micro_ccd(5_000_000 * 200 * 1),
                    tgt_units: TARGET_UNITS,
                    win_units: 1,
                    claimed_inc: 0,
                },
            );
        }

        let mut host = TestHost::new(state, state_builder);
        host.setup_mock_entrypoint(
            ContractAddress::new(1, 0),
            OwnedEntrypointName::new_unchecked("transfer".into()),
            MockFn::returning_ok(()),
        );

        let mut ctx = TestReceiveContext::empty();
        ctx.set_self_address(ContractAddress::new(0, 0));
        ctx.set_owner(OVL_TEAM_ACC);
        ctx.set_sender(Address::from(accounts[0]));

        ctx.set_metadata_slot_time(Timestamp::from_timestamp_millis(60));
        let ret: ContractResult<()> = contract_user_claim(&ctx, &mut host);
        claim!(ret.is_ok(), "Results in rejection");
        let ret: ContractResult<()> = contract_user_claim(&ctx, &mut host);
        claim!(ret.is_ok(), "Results in rejection");

        ctx.set_metadata_slot_time(Timestamp::from_timestamp_millis(70));
        let ret: ContractResult<()> = contract_user_claim(&ctx, &mut host);
        claim!(ret.is_ok(), "Results in rejection");
        let ret: ContractResult<()> = contract_user_claim(&ctx, &mut host);
        claim!(ret.is_ok(), "Results in rejection");

        ctx.set_metadata_slot_time(Timestamp::from_timestamp_millis(80));
        let ret: ContractResult<()> = contract_user_claim(&ctx, &mut host);
        claim!(ret.is_ok(), "Results in rejection");
        let ret: ContractResult<()> = contract_user_claim(&ctx, &mut host);
        claim!(ret.is_ok(), "Results in rejection");

        ctx.set_metadata_slot_time(Timestamp::from_timestamp_millis(90));
        let ret: ContractResult<()> = contract_user_claim(&ctx, &mut host);
        claim!(ret.is_ok(), "Results in rejection");

        ctx.set_sender(Address::from(accounts[1]));
        ctx.set_metadata_slot_time(Timestamp::from_timestamp_millis(100));
        let ret: ContractResult<()> = contract_user_claim(&ctx, &mut host);
        claim!(ret.is_ok(), "Results in rejection");
    }
}
