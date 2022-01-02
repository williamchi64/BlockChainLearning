use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

// create=========================================================
#[test]
fn create_kitty_works() {
    new_test_ext().execute_with( || {
        assert_ok!(KittiesTest::create_kitty(Origin::signed(1)));
        assert_eq!(KittiesTest::kitties_count(), 1);
        assert_eq!(KittiesTest::owner(0), 1);
    });
}
#[test]
fn create_kitty_failed_when_kitties_count_is_out_of_limit() {
    new_test_ext().execute_with( || {
        assert_ok!(KittiesTest::create_kitty(Origin::signed(2)));
        assert_ok!(KittiesTest::create_kitty(Origin::signed(1)));
        assert_ok!(KittiesTest::create_kitty(Origin::signed(1)));
        assert_ok!(KittiesTest::transfer(Origin::signed(2), 1, 0));
        assert_noop!(
            KittiesTest::create_kitty(Origin::signed(1)), 
            Error::<Test>::InvalidKittyIndex
        );
    });
}
#[test]
fn create_kitty_failed_when_sender_has_less_balance() {
    new_test_ext().execute_with( || {
        assert_noop!(
            KittiesTest::create_kitty(Origin::signed(3)),
            Error::<Test>::NotEnoughBalance
        );
    });
}
// not test KittiesCountOverflow
// set price=========================================================
#[test]
fn set_price_works() {
    new_test_ext().execute_with( || {
        assert_ok!(KittiesTest::create_kitty(Origin::signed(1)));
        assert_ok!(KittiesTest::set_price(Origin::signed(1), 0, Some(10)));
    });
}
#[test]
fn set_price_failed_when_sender_is_not_owner() {
    new_test_ext().execute_with( || {
        assert_ok!(KittiesTest::create_kitty(Origin::signed(1)));
        assert_noop!(
            KittiesTest::set_price(Origin::signed(2), 0, Some(10)), 
            Error::<Test>::NotKittyOwner
        );
    });
}
#[test]
fn set_price_failed_when_invalid_kitty_id() {
    new_test_ext().execute_with( || {
        assert_ok!(KittiesTest::create_kitty(Origin::signed(1)));
        assert_noop!(
            KittiesTest::set_price(Origin::signed(1), 1, Some(10)), 
            Error::<Test>::KittyNotExist
        );
    });
}
// transfer=========================================================
#[test]
fn transfer_works() {
    new_test_ext().execute_with( || {
        assert_ok!(KittiesTest::create_kitty(Origin::signed(1)));
        assert_ok!(KittiesTest::transfer(Origin::signed(1), 2, 0));
        assert_eq!(KittiesTest::owner(0), 2);
    });
}
#[test]
fn transfer_failed_when_sender_is_not_owner() {
    new_test_ext().execute_with( || {
        assert_ok!(KittiesTest::create_kitty(Origin::signed(1)));
        assert_noop!(
            KittiesTest::transfer(Origin::signed(2), 3, 0), 
            Error::<Test>::NotKittyOwner
        );
    });
}
#[test]
fn transfer_failed_when_transfer_to_self() {
    new_test_ext().execute_with( || {
        assert_ok!(KittiesTest::create_kitty(Origin::signed(1)));
        assert_noop!(
            KittiesTest::transfer(Origin::signed(1), 1, 0), 
            Error::<Test>::TransferToSelf
        );
    });
}
#[test]
fn transfer_failed_when_kitties_is_out_of_limit() {
    new_test_ext().execute_with( || {
        assert_ok!(KittiesTest::create_kitty(Origin::signed(2)));
        assert_ok!(KittiesTest::create_kitty(Origin::signed(1)));
        assert_ok!(KittiesTest::create_kitty(Origin::signed(1)));
        assert_ok!(KittiesTest::create_kitty(Origin::signed(1)));
        assert_noop!(
            KittiesTest::transfer(Origin::signed(2), 1, 0), 
            Error::<Test>::ExceedMaxKittyOwned
        );
    });
}
#[test]
fn transfer_failed_when_invalid_kitty_id() {
    new_test_ext().execute_with( || {
        assert_ok!(KittiesTest::create_kitty(Origin::signed(1)));
        assert_noop!(
            KittiesTest::transfer(Origin::signed(1), 2, 1), 
            Error::<Test>::KittyNotExist
        );
    });
}
// buy kitty=========================================================
#[test]
fn buy_kitty_works() {
    new_test_ext().execute_with( || {
        assert_ok!(KittiesTest::create_kitty(Origin::signed(1)));
        assert_ok!(KittiesTest::set_price(Origin::signed(1), 0, Some(10)));
        assert_ok!(KittiesTest::buy_kitty(Origin::signed(2), 0, 11));
        assert_eq!(KittiesTest::owner(0), 2);
    });
}
#[test]
fn buy_kitty_failed_when_invalid_kitty_id() {
    new_test_ext().execute_with( || {
        assert_ok!(KittiesTest::create_kitty(Origin::signed(1)));
        assert_ok!(KittiesTest::set_price(Origin::signed(1), 0, Some(10)));
        assert_noop!(
            KittiesTest::buy_kitty(Origin::signed(2), 1, 11), 
            Error::<Test>::KittyNotExist
        );
    });
}
#[test]
fn buy_kitty_failed_when_buyer_is_owner() {
    new_test_ext().execute_with( || {
        assert_ok!(KittiesTest::create_kitty(Origin::signed(1)));
        assert_ok!(KittiesTest::set_price(Origin::signed(1), 0, Some(10)));
        assert_noop!(
            KittiesTest::buy_kitty(Origin::signed(1), 0, 11), 
            Error::<Test>::BuyerIsKittyOwner
        );
    });
}
#[test]
fn buy_kitty_failed_when_bid_price_too_low() {
    new_test_ext().execute_with( || {
        assert_ok!(KittiesTest::create_kitty(Origin::signed(1)));
        assert_ok!(KittiesTest::set_price(Origin::signed(1), 0, Some(10)));
        assert_noop!(
            KittiesTest::buy_kitty(Origin::signed(2), 0, 9), 
            Error::<Test>::KittyBidPriceTooLow
        );
    });
}
#[test]
fn buy_kitty_failed_when_kitty_not_for_sale() {
    new_test_ext().execute_with( || {
        assert_ok!(KittiesTest::create_kitty(Origin::signed(1)));
        assert_noop!(
            KittiesTest::buy_kitty(Origin::signed(2), 0, 10), 
            Error::<Test>::KittyNotForSale
        );
    });
}
#[test]
fn buy_kitty_failed_when_not_enough_balance() {
    new_test_ext().execute_with( || {
        assert_ok!(KittiesTest::create_kitty(Origin::signed(1)));
        assert_ok!(KittiesTest::set_price(Origin::signed(1), 0, Some(100)));
        assert_noop!(
            KittiesTest::buy_kitty(Origin::signed(3), 0, 10000000), 
            Error::<Test>::NotEnoughBalance
        );
    });
}
#[test]
fn buy_kitty_failed_when_buyer_kitty_over_limited() {
    new_test_ext().execute_with( || {
        assert_ok!(KittiesTest::create_kitty(Origin::signed(2)));
        assert_ok!(KittiesTest::create_kitty(Origin::signed(1)));
        assert_ok!(KittiesTest::create_kitty(Origin::signed(1)));
        assert_ok!(KittiesTest::create_kitty(Origin::signed(1)));
        assert_ok!(KittiesTest::set_price(Origin::signed(2), 0, Some(10)));
        assert_noop!(
            KittiesTest::buy_kitty(Origin::signed(1), 0, 11), 
            Error::<Test>::ExceedMaxKittyOwned
        );
    });
}
// breed kitty===========================================================
#[test]
fn breed_kitty_works() {
    new_test_ext().execute_with( || {
        assert_ok!(KittiesTest::create_kitty(Origin::signed(1)));
        assert_ok!(KittiesTest::create_kitty(Origin::signed(1)));
        assert_ok!(KittiesTest::breed_kitty(Origin::signed(1), 0, 1));
        assert_eq!(KittiesTest::kitties_owned(1).len(), 3);
    });
}
#[test]
fn breed_kitty_failed_when_parent_is_not_owned_by_sender() {
    new_test_ext().execute_with( || {
        assert_ok!(KittiesTest::create_kitty(Origin::signed(1)));
        assert_ok!(KittiesTest::create_kitty(Origin::signed(1)));
        assert_noop!(
            KittiesTest::breed_kitty(Origin::signed(2), 0, 1), 
            Error::<Test>::NotKittyOwner);
    });
}
#[test]
fn breed_kitty_failed_when_kitties_is_out_of_limit() {
    new_test_ext().execute_with( || {
        assert_ok!(KittiesTest::create_kitty(Origin::signed(1)));
        assert_ok!(KittiesTest::create_kitty(Origin::signed(1)));
        assert_ok!(KittiesTest::create_kitty(Origin::signed(1)));
        assert_noop!(
            KittiesTest::breed_kitty(Origin::signed(1), 0, 1), 
            Error::<Test>::ExceedMaxKittyOwned);
    });
}
