use concordium_std::concordium_cfg_test;

#[concordium_cfg_test]
mod tests {
    use crate::*;
    use concordium_std::test_infrastructure::*;

    #[concordium_test]
    /// Test that init succeeds.
    fn test_init() {
        let mut state_builder = TestStateBuilder::new();
        let invoker = AccountAddress([0u8; 32]);
        let slot_time = Timestamp::from_timestamp_millis(1);
        let proj_admin = AccountAddress([1u8; 32]);
        let addr_ovl = Address::Account(AccountAddress([2u8; 32]));
        let addr_bbb = Address::Contract(ContractAddress {
            index: 100,
            subindex: 0,
        });
        let open_at = BTreeMap::from([
            (Timestamp::from_timestamp_millis(10), Prior::TOP),
            (Timestamp::from_timestamp_millis(20), Prior::SECOND),
        ]);
        let close_at = Timestamp::from_timestamp_millis(30);
        let vesting_period = BTreeMap::from([
            (Duration::from_days(1), 25),
            (Duration::from_days(2), 40),
            (Duration::from_days(3), 35),
        ]);
        let max_units = 100;
        let min_units = 50;
        let price_per_token = 5_000_000;
        let token_per_unit = 200.into();

        let expected_state = State {
            proj_admin,
            status: SaleStatus::Prepare,
            paused: false,
            addr_ovl,
            addr_bbb,
            ovl_claimed_inc: 0,
            bbb_claimed_inc: 0,
            project_token: None,
            schedule: SaleSchedule {
                open_at: open_at.clone(),
                close_at,
                vesting_start: None,
                vesting_period: vesting_period.clone(),
            },
            saleinfo: SaleInfo {
                price_per_token,
                token_per_unit,
                max_units,
                min_units,
                applied_units: 0,
            },
            participants: state_builder.new_map(),
        };

        // set init context
        let mut ctx = TestInitContext::empty();
        ctx.set_init_origin(invoker);
        ctx.set_metadata_slot_time(slot_time);

        // create params
        let params = InitParams {
            proj_admin,
            addr_ovl,
            addr_bbb,
            open_at,
            close_at,
            max_units,
            min_units,
            price_per_token,
            token_per_unit,
            vesting_period,
        };
        let params_byte = to_bytes(&params);
        ctx.set_parameter(&params_byte);

        // execute init
        let result = contract_init(&ctx, &mut state_builder);
        claim!(result.is_ok());
        let actual_state = result.unwrap();
        claim_eq!(
            actual_state,
            expected_state,
            "state has been initialized unexpectedly..."
        );
    }

    #[concordium_test]
    /// Test that init fails with InvalidSchedule error.
    /// Current time is newer than open schedule.
    fn test_init_fails_with_invalid_schedule() {
        let mut state_builder = TestStateBuilder::new();
        let invoker = AccountAddress([0u8; 32]);
        let slot_time = Timestamp::from_timestamp_millis(50);
        let proj_admin = AccountAddress([1u8; 32]);
        let addr_ovl = Address::Account(AccountAddress([2u8; 32]));
        let addr_bbb = Address::Contract(ContractAddress {
            index: 100,
            subindex: 0,
        });
        let open_at = BTreeMap::from([
            (Timestamp::from_timestamp_millis(10), Prior::TOP),
            (Timestamp::from_timestamp_millis(20), Prior::SECOND),
        ]);
        let close_at = Timestamp::from_timestamp_millis(30);
        let vesting_period = BTreeMap::from([
            (Duration::from_days(1), 25),
            (Duration::from_days(2), 40),
            (Duration::from_days(3), 35),
        ]);
        let max_units = 100;
        let min_units = 50;
        let price_per_token = 5_000_000;
        let token_per_unit = 200.into();

        // set init context
        let mut ctx = TestInitContext::empty();
        ctx.set_init_origin(invoker);
        ctx.set_metadata_slot_time(slot_time);

        // create params
        let params = InitParams {
            proj_admin,
            addr_ovl,
            addr_bbb,
            open_at,
            close_at,
            max_units,
            min_units,
            price_per_token,
            token_per_unit,
            vesting_period,
        };
        let params_byte = to_bytes(&params);
        ctx.set_parameter(&params_byte);

        // execute init
        let result = contract_init(&ctx, &mut state_builder);
        claim!(result.is_err());
        let err = result.expect_err_report("init should reject");
        claim_eq!(
            err,
            CustomContractError::InvalidSchedule.into(),
            "init should reject with InvalidSchedule"
        );
    }

