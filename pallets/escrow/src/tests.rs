use crate::{mock::*, Error, Event};
use frame_support::{assert_noop, assert_ok};

#[test]
fn create_escrow_works() {
    new_test_ext().execute_with(|| {
        let buyer = 1;
        let seller = 2;
        let amount = 1000;
        let deadline = System::block_number() + MinDeadline::get() + 100;
        let description = b"Test escrow".to_vec();
        
        // Check initial balance
        assert_eq!(Balances::free_balance(buyer), 100_000);
        
        // Create escrow
        assert_ok!(EscrowPallet::create_escrow(
            RuntimeOrigin::signed(buyer),
            seller,
            amount,
            deadline,
            description.clone()
        ));
        
        // Check escrow created
        let escrow = EscrowPallet::escrows(0).unwrap();
        assert_eq!(escrow.buyer, buyer);
        assert_eq!(escrow.seller, seller);
        assert_eq!(escrow.amount, amount);
        assert_eq!(escrow.deadline, deadline);
        assert_eq!(escrow.description.to_vec(), description);
        assert_eq!(escrow.status, crate::EscrowStatus::Pending);
        
        // Check funds transferred
        assert_eq!(Balances::free_balance(buyer), 99_000);
        assert_eq!(Balances::free_balance(EscrowPallet::account_id()), 1000);
        
        // Check event emitted
        System::assert_last_event(
            Event::EscrowCreated {
                escrow_id: 0,
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
fn create_escrow_fails_with_self() {
    new_test_ext().execute_with(|| {
        let buyer = 1;
        let amount = 1000;
        let deadline = System::block_number() + MinDeadline::get() + 100;
        let description = b"Test escrow".to_vec();
        
        assert_noop!(
            EscrowPallet::create_escrow(
                RuntimeOrigin::signed(buyer),
                buyer, // Same as buyer
                amount,
                deadline,
                description
            ),
            Error::<Test>::CannotEscrowToSelf
        );
    });
}

#[test]
fn create_escrow_fails_with_invalid_amount() {
    new_test_ext().execute_with(|| {
        let buyer = 1;
        let seller = 2;
        let deadline = System::block_number() + MinDeadline::get() + 100;
        let description = b"Test escrow".to_vec();
        
        // Amount too small
        assert_noop!(
            EscrowPallet::create_escrow(
                RuntimeOrigin::signed(buyer),
                seller,
                5, // Less than MinEscrow
                deadline,
                description.clone()
            ),
            Error::<Test>::AmountOutOfBounds
        );
        
        // Amount too large
        assert_noop!(
            EscrowPallet::create_escrow(
                RuntimeOrigin::signed(buyer),
                seller,
                2_000_000, // More than MaxEscrow
                deadline,
                description
            ),
            Error::<Test>::AmountOutOfBounds
        );
    });
}

#[test]
fn create_escrow_fails_with_invalid_deadline() {
    new_test_ext().execute_with(|| {
        let buyer = 1;
        let seller = 2;
        let amount = 1000;
        let description = b"Test escrow".to_vec();
        
        // Deadline too soon
        assert_noop!(
            EscrowPallet::create_escrow(
                RuntimeOrigin::signed(buyer),
                seller,
                amount,
                System::block_number() + 5, // Less than MinDeadline (which is 10)
                description.clone()
            ),
            Error::<Test>::DeadlineOutOfBounds
        );
        
        // Deadline too far
        assert_noop!(
            EscrowPallet::create_escrow(
                RuntimeOrigin::signed(buyer),
                seller,
                amount,
                System::block_number() + 20_000, // More than MaxDeadline (which is 10_000)
                description
            ),
            Error::<Test>::DeadlineOutOfBounds
        );
    });
}

#[test]
fn create_escrow_fails_with_long_description() {
    new_test_ext().execute_with(|| {
        let buyer = 1;
        let seller = 2;
        let amount = 1000;
        let deadline = System::block_number() + MinDeadline::get() + 100;
        let description = vec![b'a'; 501]; // More than MaxDescriptionLen
        
        assert_noop!(
            EscrowPallet::create_escrow(
                RuntimeOrigin::signed(buyer),
                seller,
                amount,
                deadline,
                description
            ),
            Error::<Test>::DescriptionTooLong
        );
    });
}

#[test]
fn create_escrow_fails_with_max_active_escrows() {
    new_test_ext().execute_with(|| {
        let buyer = 1;
        let seller = 2;
        let amount = 10;
        let deadline = System::block_number() + MinDeadline::get() + 100;
        let description = b"Test".to_vec();
        
        // Create maximum allowed escrows
        for i in 0..MaxActiveEscrows::get() {
            assert_ok!(EscrowPallet::create_escrow(
                RuntimeOrigin::signed(buyer),
                seller,
                amount,
                deadline + i as u64,
                description.clone()
            ));
        }
        
        // Try to create one more
        assert_noop!(
            EscrowPallet::create_escrow(
                RuntimeOrigin::signed(buyer),
                seller,
                amount,
                deadline,
                description
            ),
            Error::<Test>::MaxActiveEscrowsReached
        );
    });
}

#[test]
fn release_escrow_works() {
    new_test_ext().execute_with(|| {
        let buyer = 1;
        let seller = 2;
        let amount = 1000;
        let deadline = System::block_number() + MinDeadline::get() + 100;
        let description = b"Test escrow".to_vec();
        
        // Create escrow
        assert_ok!(EscrowPallet::create_escrow(
            RuntimeOrigin::signed(buyer),
            seller,
            amount,
            deadline,
            description
        ));
        
        // Release escrow
        assert_ok!(EscrowPallet::release_escrow(
            RuntimeOrigin::signed(buyer),
            0
        ));
        
        // Check escrow status updated
        let escrow = EscrowPallet::escrows(0).unwrap();
        assert_eq!(escrow.status, crate::EscrowStatus::Released);
        
        // Check funds transferred with fee
        let fee = amount * EscrowPallet::platform_fee() as u128 / 10000;
        let amount_to_seller = amount - fee;
        assert_eq!(Balances::free_balance(seller), 100_000 + amount_to_seller);
        
        // Check event emitted
        System::assert_last_event(
            Event::EscrowReleased {
                escrow_id: 0,
                buyer,
                seller,
                amount: amount_to_seller,
                fee,
            }
            .into(),
        );
        
        // Check escrow removed from active list
        assert_eq!(EscrowPallet::active_by_user(buyer).len(), 0);
    });
}

#[test]
fn release_escrow_fails_if_not_buyer() {
    new_test_ext().execute_with(|| {
        let buyer = 1;
        let seller = 2;
        let amount = 1000;
        let deadline = System::block_number() + MinDeadline::get() + 100;
        let description = b"Test escrow".to_vec();
        
        // Create escrow
        assert_ok!(EscrowPallet::create_escrow(
            RuntimeOrigin::signed(buyer),
            seller,
            amount,
            deadline,
            description
        ));
        
        // Try to release as seller
        assert_noop!(
            EscrowPallet::release_escrow(
                RuntimeOrigin::signed(seller),
                0
            ),
            Error::<Test>::NotBuyer
        );
        
        // Try to release as other account
        assert_noop!(
            EscrowPallet::release_escrow(
                RuntimeOrigin::signed(3),
                0
            ),
            Error::<Test>::NotBuyer
        );
    });
}

#[test]
fn release_escrow_fails_if_not_pending() {
    new_test_ext().execute_with(|| {
        let buyer = 1;
        let seller = 2;
        let amount = 1000;
        let deadline = System::block_number() + MinDeadline::get() + 100;
        let description = b"Test escrow".to_vec();
        
        // Create escrow
        assert_ok!(EscrowPallet::create_escrow(
            RuntimeOrigin::signed(buyer),
            seller,
            amount,
            deadline,
            description
        ));
        
        // Release escrow
        assert_ok!(EscrowPallet::release_escrow(
            RuntimeOrigin::signed(buyer),
            0
        ));
        
        // Try to release again
        assert_noop!(
            EscrowPallet::release_escrow(
                RuntimeOrigin::signed(buyer),
                0
            ),
            Error::<Test>::EscrowNotPending
        );
    });
}

#[test]
fn refund_escrow_works() {
    new_test_ext().execute_with(|| {
        let buyer = 1;
        let seller = 2;
        let amount = 1000;
        let deadline = System::block_number() + MinDeadline::get() + 100;
        let description = b"Test escrow".to_vec();
        
        // Create escrow
        assert_ok!(EscrowPallet::create_escrow(
            RuntimeOrigin::signed(buyer),
            seller,
            amount,
            deadline,
            description
        ));
        
        // Refund escrow (by seller)
        assert_ok!(EscrowPallet::refund_escrow(
            RuntimeOrigin::signed(seller),
            0
        ));
        
        // Check escrow status updated
        let escrow = EscrowPallet::escrows(0).unwrap();
        assert_eq!(escrow.status, crate::EscrowStatus::Refunded);
        
        // Check full refund (no fee)
        assert_eq!(Balances::free_balance(buyer), 100_000);
        assert_eq!(Balances::free_balance(seller), 100_000);
        
        // Check event emitted
        System::assert_last_event(
            Event::EscrowRefunded {
                escrow_id: 0,
                buyer,
                seller,
                amount,
            }
            .into(),
        );
        
        // Check escrow removed from active list
        assert_eq!(EscrowPallet::active_by_user(buyer).len(), 0);
    });
}

#[test]
fn refund_escrow_fails_if_not_seller() {
    new_test_ext().execute_with(|| {
        let buyer = 1;
        let seller = 2;
        let amount = 1000;
        let deadline = System::block_number() + MinDeadline::get() + 100;
        let description = b"Test escrow".to_vec();
        
        // Create escrow
        assert_ok!(EscrowPallet::create_escrow(
            RuntimeOrigin::signed(buyer),
            seller,
            amount,
            deadline,
            description
        ));
        
        // Try to refund as buyer
        assert_noop!(
            EscrowPallet::refund_escrow(
                RuntimeOrigin::signed(buyer),
                0
            ),
            Error::<Test>::NotSeller
        );
        
        // Try to refund as other account
        assert_noop!(
            EscrowPallet::refund_escrow(
                RuntimeOrigin::signed(3),
                0
            ),
            Error::<Test>::NotSeller
        );
    });
}

#[test]
fn reclaim_escrow_works() {
    new_test_ext().execute_with(|| {
        let buyer = 1;
        let seller = 2;
        let amount = 1000;
        let deadline = System::block_number() + MinDeadline::get() + 100;
        let description = b"Test escrow".to_vec();
        
        // Create escrow
        assert_ok!(EscrowPallet::create_escrow(
            RuntimeOrigin::signed(buyer),
            seller,
            amount,
            deadline,
            description
        ));
        
        // Move past deadline
        run_to_block(deadline + 1);
        
        // Reclaim escrow
        assert_ok!(EscrowPallet::reclaim_escrow(
            RuntimeOrigin::signed(buyer),
            0
        ));
        
        // Check escrow status updated
        let escrow = EscrowPallet::escrows(0).unwrap();
        assert_eq!(escrow.status, crate::EscrowStatus::Reclaimed);
        
        // Check full refund (no fee)
        assert_eq!(Balances::free_balance(buyer), 100_000);
        
        // Check event emitted
        System::assert_last_event(
            Event::EscrowReclaimed {
                escrow_id: 0,
                buyer,
                amount,
            }
            .into(),
        );
        
        // Check escrow removed from active list
        assert_eq!(EscrowPallet::active_by_user(buyer).len(), 0);
    });
}

#[test]
fn reclaim_escrow_fails_before_deadline() {
    new_test_ext().execute_with(|| {
        let buyer = 1;
        let seller = 2;
        let amount = 1000;
        let deadline = System::block_number() + MinDeadline::get() + 100;
        let description = b"Test escrow".to_vec();
        
        // Create escrow
        assert_ok!(EscrowPallet::create_escrow(
            RuntimeOrigin::signed(buyer),
            seller,
            amount,
            deadline,
            description
        ));
        
        // Try to reclaim before deadline
        assert_noop!(
            EscrowPallet::reclaim_escrow(
                RuntimeOrigin::signed(buyer),
                0
            ),
            Error::<Test>::DeadlineNotPassed
        );
    });
}

#[test]
fn reclaim_escrow_fails_if_not_buyer() {
    new_test_ext().execute_with(|| {
        let buyer = 1;
        let seller = 2;
        let amount = 1000;
        let deadline = System::block_number() + MinDeadline::get() + 100;
        let description = b"Test escrow".to_vec();
        
        // Create escrow
        assert_ok!(EscrowPallet::create_escrow(
            RuntimeOrigin::signed(buyer),
            seller,
            amount,
            deadline,
            description
        ));
        
        // Move past deadline
        run_to_block(deadline + 1);
        
        // Try to reclaim as seller
        assert_noop!(
            EscrowPallet::reclaim_escrow(
                RuntimeOrigin::signed(seller),
                0
            ),
            Error::<Test>::NotBuyer
        );
    });
}

#[test]
fn set_fee_works() {
    new_test_ext().execute_with(|| {
        // Check initial fee
        assert_eq!(EscrowPallet::platform_fee(), 50); // 0.5%
        
        // Set new fee (as root)
        assert_ok!(EscrowPallet::set_fee(
            RuntimeOrigin::root(),
            100 // 1%
        ));
        
        // Check fee updated
        assert_eq!(EscrowPallet::platform_fee(), 100);
        
        // Check event emitted
        System::assert_last_event(
            Event::PlatformFeeChanged {
                old_fee: 50,
                new_fee: 100,
            }
            .into(),
        );
    });
}

#[test]
fn set_fee_fails_if_not_governance() {
    new_test_ext().execute_with(|| {
        // Try to set fee as regular user
        assert_noop!(
            EscrowPallet::set_fee(
                RuntimeOrigin::signed(1),
                100
            ),
            sp_runtime::DispatchError::BadOrigin
        );
    });
}

#[test]
fn set_fee_fails_with_invalid_bounds() {
    new_test_ext().execute_with(|| {
        // Fee too low
        assert_noop!(
            EscrowPallet::set_fee(
                RuntimeOrigin::root(),
                40 // Less than 0.5%
            ),
            Error::<Test>::FeeOutOfBounds
        );
        
        // Fee too high
        assert_noop!(
            EscrowPallet::set_fee(
                RuntimeOrigin::root(),
                250 // More than 2%
            ),
            Error::<Test>::FeeOutOfBounds
        );
    });
}

#[test]
fn fee_calculation_works() {
    new_test_ext().execute_with(|| {
        let buyer = 1;
        let seller = 2;
        let amount = 10000;
        let deadline = System::block_number() + MinDeadline::get() + 100;
        let description = b"Test escrow".to_vec();
        
        // Set fee to 1.5% (150 basis points)
        assert_ok!(EscrowPallet::set_fee(RuntimeOrigin::root(), 150));
        
        // Create escrow
        assert_ok!(EscrowPallet::create_escrow(
            RuntimeOrigin::signed(buyer),
            seller,
            amount,
            deadline,
            description
        ));
        
        // Release escrow
        assert_ok!(EscrowPallet::release_escrow(
            RuntimeOrigin::signed(buyer),
            0
        ));
        
        // Check fee calculation (1.5% of 10000 = 150)
        let expected_fee = 150;
        let expected_amount_to_seller = amount - expected_fee;
        assert_eq!(Balances::free_balance(seller), 100_000 + expected_amount_to_seller);
    });
}

#[test]
fn escrow_not_found_error() {
    new_test_ext().execute_with(|| {
        // Try to release non-existent escrow
        assert_noop!(
            EscrowPallet::release_escrow(
                RuntimeOrigin::signed(1),
                999
            ),
            Error::<Test>::EscrowNotFound
        );
        
        // Try to refund non-existent escrow
        assert_noop!(
            EscrowPallet::refund_escrow(
                RuntimeOrigin::signed(1),
                999
            ),
            Error::<Test>::EscrowNotFound
        );
        
        // Try to reclaim non-existent escrow
        assert_noop!(
            EscrowPallet::reclaim_escrow(
                RuntimeOrigin::signed(1),
                999
            ),
            Error::<Test>::EscrowNotFound
        );
    });
}