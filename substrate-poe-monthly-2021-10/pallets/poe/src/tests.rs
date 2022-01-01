use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};
use super::*;

// #[test]
// fn it_works_for_default_value() {
// 	new_test_ext().execute_with(|| {
// 		// Dispatch a signed extrinsic.
// 		assert_ok!(TemplateModule::do_something(Origin::signed(1), 42));
// 		// Read pallet storage and assert an expected result.
// 		assert_eq!(TemplateModule::something(), Some(42));
// 	});
// }

// #[test]
// fn correct_error_for_none_value() {
// 	new_test_ext().execute_with(|| {
// 		// Ensure the expected error is thrown when no value is present.
// 		assert_noop!(TemplateModule::cause_error(Origin::signed(1)), Error::<Test>::NoneValue);
// 	});
// }

#[test]
fn create_claim_works() {
    /*
        help: the trait `FnOnce<()>` is not implemented for `()`
        note: wrap the `()` in a closure with no arguments: `|| { /* code */ }`
    */
    new_test_ext().execute_with(||{
        // test claim
        let claim = vec![0, 1];
        // assert that create_claim is working, account 1, u64
        assert_ok!(PoeModule::create_claim(Origin::signed(1), claim.clone()));
        // assert that storagemap on chain(find by claim) is equal to test data here(accountId, block number)
        assert_eq!(
            // need use father module
            Proofs::<Test>::get(&claim),
            Some((1, frame_system::Pallet::<Test>::block_number()))
        );
    })
}

#[test]
fn create_claim_failed_when_claim_already_exist() {
    new_test_ext().execute_with(||{
        let claim = vec![0, 1];
        // create claim for test, so this clain is upload to chain
        let _ = PoeModule::create_claim(Origin::signed(1), claim.clone());
        // exception operate, assert that Error message match here when create existing proof, do not modify chain
        assert_noop!(
            PoeModule::create_claim(Origin::signed(1), claim.clone()),
            Error::<Test>::ProofAlreadyExist
        );
    })
}

#[test]
fn create_claim_failed_when_claim_is_out_of_bound() {
    new_test_ext().execute_with(||{
        // vec with 3 len trigger the error ClaimSizeOutOfBound
        let claim = vec![0, 1, 2];
        assert_noop!(
            PoeModule::create_claim(Origin::signed(1), claim.clone()),
            Error::<Test>::ClaimSizeOutOfBound
        );
    })
}

#[test]
fn revoke_claim_works() {
    new_test_ext().execute_with(||{
        let claim = vec![0, 1];
        let _ = PoeModule::create_claim(Origin::signed(1), claim.clone());
        assert_ok!(PoeModule::revoke_claim(Origin::signed(1), claim.clone()));
        // assert getting no storagemap by the claim
        assert_eq!(Proofs::<Test>::get(&claim), None);
    })
}

#[test]
fn revoke_claim_failed_when_claim_is_not_exist() {
    new_test_ext().execute_with(||{
        let claim = vec![0, 1];
        // exception operate, assert that Error message match here when to revoke nothing 
        assert_noop!(
            PoeModule::revoke_claim(Origin::signed(1), claim.clone()),
            Error::<Test>::ClaimNotExist
        );
    })
}

#[test]
fn transfer_claim_works() {
    new_test_ext().execute_with(||{
        let claim = vec![0, 1];
        let _ = PoeModule::create_claim(Origin::signed(1), claim.clone());
        // account 2
        assert_ok!(PoeModule::transfer_claim(Origin::signed(1), claim.clone(), 2));
        // assert that the claim is owned by not original 1, but account 2
        assert_eq!(
            Proofs::<Test>::get(&claim),
            Some((2, frame_system::Pallet::<Test>::block_number())));
    })
}

#[test]
fn transfer_claim_failed_when_claim_is_not_exist() {
    new_test_ext().execute_with(||{
        let claim = vec![0, 1];
        // same to revoke
        assert_noop!(
            PoeModule::transfer_claim(Origin::signed(1), claim.clone(), 2),
            Error::<Test>::ClaimNotExist
        );
    })
}

#[test]
fn transfer_claim_failed_when_sender_is_dest() {
    new_test_ext().execute_with(||{
        let claim = vec![0, 1];
        let _ = PoeModule::create_claim(Origin::signed(1), claim.clone());
        // exception operate, assert that Error msg is match here when sender is same with dest
        assert_noop!(
            PoeModule::transfer_claim(Origin::signed(1), claim.clone(), 1),
            Error::<Test>::NotDestination
        );
    })
}