    #[concordium_test]
    /// Test that init fails with Inappropriate error.
    /// Total percentage of vesting period != 100.
    fn test_init_invalid_vesting_total() {
        let mut state_builder = TestStateBuilder::new();
        let invoker = AccountAddress([0u8; 32]);
        let slot_time = Timestamp::from_timestamp_millis(1);
        let proj_admin = AccountAddress([1u8; 32]);
        let addr_ovl = Address::Account(AccountAddress([2u8; 32]));
        let addr_bbb = Address::Contract(ContractAddress {
            index: 100,
            subindex: 0,
        });
        let open_at = BTreeMap::from([
            (Timestamp::from_timestamp_millis(10), Prior::TOP),
            (Timestamp::from_timestamp_millis(20), Prior::SECOND),
        ]);
        let close_at = Timestamp::from_timestamp_millis(30);
        let vesting_period = BTreeMap::from([
            (Duration::from_days(30), 33),
            (Duration::from_days(60), 33),
            (Duration::from_days(90), 35),
        ]);
        let max_units = 100;
        let min_units = 50;
        let price_per_token = 5_000_000;
        let token_per_unit = 200.into();

        // set init context
        let mut ctx = TestInitContext::empty();
        ctx.set_init_origin(invoker);
        ctx.set_metadata_slot_time(slot_time);

        // create params
        let params = InitParams {
            proj_admin,
            addr_ovl,
            addr_bbb,
            open_at,
            close_at,
            max_units,
            min_units,
            price_per_token,
            token_per_unit,
            vesting_period,
        };
        let params_byte = to_bytes(&params);
        ctx.set_parameter(&params_byte);

        // execute init
        let result = contract_init(&ctx, &mut state_builder);
        claim!(result.is_err());
        let err = result.expect_err_report("init should reject");
        claim_eq!(
            err,
            CustomContractError::Inappropriate.into(),
            "init should reject with Inappropriate"
        );
    }

    #[concordium_test]
    /// Test that init fails with Inappropriate error.
    /// case max_units == min_units
    fn test_init_invalid_min_max_unit() {
        let mut state_builder = TestStateBuilder::new();
        let invoker = AccountAddress([0u8; 32]);
        let slot_time = Timestamp::from_timestamp_millis(1);
        let proj_admin = AccountAddress([1u8; 32]);
        let addr_ovl = Address::Account(AccountAddress([2u8; 32]));
        let addr_bbb = Address::Contract(ContractAddress {
            index: 100,
            subindex: 0,
        });
        let open_at = BTreeMap::from([
            (Timestamp::from_timestamp_millis(10), Prior::TOP),
            (Timestamp::from_timestamp_millis(20), Prior::SECOND),
        ]);
        let close_at = Timestamp::from_timestamp_millis(30);
        let vesting_period = BTreeMap::from([
            (Duration::from_days(30), 20),
            (Duration::from_days(60), 30),
            (Duration::from_days(90), 50),
        ]);
        let max_units = 0;
        let min_units = 0;
        let price_per_token = 5_000_000;
        let token_per_unit = 200.into();

        // set init context
        let mut ctx = TestInitContext::empty();
        ctx.set_init_origin(invoker);
        ctx.set_metadata_slot_time(slot_time);

        // create params
        let params = InitParams {
            proj_admin,
            addr_ovl,
            addr_bbb,
            open_at,
            close_at,
            max_units,
            min_units,
            price_per_token,
            token_per_unit,
            vesting_period,
        };
        let params_byte = to_bytes(&params);
        ctx.set_parameter(&params_byte);

        // execute init
        let result = contract_init(&ctx, &mut state_builder);
        claim!(result.is_err());
        let err = result.expect_err_report("init should reject");
        claim_eq!(
            err,
            CustomContractError::Inappropriate.into(),
            "init should reject with Inappropriate"
        );
    }

