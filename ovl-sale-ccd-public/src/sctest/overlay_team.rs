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
        claim!(result.is_err());
        let err = result.expect_err_report("init should reject");
        claim_eq!(
            err,
            CustomContractError::OverflowError.into(),
            "init should reject with OverflowError"
        );
    }
}
