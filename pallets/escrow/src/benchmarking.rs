//! Benchmarking setup for pallet-escrow

use super::*;

#[allow(unused)]
use crate::Pallet as Escrow;
use frame_benchmarking::{benchmarks, whitelisted_caller, account};
use frame_system::RawOrigin;
use frame_support::traits::Currency;
use sp_std::vec;

const SEED: u32 = 0;

fn create_funded_user<T: Config>(
    string: &'static str,
    n: u32,
    balance_factor: u32,
) -> T::AccountId {
    let user = account(string, n, SEED);
    let balance = T::Currency::minimum_balance() * balance_factor.into();
    T::Currency::make_free_balance_be(&user, balance);
    user
}

benchmarks! {
    create_escrow {
        let buyer: T::AccountId = whitelisted_caller();
        let seller = create_funded_user::<T>("seller", 0, 100);
        let amount = <T as Config>::MinEscrow::get();
        let deadline = <frame_system::Pallet<T>>::block_number() + <T as Config>::MinDeadline::get() + 100u32.into();
        let description = vec![b'x'; 100];
        
        // Fund the buyer
        T::Currency::make_free_balance_be(&buyer, amount * 10u32.into());
        
    }: _(RawOrigin::Signed(buyer.clone()), seller.clone(), amount, deadline, description)
    verify {
        assert_eq!(Escrows::<T>::get(0).unwrap().buyer, buyer);
        assert_eq!(Escrows::<T>::get(0).unwrap().seller, seller);
    }

    release_escrow {
        let buyer: T::AccountId = whitelisted_caller();
        let seller = create_funded_user::<T>("seller", 0, 100);
        let amount = <T as Config>::MinEscrow::get();
        let deadline = <frame_system::Pallet<T>>::block_number() + <T as Config>::MinDeadline::get() + 100u32.into();
        let description = vec![b'x'; 100];
        
        // Fund the buyer and create escrow
        T::Currency::make_free_balance_be(&buyer, amount * 10u32.into());
        Escrow::<T>::create_escrow(
            RawOrigin::Signed(buyer.clone()).into(),
            seller.clone(),
            amount,
            deadline,
            description
        )?;
        
        let escrow_id = 0u64;
        
    }: _(RawOrigin::Signed(buyer), escrow_id)
    verify {
        assert_eq!(Escrows::<T>::get(escrow_id).unwrap().status, EscrowStatus::Released);
    }

    refund_escrow {
        let buyer = create_funded_user::<T>("buyer", 0, 100);
        let seller: T::AccountId = whitelisted_caller();
        let amount = <T as Config>::MinEscrow::get();
        let deadline = <frame_system::Pallet<T>>::block_number() + <T as Config>::MinDeadline::get() + 100u32.into();
        let description = vec![b'x'; 100];
        
        // Create escrow
        Escrow::<T>::create_escrow(
            RawOrigin::Signed(buyer.clone()).into(),
            seller.clone(),
            amount,
            deadline,
            description
        )?;
        
        let escrow_id = 0u64;
        
    }: _(RawOrigin::Signed(seller), escrow_id)
    verify {
        assert_eq!(Escrows::<T>::get(escrow_id).unwrap().status, EscrowStatus::Refunded);
    }

    reclaim_escrow {
        let buyer: T::AccountId = whitelisted_caller();
        let seller = create_funded_user::<T>("seller", 0, 100);
        let amount = <T as Config>::MinEscrow::get();
        let deadline = <frame_system::Pallet<T>>::block_number() + <T as Config>::MinDeadline::get() + 1u32.into();
        let description = vec![b'x'; 100];
        
        // Fund the buyer and create escrow
        T::Currency::make_free_balance_be(&buyer, amount * 10u32.into());
        Escrow::<T>::create_escrow(
            RawOrigin::Signed(buyer.clone()).into(),
            seller.clone(),
            amount,
            deadline,
            description
        )?;
        
        // Move past deadline
        frame_system::Pallet::<T>::set_block_number(deadline + 1u32.into());
        
        let escrow_id = 0u64;
        
    }: _(RawOrigin::Signed(buyer), escrow_id)
    verify {
        assert_eq!(Escrows::<T>::get(escrow_id).unwrap().status, EscrowStatus::Reclaimed);
    }

    set_fee {
        let new_fee = 100u16; // 1%
        
    }: _(RawOrigin::Root, new_fee)
    verify {
        assert_eq!(PlatformFee::<T>::get(), new_fee);
    }

    impl_benchmark_test_suite!(Escrow, crate::mock::new_test_ext(), crate::mock::Test);
}