    #[concordium_test]
    /// Test that init fails with OverflowError error.
    /// case price_per_token * token_per_uni > 18_446_744_073_709_551_615 overflow
    /// see https://doc.rust-lang.org/std/u64/constant.MAX.html
    fn test_init_unit_price_overflow() {
        let mut state_builder = TestStateBuilder::new();
        let invoker = AccountAddress([0u8; 32]);
        let slot_time = Timestamp::from_timestamp_millis(1);
        let proj_admin = AccountAddress([1u8; 32]);
        let addr_ovl = Address::Account(AccountAddress([2u8; 32]));
        let addr_bbb = Address::Contract(ContractAddress {
            index: 100,
            subindex: 0,
        });
        let open_at = BTreeMap::from([
            (Timestamp::from_timestamp_millis(10), Prior::TOP),
            (Timestamp::from_timestamp_millis(20), Prior::SECOND),
        ]);
        let close_at = Timestamp::from_timestamp_millis(30);
        let vesting_period = BTreeMap::from([
            (Duration::from_days(30), 20),
            (Duration::from_days(60), 30),
            (Duration::from_days(90), 50),
        ]);
        let max_units = 100;
        let min_units = 50;
        let price_per_token = 2000_000_000;
        let token_per_unit = 9_300_000_000.into();

        // set init context
        let mut ctx = TestInitContext::empty();
        ctx.set_init_origin(invoker);
        ctx.set_metadata_slot_time(slot_time);

        // create params
        let params = InitParams {
            proj_admin,
            addr_ovl,
            addr_bbb,
            open_at,
            close_at,
            max_units,
            min_units,
            price_per_token,
            token_per_unit,
            vesting_period,
        };
        let params_byte = to_bytes(&params);
        ctx.set_parameter(&params_byte);

        // execute init
        let result = contract_init(&ctx, &mut state_builder);
        let err = result.expect_err_report("init should reject");
        claim_eq!(
            err,
            CustomContractError::OverflowError.into(),
            "init should reject with OverflowError"
        );
    }

