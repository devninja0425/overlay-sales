use concordium_std::concordium_cfg_test;

#[concordium_cfg_test]
mod tests {
    use crate::*;
    use concordium_std::test_infrastructure::*;

    #[concordium_test]
    /// Test that setPjtoken successfully set project_token.
    fn test_set_pjtoken() {
        let mut state_builder = TestStateBuilder::new();
        let admin = AccountAddress([0u8; 32]);
        let proj_admin = AccountAddress([1u8; 32]);
        let project_token_address_to_be_set = ContractAddress {
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
            status: SaleStatus::Ready,
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
                applied_units: min_units,
            },
            participants: state_builder.new_map(),
        };
        let expected_state = State {
            proj_admin,
            status: SaleStatus::Ready,
            paused: false,
            addr_ovl,
            addr_bbb,
            ovl_claimed_inc: 0,
            bbb_claimed_inc: 0,
            project_token: Some(project_token_address_to_be_set),
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
        ctx.set_sender(Address::Account(proj_admin));
        let params_byes = to_bytes(&project_token_address_to_be_set);
        ctx.set_parameter(&params_byes);

        // execute func
        let result = contract_set_pjtoken(&ctx, &mut host);
        claim!(result.is_ok());
        claim_eq!(
            *host.state(),
            expected_state,
            "state has been changed unexpectedly..."
        );
    }
}
