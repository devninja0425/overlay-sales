use concordium_std::concordium_cfg_test;

#[concordium_cfg_test]
mod tests {
    use crate::*;
    use concordium_std::test_infrastructure::*;

    #[concordium_test]
    /// Test that userDeposit successfully update user state.
    fn test_user_deposit() {
        let mut state_builder = TestStateBuilder::new();
        let admin = AccountAddress([0u8; 32]);
        let proj_admin = AccountAddress([1u8; 32]);
        let first_user = AccountAddress([10u8; 32]);
        let second_user = AccountAddress([11u8; 32]);
        let deposit_amount = Amount::from_micro_ccd(5_000_000 * 200 * 1);
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
                user: Address::Account(first_user),
                prior: Prior::TOP,
            },
            AllowedUserParams {
                user: Address::Account(second_user),
                prior: Prior::SECOND,
            },
        ];
        let mut participants = state_builder.new_map();
        for params in &whitelist {
            participants.insert(
                params.user,
                UserState::new(params.prior.clone(), Amount::zero(), TARGET_UNITS),
            );
        }
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
                applied_units: 0,
            },
            participants,
        };
        let mut expected_participants = state_builder.new_map();
        for params in &whitelist {
            if params.user == Address::Account(first_user) {
                expected_participants.insert(
                    params.user,
                    UserState {
                        prior: params.prior.clone(),
                        deposit_ccd: deposit_amount,
                        tgt_units: TARGET_UNITS,
                        win_units: 1,
                        claimed_inc: 0,
                    },
                );
            } else {
                expected_participants.insert(
                    params.user,
                    UserState::new(params.prior.clone(), Amount::zero(), TARGET_UNITS),
                );
            }
        }
        let expected_state_after_first_call = State {
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
                applied_units: 1,
            },
            participants: expected_participants,
        };
        let mut expected_participants = state_builder.new_map();
        for params in &whitelist {
            expected_participants.insert(
                params.user,
                UserState {
                    prior: params.prior.clone(),
                    deposit_ccd: deposit_amount,
                    tgt_units: TARGET_UNITS,
                    win_units: 1,
                    claimed_inc: 0,
                },
            );
        }
        let expected_state_after_second_call = State {
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
                applied_units: 2,
            },
            participants: expected_participants,
        };
        let mut host = TestHost::new(initial_state, state_builder);

        // first user call
        // create params
        let mut ctx = TestReceiveContext::empty();
        ctx.set_owner(admin);
        ctx.set_sender(Address::Account(first_user));
        ctx.set_metadata_slot_time(Timestamp::from_timestamp_millis(15));

        // execute function
        let result = contract_user_deposit(&ctx, &mut host, deposit_amount);
        claim!(result.is_ok());
        claim_eq!(*host.state(), expected_state_after_first_call);

        // 2nd user call
        // create params
        let mut ctx = TestReceiveContext::empty();
        ctx.set_owner(admin);
        ctx.set_sender(Address::Account(second_user));
        ctx.set_metadata_slot_time(Timestamp::from_timestamp_millis(25));

        // execute function
        let result = contract_user_deposit(&ctx, &mut host, deposit_amount);
        claim!(result.is_ok());
        claim_eq!(*host.state(), expected_state_after_second_call);
    }
}