    #[concordium_test]
    /// Test that setPaused/setUnpaused works as intended.
    /// if `pause` flag is active the following features will be disabled.
    /// - `createPool`
    /// - `projectClaim`
    /// - `userDeposit`
    /// - `userQuit`
    /// - `userClaim`
    fn test_pause_controls() {
        let mut state_builder = TestStateBuilder::new();
        let admin = AccountAddress([0u8; 32]);
        let proj_admin = AccountAddress([1u8; 32]);
        let some_user = AccountAddress([2u8; 32]);
        let project_token_address = ContractAddress {
            index: 1000,
            subindex: 0,
        };
        let addr_ovl = Address::Account(AccountAddress([2u8; 32]));
        let addr_bbb = Address::Contract(ContractAddress {
            index: 100,
            subindex: 0,
        });
        let open_at = BTreeMap::from([
            (Timestamp::from_timestamp_millis(10), Prior::TOP),
            (Timestamp::from_timestamp_millis(20), Prior::SECOND),
        ]);
        let close_at = Timestamp::from_timestamp_millis(30);
        let vesting_period = BTreeMap::from([
            (Duration::from_days(1), 25),
            (Duration::from_days(2), 40),
            (Duration::from_days(3), 35),
        ]);
        let max_units = 100;
        let min_units = 50;
        let price_per_token = 5_000_000;
        let token_per_unit = 200.into();
        let initial_state = State {
            proj_admin,
            status: SaleStatus::Prepare,
            paused: false,
            addr_ovl,
            addr_bbb,
            ovl_claimed_inc: 0,
            bbb_claimed_inc: 0,
            project_token: Some(project_token_address),
            schedule: SaleSchedule {
                open_at: open_at.clone(),
                close_at,
                vesting_start: None,
                vesting_period: vesting_period.clone(),
            },
            saleinfo: SaleInfo {
                price_per_token,
                token_per_unit,
                max_units,
                min_units,
                applied_units: 0,
            },
            participants: state_builder.new_map(),
        };
        let after_paused_state = State {
            proj_admin,
            status: SaleStatus::Prepare,
            paused: true,
            addr_ovl,
            addr_bbb,
            ovl_claimed_inc: 0,
            bbb_claimed_inc: 0,
            project_token: Some(project_token_address),
            schedule: SaleSchedule {
                open_at: open_at.clone(),
                close_at,
                vesting_start: None,
                vesting_period: vesting_period.clone(),
            },
            saleinfo: SaleInfo {
                price_per_token,
                token_per_unit,
                max_units,
                min_units,
                applied_units: 0,
            },
            participants: state_builder.new_map(),
        };
        let after_unpaused_state = State {
            proj_admin,
            status: SaleStatus::Prepare,
            paused: false,
            addr_ovl,
            addr_bbb,
            ovl_claimed_inc: 0,
            bbb_claimed_inc: 0,
            project_token: Some(project_token_address),
            schedule: SaleSchedule {
                open_at: open_at.clone(),
                close_at,
                vesting_start: None,
                vesting_period: vesting_period.clone(),
            },
            saleinfo: SaleInfo {
                price_per_token,
                token_per_unit,
                max_units,
                min_units,
                applied_units: 0,
            },
            participants: state_builder.new_map(),
        };
        let mut host = TestHost::new(initial_state, state_builder);

        // create params for setPaused
        let mut ctx = TestReceiveContext::empty();
        ctx.set_owner(admin);
        ctx.set_sender(Address::Account(admin));
        let set_paused_result = contract_set_paused(&ctx, &mut host);
        claim!(set_paused_result.is_ok());
        claim_eq!(
            *host.state(),
            after_paused_state,
            "state has been changed unexpectedly..."
        );

        // let's check the prohibited actions.
        // createPool
        let mut ctx = TestReceiveContext::empty();
        ctx.set_owner(admin);
        ctx.set_sender(Address::Account(admin));
        let result = contract_create_pool(&ctx, &mut host);
        let err = result.expect_err_report("createPool should reject when paused");
        claim_eq!(
            err,
            CustomContractError::ContractPaused.into(),
            "createPool should reject when paused"
        );

        // projectClaim
        let mut ctx = TestReceiveContext::empty();
        ctx.set_owner(admin);
        ctx.set_invoker(proj_admin);
        ctx.set_sender(Address::Contract(project_token_address));
        let result = contract_project_claim(&ctx, &mut host);
        let err = result.expect_err_report("projectClaim should reject when paused");
        claim_eq!(
            err,
            CustomContractError::ContractPaused.into(),
            "projectClaim should reject when paused"
        );

        // userDeposit
        let mut ctx = TestReceiveContext::empty();
        ctx.set_owner(admin);
        ctx.set_sender(Address::Account(some_user));
        let result = contract_user_deposit(&ctx, &mut host, Amount::from_ccd(100));
        let err = result.expect_err_report("userDeposit should reject when paused");
        claim_eq!(
            err,
            CustomContractError::ContractPaused.into(),
            "userDeposit should reject when paused"
        );

        // userQuit
        let mut ctx = TestReceiveContext::empty();
        ctx.set_owner(admin);
        ctx.set_sender(Address::Account(some_user));
        let result = contract_user_quit(&ctx, &mut host);
        let err = result.expect_err_report("userQuit should reject when paused");
        claim_eq!(
            err,
            CustomContractError::ContractPaused.into(),
            "userQuit should reject when paused"
        );

        // userClaim
        let mut ctx = TestReceiveContext::empty();
        ctx.set_owner(admin);
        ctx.set_sender(Address::Account(some_user));
        let result = contract_user_claim(&ctx, &mut host);
        let err = result.expect_err_report("userClaim should reject when paused");
        claim_eq!(
            err,
            CustomContractError::ContractPaused.into(),
            "userClaim should reject when paused"
        );

        // create params for setUnpaused
        let mut ctx = TestReceiveContext::empty();
        ctx.set_owner(admin);
        ctx.set_sender(Address::Account(admin));
        let set_paused_result = contract_set_unpaused(&ctx, &mut host);
        claim!(set_paused_result.is_ok());
        claim_eq!(
            *host.state(),
            after_unpaused_state,
            "state has been changed unexpectedly..."
        );
    }

