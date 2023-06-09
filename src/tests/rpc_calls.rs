use super::{helper::*, mock::*};
use crate::{rpc::ProposalState, Error, Role};
use codec::Encode;
use frame_support::{assert_noop, assert_ok};
use frame_system::RawOrigin;
pub use sp_std::{boxed::Box, mem::size_of};

fn create_supersig(supersig_id: u128) -> sp_runtime::AccountId32 {
	let creator = vec![(ALICE(), Role::Master)].try_into().unwrap();
	assert_ok!(Supersig::create_supersig(
		RawOrigin::Signed(ALICE()).into(),
		creator,
	));
	let supersig_account = get_supersig_account(u64::try_from(supersig_id).unwrap());
	assert_ok!(Balances::transfer(
		RawOrigin::Signed(ALICE()).into(),
		supersig_account.clone(),
		100_000
	));
	assert_ok!(Supersig::add_members(
		RawOrigin::Signed(supersig_account.clone()).into(),
		vec!((BOB(), Role::Standard), (CHARLIE(), Role::Standard)).try_into().unwrap()
	));
	assert_eq!(Supersig::members(supersig_id, ALICE()), Role::Master);
	assert_eq!(Supersig::members(supersig_id, BOB()), Role::Standard);
	assert_eq!(Supersig::members(supersig_id, CHARLIE()), Role::Standard);
	assert_eq!(Supersig::total_members(supersig_id), 3);
	supersig_account
}

#[test]
fn get_account_supersigs() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		let mut supersig_count: u128 = 0;

		assert_eq!(Supersig::get_user_supersigs(&ALICE()), vec![]);

		create_supersig(supersig_count);
		let alice_supersigs = Supersig::get_user_supersigs(&ALICE());
		assert!(alice_supersigs.contains(&0));
		supersig_count += 1;

		create_supersig(supersig_count);
		let alice_supersigs = Supersig::get_user_supersigs(&ALICE());
		assert!(alice_supersigs.contains(&0));
		assert!(alice_supersigs.contains(&1));
		supersig_count += 1;

		create_supersig(supersig_count);
		let alice_supersigs = Supersig::get_user_supersigs(&ALICE());
		assert!(alice_supersigs.contains(&0));
		assert!(alice_supersigs.contains(&1));
		assert!(alice_supersigs.contains(&2));
	})
}

#[test]
fn list_members() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		let supersig_account = create_supersig(0);
		assert!(
			Supersig::list_members(&supersig_account)
				.unwrap()
				.contains(&(ALICE(), Role::Master))
		);
		assert!(
			Supersig::list_members(&supersig_account)
				.unwrap()
				.contains(&(BOB(), Role::Standard))
		);
		assert!(
			Supersig::list_members(&supersig_account)
				.unwrap()
				.contains(&(CHARLIE(), Role::Standard))
		);
	})
}

#[test]
fn get_proposals() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		let supersig_account = create_supersig(0);
		let call: RuntimeCall = frame_system::Call::remark {
			remark: "test".into(),
		}
		.into();

		assert_ok!(Supersig::propose_call(
			RawOrigin::Signed(ALICE()).into(),
			supersig_account.clone(),
			Box::new(call.clone())
		));

		assert_ok!(Supersig::propose_call(
			RawOrigin::Signed(ALICE()).into(),
			supersig_account.clone(),
			Box::new(call.clone())
		));

		assert_ok!(Supersig::approve_call(
			RawOrigin::Signed(BOB()).into(),
			supersig_account.clone(),
			0,
		));

		assert_ok!(Supersig::approve_call(
			RawOrigin::Signed(BOB()).into(),
			supersig_account.clone(),
			1,
		));

		let list = Supersig::list_proposals(&supersig_account).unwrap();
		assert_eq!(list.1, 3);
		assert_eq!(list.0.len(), 2);
		assert!(list.0.contains(&ProposalState::new(0, call.encode(), ALICE(), vec![BOB()])));
		assert!(list.0.contains(&ProposalState::new(1, call.encode(), ALICE(), vec![BOB()])));

		assert_ok!(Supersig::approve_call(
			RawOrigin::Signed(ALICE()).into(),
			supersig_account.clone(),
			1,
		));

		assert_ok!(
			Supersig::list_proposals(&supersig_account),
			(
				vec![ProposalState::new(0, call.encode(), ALICE(), vec![BOB()])],
				3
			)
		);
	})
}

#[test]
fn get_proposal_state() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		let supersig_account = create_supersig(0);
		let call: RuntimeCall = frame_system::Call::remark {
			remark: "test".into(),
		}
		.into();

		assert_noop!(
			Supersig::get_proposal_state(&supersig_account, &0),
			Error::<Test>::CallNotFound
		);

		assert_ok!(Supersig::propose_call(
			RawOrigin::Signed(ALICE()).into(),
			supersig_account.clone(),
			Box::new(call.clone())
		));

		assert_ok!(
			Supersig::get_proposal_state(&supersig_account, &0),
			(ProposalState::new(0, call.encode(), ALICE(), vec![]), 3)
		);

		assert_ok!(Supersig::approve_call(
			RawOrigin::Signed(ALICE()).into(),
			supersig_account.clone(),
			0,
		));

		assert_ok!(
			Supersig::get_proposal_state(&supersig_account, &0),
			(
				ProposalState::new(0, call.encode(), ALICE(), vec![ALICE()]),
				3
			)
		);

		assert_ok!(Supersig::approve_call(
			RawOrigin::Signed(BOB()).into(),
			supersig_account.clone(),
			0,
		));

		assert_noop!(
			Supersig::get_proposal_state(&supersig_account, &0),
			Error::<Test>::CallNotFound
		);
	})
}
