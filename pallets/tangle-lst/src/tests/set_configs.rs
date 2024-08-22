use super::*;

#[test]
fn test_set_configs() {
	ExtBuilder::default().build_and_execute(|| {
		// only root can call
		assert_noop!(
			Pools::set_configs(
				RuntimeOrigin::signed(10),
				ConfigOp::Set(1 as Balance),
				ConfigOp::Set(2 as Balance),
				ConfigOp::Set(Perbill::from_percent(5)),
				ConfigOp::Set(Perbill::from_percent(20)),
			),
			frame_support::error::BadOrigin
		);

		// Setting works
		assert_ok!(Pools::set_configs(
			RuntimeOrigin::root(),
			ConfigOp::Set(1 as Balance),
			ConfigOp::Set(2 as Balance),
			ConfigOp::Set(Perbill::from_percent(5)),
			ConfigOp::Set(Perbill::from_percent(20)),
		));
		assert_eq!(MinJoinBond::<Runtime>::get(), 1);
		assert_eq!(MinCreateBond::<Runtime>::get(), 2);
		assert_eq!(GlobalMaxCommission::<Runtime>::get(), Some(Perbill::from_percent(5)));
		assert_eq!(
			EraPayoutInfo::<Runtime>::get().required_payments_percent,
			Perbill::from_percent(20)
		);

		// Noop does nothing
		assert_storage_noop!(assert_ok!(Pools::set_configs(
			RuntimeOrigin::root(),
			ConfigOp::Noop,
			ConfigOp::Noop,
			ConfigOp::Noop,
			ConfigOp::Noop,
		)));
		assert_eq!(MinJoinBond::<Runtime>::get(), 1);
		assert_eq!(MinCreateBond::<Runtime>::get(), 2);
		assert_eq!(GlobalMaxCommission::<Runtime>::get(), Some(Perbill::from_percent(5)));
		assert_eq!(
			EraPayoutInfo::<Runtime>::get().required_payments_percent,
			Perbill::from_percent(20)
		);

		// Removing works
		assert_ok!(Pools::set_configs(
			RuntimeOrigin::root(),
			ConfigOp::Remove,
			ConfigOp::Remove,
			ConfigOp::Remove,
			ConfigOp::Remove,
		));
		assert_eq!(MinJoinBond::<Runtime>::get(), 0);
		assert_eq!(MinCreateBond::<Runtime>::get(), 0);
		assert_eq!(GlobalMaxCommission::<Runtime>::get(), None);
		assert_eq!(
			EraPayoutInfo::<Runtime>::get().required_payments_percent,
			Perbill::from_percent(0)
		);
	});
}

#[test]
fn test_set_staking_info() {
	ExtBuilder::default().build_and_execute(|| {
		assert_eq!(StakingInformation::<Runtime>::get(), None);

		let info = StakingInfo {
			annual_inflation_rate: Perbill::from_percent(42),
			collator_payout_cut: Perbill::from_percent(24),
			treasury_payout_cut: Perbill::from_percent(12),
		};

		// regular accounts cannot call
		assert_noop!(
			Pools::set_staking_info(RuntimeOrigin::signed(1), info),
			DispatchError::BadOrigin
		);

		// only ForceOrigin can call
		let caller = RuntimeOrigin::root();
		assert_ok!(Pools::set_staking_info(caller, info));

		// ensure info is set
		assert_eq!(StakingInformation::<Runtime>::get(), Some(info));
	})
}