    #[concordium_test]
    /// Test that setFixed successfully update status as Fixed
    fn test_set_fixed() {
        let mut state_builder = TestStateBuilder::new();
        let admin = AccountAddress([0u8; 32]);
        let proj_admin = AccountAddress([1u8; 32]);
        let project_token_address = ContractAddress {
            index: 1000,
            subindex: 0,
        };
        let addr_ovl = Address::Account(AccountAddress([2u8; 32]));
        let addr_bbb = Address::Contract(ContractAddress {
            index: 100,
            subindex: 0,
        });
        let open_at = BTreeMap::from([
            (Timestamp::from_timestamp_millis(10), Prior::TOP),
            (Timestamp::from_timestamp_millis(20), Prior::SECOND),
        ]);
        let close_at = Timestamp::from_timestamp_millis(30);
        let slot_time = Timestamp::from_timestamp_millis(31);
        let vesting_period = BTreeMap::from([
            (Duration::from_days(1), 25),
            (Duration::from_days(2), 40),
            (Duration::from_days(3), 35),
        ]);
        let max_units = 100;
        let min_units = 50;
        let price_per_token = 5_000_000;
        let token_per_unit = 200.into();
        let initial_state = State {
            proj_admin,
            status: SaleStatus::Ready,
            paused: false,
            addr_ovl,
            addr_bbb,
            ovl_claimed_inc: 0,
            bbb_claimed_inc: 0,
            project_token: Some(project_token_address),
            schedule: SaleSchedule {
                open_at: open_at.clone(),
                close_at,
                vesting_start: None,
                vesting_period: vesting_period.clone(),
            },
            saleinfo: SaleInfo {
                price_per_token,
                token_per_unit,
                max_units,
                min_units,
                applied_units: min_units,
            },
            participants: state_builder.new_map(),
        };
        let expected_state = State {
            proj_admin,
            status: SaleStatus::Fixed,
            paused: false,
            addr_ovl,
            addr_bbb,
            ovl_claimed_inc: 0,
            bbb_claimed_inc: 0,
            project_token: Some(project_token_address),
            schedule: SaleSchedule {
                open_at: open_at.clone(),
                close_at,
                vesting_start: None,
                vesting_period: vesting_period.clone(),
            },
            saleinfo: SaleInfo {
                price_per_token,
                token_per_unit,
                max_units,
                min_units,
                applied_units: min_units,
            },
            participants: state_builder.new_map(),
        };
        let mut host = TestHost::new(initial_state, state_builder);

        // create params
        let mut ctx = TestReceiveContext::empty();
        ctx.set_owner(admin);
        ctx.set_sender(Address::Account(admin));
        ctx.set_metadata_slot_time(slot_time);
        let result = contract_set_fixed(&ctx, &mut host);
        claim!(result.is_ok());
        claim_eq!(
            *host.state(),
            expected_state,
            "state has been changed unexpectedly..."
        );
    }

