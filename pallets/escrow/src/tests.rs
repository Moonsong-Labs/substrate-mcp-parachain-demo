use crate::{mock::*, Error, Event, Pallet as Escrow};
use mock::{Test, RuntimeOrigin, System, Balances, Timestamp};
use frame::deps::{
    frame_support::{assert_noop, assert_ok},
    sp_core::H256,
    sp_runtime::Percent,
};

#[test]
fn create_escrow_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        
        let buyer = 1;
        let seller = 2;
        let amount = 1000;
        let description = b"Test escrow".to_vec();
        let deadline = 100;
        
        assert_ok!(Escrow::<Test>::create_escrow(
            RuntimeOrigin::signed(buyer),
            seller,
            amount,
            description,
            deadline,
            None,
        ));
        
        System::assert_last_event(
            Event::EscrowCreated {
                escrow_id: get_last_escrow_id(),
                buyer,
                seller,
                amount,
                deadline,
            }
            .into(),
        );
    });
}

#[test]
fn create_escrow_with_arbitrator_works() {
    new_test_ext().execute_with(|| {
        let buyer = 1;
        let seller = 2;
        let arbitrator = 3;
        let amount = 1000;
        
        assert_ok!(Escrow::<Test>::create_escrow(
            RuntimeOrigin::signed(buyer),
            seller,
            amount,
            b"Test escrow".to_vec(),
            100,
            Some(arbitrator),
        ));
        
        let escrow_id = get_last_escrow_id();
        let escrow = Escrow::<Test>::escrows(escrow_id).unwrap();
        assert_eq!(escrow.arbitrator, Some(arbitrator));
    });
}

#[test]
fn cannot_create_self_escrow() {
    new_test_ext().execute_with(|| {
        let buyer = 1;
        
        assert_noop!(
            Escrow::<Test>::create_escrow(
                RuntimeOrigin::signed(buyer),
                buyer,
                1000,
                b"Test".to_vec(),
                100,
                None,
            ),
            Error::<Test>::SelfEscrow
        );
    });
}

#[test]
fn cannot_create_escrow_with_invalid_amount() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Escrow::<Test>::create_escrow(
                RuntimeOrigin::signed(1),
                2,
                5,
                b"Test".to_vec(),
                100,
                None,
            ),
            Error::<Test>::InvalidAmount
        );
        
        assert_noop!(
            Escrow::<Test>::create_escrow(
                RuntimeOrigin::signed(1),
                2,
                2_000_000,
                b"Test".to_vec(),
                100,
                None,
            ),
            Error::<Test>::InvalidAmount
        );
    });
}

#[test]
fn release_funds_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        
        let buyer = 1;
        let seller = 2;
        let amount = 1000;
        
        assert_ok!(Escrow::<Test>::create_escrow(
            RuntimeOrigin::signed(buyer),
            seller,
            amount,
            b"Test".to_vec(),
            100,
            None,
        ));
        
        let escrow_id = get_last_escrow_id();
        let initial_seller_balance = Balances::free_balance(seller);
        
        assert_ok!(Escrow::<Test>::release_funds(
            RuntimeOrigin::signed(buyer),
            escrow_id,
        ));
        
        let final_seller_balance = Balances::free_balance(seller);
        assert_eq!(final_seller_balance, initial_seller_balance + amount);
        
        System::assert_last_event(
            Event::EscrowReleased {
                escrow_id,
                amount,
                platform_fee: 10,
            }
            .into(),
        );
    });
}

#[test]
fn only_buyer_can_release_funds() {
    new_test_ext().execute_with(|| {
        let buyer = 1;
        let seller = 2;
        
        assert_ok!(Escrow::<Test>::create_escrow(
            RuntimeOrigin::signed(buyer),
            seller,
            1000,
            b"Test".to_vec(),
            100,
            None,
        ));
        
        let escrow_id = get_last_escrow_id();
        
        assert_noop!(
            Escrow::<Test>::release_funds(RuntimeOrigin::signed(seller), escrow_id),
            Error::<Test>::NotAuthorized
        );
    });
}

#[test]
fn refund_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        
        let buyer = 1;
        let seller = 2;
        let amount = 1000;
        
        let initial_buyer_balance = Balances::free_balance(buyer);
        
        assert_ok!(Escrow::<Test>::create_escrow(
            RuntimeOrigin::signed(buyer),
            seller,
            amount,
            b"Test".to_vec(),
            100,
            None,
        ));
        
        let escrow_id = get_last_escrow_id();
        
        assert_ok!(Escrow::<Test>::refund(RuntimeOrigin::signed(seller), escrow_id));
        
        let final_buyer_balance = Balances::free_balance(buyer);
        assert_eq!(final_buyer_balance, initial_buyer_balance);
        
        System::assert_last_event(
            Event::EscrowRefunded {
                escrow_id,
                amount,
            }
            .into(),
        );
    });
}