    #[concordium_test]
    /// Test that setFixed successfully update status as Suspend
    fn test_set_fixed_when_not_reached_its_min_units() {
        let mut state_builder = TestStateBuilder::new();
        let admin = AccountAddress([0u8; 32]);
        let proj_admin = AccountAddress([1u8; 32]);
        let project_token_address = ContractAddress {
            index: 1000,
            subindex: 0,
        };
        let addr_ovl = Address::Account(AccountAddress([2u8; 32]));
        let addr_bbb = Address::Contract(ContractAddress {
            index: 100,
            subindex: 0,
        });
        let open_at = BTreeMap::from([
            (Timestamp::from_timestamp_millis(10), Prior::TOP),
            (Timestamp::from_timestamp_millis(20), Prior::SECOND),
        ]);
        let close_at = Timestamp::from_timestamp_millis(30);
        let slot_time = Timestamp::from_timestamp_millis(31);
        let vesting_period = BTreeMap::from([
            (Duration::from_days(1), 25),
            (Duration::from_days(2), 40),
            (Duration::from_days(3), 35),
        ]);
        let max_units = 100;
        let min_units = 50;
        let price_per_token = 5_000_000;
        let token_per_unit = 200.into();
        let initial_state = State {
            proj_admin,
            status: SaleStatus::Ready,
            paused: false,
            addr_ovl,
            addr_bbb,
            ovl_claimed_inc: 0,
            bbb_claimed_inc: 0,
            project_token: Some(project_token_address),
            schedule: SaleSchedule {
                open_at: open_at.clone(),
                close_at,
                vesting_start: None,
                vesting_period: vesting_period.clone(),
            },
            saleinfo: SaleInfo {
                price_per_token,
                token_per_unit,
                max_units,
                min_units,
                applied_units: min_units - 1,
            },
            participants: state_builder.new_map(),
        };
        let expected_state = State {
            proj_admin,
            status: SaleStatus::Suspend,
            paused: false,
            addr_ovl,
            addr_bbb,
            ovl_claimed_inc: 0,
            bbb_claimed_inc: 0,
            project_token: Some(project_token_address),
            schedule: SaleSchedule {
                open_at: open_at.clone(),
                close_at,
                vesting_start: None,
                vesting_period: vesting_period.clone(),
            },
            saleinfo: SaleInfo {
                price_per_token,
                token_per_unit,
                max_units,
                min_units,
                applied_units: min_units - 1,
            },
            participants: state_builder.new_map(),
        };
        let mut host = TestHost::new(initial_state, state_builder);

        // create params
        let mut ctx = TestReceiveContext::empty();
        ctx.set_owner(admin);
        ctx.set_sender(Address::Account(admin));
        ctx.set_metadata_slot_time(slot_time);
        let result = contract_set_fixed(&ctx, &mut host);
        claim!(result.is_ok());
        claim_eq!(
            *host.state(),
            expected_state,
            "state has been changed unexpectedly..."
        );
    }

    #[concordium_test]
    /// Test that whitelisting successfully update participants & status
    fn test_whitelisted() {
        let mut state_builder = TestStateBuilder::new();
        let admin = AccountAddress([0u8; 32]);
        let proj_admin = AccountAddress([1u8; 32]);
        let project_token_address = ContractAddress {
            index: 1000,
            subindex: 0,
        };
        let addr_ovl = Address::Account(AccountAddress([2u8; 32]));
        let addr_bbb = Address::Contract(ContractAddress {
            index: 100,
            subindex: 0,
        });
        let open_at = BTreeMap::from([
            (Timestamp::from_timestamp_millis(10), Prior::TOP),
            (Timestamp::from_timestamp_millis(20), Prior::SECOND),
        ]);
        let close_at = Timestamp::from_timestamp_millis(30);
        let vesting_period = BTreeMap::from([
            (Duration::from_days(1), 25),
            (Duration::from_days(2), 40),
            (Duration::from_days(3), 35),
        ]);
        let max_units = 100;
        let min_units = 50;
        let price_per_token = 5_000_000;
        let token_per_unit = 200.into();

        let whitelist = vec![
            AllowedUserParams {
                user: Address::Account(AccountAddress([10u8; 32])),
                prior: Prior::TOP,
            },
            AllowedUserParams {
                user: Address::Account(AccountAddress([11u8; 32])),
                prior: Prior::TOP,
            },
            AllowedUserParams {
                user: Address::Account(AccountAddress([12u8; 32])),
                prior: Prior::SECOND,
            },
            AllowedUserParams {
                user: Address::Account(AccountAddress([13u8; 32])),
                prior: Prior::SECOND,
            },
            AllowedUserParams {
                user: Address::Account(AccountAddress([14u8; 32])),
                prior: Prior::ANY,
            },
        ];

        let initial_state = State {
            proj_admin,
            status: SaleStatus::Prepare,
            paused: false,
            addr_ovl,
            addr_bbb,
            ovl_claimed_inc: 0,
            bbb_claimed_inc: 0,
            project_token: Some(project_token_address),
            schedule: SaleSchedule {
                open_at: open_at.clone(),
                close_at,
                vesting_start: None,
                vesting_period: vesting_period.clone(),
            },
            saleinfo: SaleInfo {
                price_per_token,
                token_per_unit,
                max_units,
                min_units,
                applied_units: 0,
            },
            participants: state_builder.new_map(),
        };
        let mut expected_participants = state_builder.new_map();
        for params in &whitelist {
            expected_participants.insert(
                params.user,
                UserState::new(params.prior.clone(), Amount::zero(), TARGET_UNITS),
            );
        }
        let expected_state = State {
            proj_admin,
            status: SaleStatus::Ready,
            paused: false,
            addr_ovl,
            addr_bbb,
            ovl_claimed_inc: 0,
            bbb_claimed_inc: 0,
            project_token: Some(project_token_address),
            schedule: SaleSchedule {
                open_at: open_at.clone(),
                close_at,
                vesting_start: None,
                vesting_period: vesting_period.clone(),
            },
            saleinfo: SaleInfo {
                price_per_token,
                token_per_unit,
                max_units,
                min_units,
                applied_units: 0,
            },
            participants: expected_participants,
        };
        let mut host = TestHost::new(initial_state, state_builder);

        // create params
        let mut ctx = TestReceiveContext::empty();
        ctx.set_owner(admin);
        ctx.set_sender(Address::Account(admin));
        let params_byte = to_bytes(&whitelist);
        ctx.set_parameter(&params_byte);

        // execute function
        let result = contract_whitelisting(&ctx, &mut host);
        claim!(result.is_ok());
        claim_eq!(
            *host.state(),
            expected_state,
            "state has been changed unexpectedly..."
        );
    }

    #[concordium_test]
    /// Test that whitelisting fails with AccountOnly error.
    /// Currently contract address input is not supported.
    fn test_whitelisted_fails_with_account_only() {
        let mut state_builder = TestStateBuilder::new();
        let admin = AccountAddress([0u8; 32]);
        let proj_admin = AccountAddress([1u8; 32]);
        let project_token_address = ContractAddress {
            index: 1000,
            subindex: 0,
        };
        let addr_ovl = Address::Account(AccountAddress([2u8; 32]));
        let addr_bbb = Address::Contract(ContractAddress {
            index: 100,
            subindex: 0,
        });
        let open_at = BTreeMap::from([
            (Timestamp::from_timestamp_millis(10), Prior::TOP),
            (Timestamp::from_timestamp_millis(20), Prior::SECOND),
        ]);
        let close_at = Timestamp::from_timestamp_millis(30);
        let vesting_period = BTreeMap::from([
            (Duration::from_days(1), 25),
            (Duration::from_days(2), 40),
            (Duration::from_days(3), 35),
        ]);
        let max_units = 100;
        let min_units = 50;
        let price_per_token = 5_000_000;
        let token_per_unit = 200.into();

        let whitelist = vec![
            AllowedUserParams {
                user: Address::Account(AccountAddress([10u8; 32])),
                prior: Prior::TOP,
            },
            AllowedUserParams {
                user: Address::Account(AccountAddress([11u8; 32])),
                prior: Prior::TOP,
            },
            AllowedUserParams {
                user: Address::Account(AccountAddress([12u8; 32])),
                prior: Prior::SECOND,
            },
            AllowedUserParams {
                user: Address::Contract(ContractAddress::new(123, 0)),
                prior: Prior::SECOND,
            },
            AllowedUserParams {
                user: Address::Account(AccountAddress([14u8; 32])),
                prior: Prior::ANY,
            },
        ];

        let initial_state = State {
            proj_admin,
            status: SaleStatus::Prepare,
            paused: false,
            addr_ovl,
            addr_bbb,
            ovl_claimed_inc: 0,
            bbb_claimed_inc: 0,
            project_token: Some(project_token_address),
            schedule: SaleSchedule {
                open_at: open_at.clone(),
                close_at,
                vesting_start: None,
                vesting_period: vesting_period.clone(),
            },
            saleinfo: SaleInfo {
                price_per_token,
                token_per_unit,
                max_units,
                min_units,
                applied_units: 0,
            },
            participants: state_builder.new_map(),
        };
        let mut host = TestHost::new(initial_state, state_builder);

        // create params
        let mut ctx = TestReceiveContext::empty();
        ctx.set_owner(admin);
        ctx.set_sender(Address::Account(admin));
        let params_byte = to_bytes(&whitelist);
        ctx.set_parameter(&params_byte);

        // execute function
        let result = contract_whitelisting(&ctx, &mut host);
        let err = result.expect_err_report("should fail");
        claim_eq!(
            err,
            CustomContractError::AccountOnly.into(),
            "whitelisting should reject with AccountOnly"
        );
    }