#[test]
fn claim_expired_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        Timestamp::set_timestamp(1);
        
        let buyer = 1;
        let seller = 2;
        let amount = 1000;
        let deadline = 100;
        
        let initial_buyer_balance = Balances::free_balance(buyer);
        
        assert_ok!(Escrow::<Test>::create_escrow(
            RuntimeOrigin::signed(buyer),
            seller,
            amount,
            b"Test".to_vec(),
            deadline,
            None,
        ));
        
        let escrow_id = get_last_escrow_id();
        
        Timestamp::set_timestamp(101);
        
        assert_ok!(Escrow::<Test>::claim_expired(
            RuntimeOrigin::signed(buyer),
            escrow_id,
        ));
        
        let final_buyer_balance = Balances::free_balance(buyer);
        assert_eq!(final_buyer_balance, initial_buyer_balance);
        
        System::assert_last_event(Event::EscrowExpired { escrow_id }.into());
    });
}

#[test]
fn cannot_claim_before_deadline() {
    new_test_ext().execute_with(|| {
        Timestamp::set_timestamp(1);
        
        let buyer = 1;
        let seller = 2;
        
        assert_ok!(Escrow::<Test>::create_escrow(
            RuntimeOrigin::signed(buyer),
            seller,
            1000,
            b"Test".to_vec(),
            100,
            None,
        ));
        
        let escrow_id = get_last_escrow_id();
        
        Timestamp::set_timestamp(50);
        
        assert_noop!(
            Escrow::<Test>::claim_expired(RuntimeOrigin::signed(buyer), escrow_id),
            Error::<Test>::DeadlineNotReached
        );
    });
}

#[test]
fn raise_dispute_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        
        let buyer = 1;
        let seller = 2;
        let arbitrator = 3;
        
        assert_ok!(Escrow::<Test>::create_escrow(
            RuntimeOrigin::signed(buyer),
            seller,
            1000,
            b"Test".to_vec(),
            100,
            Some(arbitrator),
        ));
        
        let escrow_id = get_last_escrow_id();
        
        assert_ok!(Escrow::<Test>::raise_dispute(
            RuntimeOrigin::signed(buyer),
            escrow_id,
            b"Product not as described".to_vec(),
        ));
        
        System::assert_last_event(
            Event::DisputeRaised {
                escrow_id,
                initiator: buyer,
            }
            .into(),
        );
        
        let escrow = Escrow::<Test>::escrows(escrow_id).unwrap();
        assert_eq!(escrow.status, crate::EscrowStatus::Disputed);
    });
}

#[test]
fn cannot_dispute_without_arbitrator() {
    new_test_ext().execute_with(|| {
        let buyer = 1;
        let seller = 2;
        
        assert_ok!(Escrow::<Test>::create_escrow(
            RuntimeOrigin::signed(buyer),
            seller,
            1000,
            b"Test".to_vec(),
            100,
            None,
        ));
        
        let escrow_id = get_last_escrow_id();
        
        assert_noop!(
            Escrow::<Test>::raise_dispute(
                RuntimeOrigin::signed(buyer),
                escrow_id,
                b"Issue".to_vec(),
            ),
            Error::<Test>::NoArbitratorAssigned
        );
    });
}

#[test]
fn resolve_dispute_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        
        let buyer = 1;
        let seller = 2;
        let arbitrator = 3;
        let amount = 1000;
        
        assert_ok!(Escrow::<Test>::create_escrow(
            RuntimeOrigin::signed(buyer),
            seller,
            amount,
            b"Test".to_vec(),
            100,
            Some(arbitrator),
        ));
        
        let escrow_id = get_last_escrow_id();
        
        assert_ok!(Escrow::<Test>::raise_dispute(
            RuntimeOrigin::signed(buyer),
            escrow_id,
            b"Issue".to_vec(),
        ));
        
        let initial_seller_balance = Balances::free_balance(seller);
        
        assert_ok!(Escrow::<Test>::resolve_dispute(
            RuntimeOrigin::signed(arbitrator),
            escrow_id,
            seller,
        ));
        
        let final_seller_balance = Balances::free_balance(seller);
        assert_eq!(final_seller_balance, initial_seller_balance + amount);
        
        System::assert_last_event(
            Event::DisputeResolved {
                escrow_id,
                winner: seller,
                amount,
            }
            .into(),
        );
    });
}

#[test]
fn set_arbitrator_fee_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        
        let arbitrator = 3;
        let fee_percent = Percent::from_percent(2);
        
        assert_ok!(Escrow::<Test>::set_arbitrator_fee(
            RuntimeOrigin::signed(arbitrator),
            fee_percent,
        ));
        
        assert_eq!(Escrow::<Test>::arbitrator_fees(arbitrator), fee_percent);
        
        System::assert_last_event(
            Event::ArbitratorFeeSet {
                arbitrator,
                fee_percent,
            }
            .into(),
        );
    });
}

fn get_last_escrow_id() -> H256 {
    use frame::deps::sp_runtime::traits::Hash;
    let count = Escrow::<Test>::escrow_count();
    <Test as frame_system::Config>::Hashing::hash_of(&(1u64, 2u64, count))
}