    #[concordium_quickcheck(num_tests = 10)]
    /// Quick check that whitelisting succeeds with generated addresses and priorities.
    fn test_whitelisted_pbt(participants: Vec<AccountAddress>, prior: Vec<u8>) -> bool {
        let mut state_builder = TestStateBuilder::new();
        let admin = AccountAddress([0u8; 32]);
        let proj_admin = AccountAddress([1u8; 32]);
        let project_token_address = ContractAddress {
            index: 1000,
            subindex: 0,
        };
        let addr_ovl = Address::Account(AccountAddress([2u8; 32]));
        let addr_bbb = Address::Contract(ContractAddress {
            index: 100,
            subindex: 0,
        });
        let open_at = BTreeMap::from([
            (Timestamp::from_timestamp_millis(10), Prior::TOP),
            (Timestamp::from_timestamp_millis(20), Prior::SECOND),
        ]);
        let close_at = Timestamp::from_timestamp_millis(30);
        let vesting_period = BTreeMap::from([
            (Duration::from_days(1), 25),
            (Duration::from_days(2), 40),
            (Duration::from_days(3), 35),
        ]);
        let max_units = 100;
        let min_units = 50;
        let price_per_token = 5_000_000;
        let token_per_unit = 200.into();

        let initial_state = State {
            proj_admin,
            status: SaleStatus::Prepare,
            paused: false,
            addr_ovl,
            addr_bbb,
            ovl_claimed_inc: 0,
            bbb_claimed_inc: 0,
            project_token: Some(project_token_address),
            schedule: SaleSchedule {
                open_at: open_at.clone(),
                close_at,
                vesting_start: None,
                vesting_period: vesting_period.clone(),
            },
            saleinfo: SaleInfo {
                price_per_token,
                token_per_unit,
                max_units,
                min_units,
                applied_units: 0,
            },
            participants: state_builder.new_map(),
        };
        let mut host = TestHost::new(initial_state, state_builder);
        host.set_self_balance(Amount::from_ccd(100));

        let params: Vec<AllowedUserParams> = participants
            .into_iter()
            .enumerate()
            .map(|(n, addr)| AllowedUserParams {
                user: Address::from(addr),
                prior: match prior.get(n) {
                    Some(x) => match x {
                        x if *x > 50 => Prior::TOP,
                        _ => Prior::SECOND,
                    },
                    None => Prior::ANY,
                },
            })
            .collect();

        // create params
        let mut ctx = TestReceiveContext::empty();
        ctx.set_owner(admin);
        ctx.set_sender(Address::Account(admin));
        let params_byte = to_bytes(&params);
        ctx.set_parameter(&params_byte);

        // execute func
        let ret = contract_whitelisting(&ctx, &mut host);
        ret.is_ok()
    }
}
