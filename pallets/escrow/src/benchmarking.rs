#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame::deps::{
    frame_benchmarking::{account, benchmarks, whitelisted_caller},
    frame_system::RawOrigin,
    sp_runtime::traits::Bounded,
};

const SEED: u32 = 0;

fn create_funded_user<T: Config>(name: &'static str, index: u32) -> T::AccountId {
    let user = account(name, index, SEED);
    let balance = T::Currency::minimum_balance() * 1_000_000u32.into();
    let _ = T::Currency::make_free_balance_be(&user, balance);
    user
}

fn create_test_escrow<T: Config>(buyer: &T::AccountId, seller: &T::AccountId) -> T::Hash {
    let amount = T::MinimumEscrowAmount::get() * 10u32.into();
    let deadline = T::Time::now() + T::MinimumDeadline::get() * 2u32.into();
    let description = frame::deps::sp_std::vec![b'x'; 100];
    
    let _ = Pallet::<T>::create_escrow(
        RawOrigin::Signed(buyer.clone()).into(),
        seller.clone(),
        amount,
        description,
        deadline,
        None,
    );
    
    let count = EscrowCount::<T>::get();
    T::Hashing::hash_of(&(buyer, seller, count))
}

benchmarks! {
    create_escrow {
        let buyer = create_funded_user::<T>("buyer", 1);
        let seller = create_funded_user::<T>("seller", 2);
        let amount = T::MinimumEscrowAmount::get() * 10u32.into();
        let deadline = T::Time::now() + T::MinimumDeadline::get() * 2u32.into();
        let description = frame::deps::sp_std::vec![b'x'; T::MaxDescriptionLength::get() as usize];
        
    }: _(RawOrigin::Signed(buyer.clone()), seller, amount, description, deadline, None)
    verify {
        assert_eq!(EscrowCount::<T>::get(), 1);
    }
    
    release_funds {
        let buyer = create_funded_user::<T>("buyer", 1);
        let seller = create_funded_user::<T>("seller", 2);
        let escrow_id = create_test_escrow::<T>(&buyer, &seller);
        
    }: _(RawOrigin::Signed(buyer), escrow_id)
    verify {
        let escrow = Escrows::<T>::get(escrow_id).unwrap();
        assert_eq!(escrow.status, EscrowStatus::Released);
    }
    
    refund {
        let buyer = create_funded_user::<T>("buyer", 1);
        let seller = create_funded_user::<T>("seller", 2);
        let escrow_id = create_test_escrow::<T>(&buyer, &seller);
        
    }: _(RawOrigin::Signed(seller), escrow_id)
    verify {
        let escrow = Escrows::<T>::get(escrow_id).unwrap();
        assert_eq!(escrow.status, EscrowStatus::Refunded);
    }
    
    claim_expired {
        let buyer = create_funded_user::<T>("buyer", 1);
        let seller = create_funded_user::<T>("seller", 2);
        
        let amount = T::MinimumEscrowAmount::get() * 10u32.into();
        let deadline = T::Time::now() + T::MinimumDeadline::get();
        let description = frame::deps::sp_std::vec![b'x'; 100];
        
        let _ = Pallet::<T>::create_escrow(
            RawOrigin::Signed(buyer.clone()).into(),
            seller.clone(),
            amount,
            description,
            deadline,
            None,
        );
        
        let count = EscrowCount::<T>::get();
        let escrow_id = T::Hashing::hash_of(&(&buyer, &seller, count));
        
        frame::deps::frame_benchmarking::benchmarking::add_to_whitelist(escrow_id.as_ref().into());
        
    }: _(RawOrigin::Signed(buyer), escrow_id)
    verify {
        let escrow = Escrows::<T>::get(escrow_id).unwrap();
        assert_eq!(escrow.status, EscrowStatus::Expired);
    }
    
    raise_dispute {
        let buyer = create_funded_user::<T>("buyer", 1);
        let seller = create_funded_user::<T>("seller", 2);
        let arbitrator = create_funded_user::<T>("arbitrator", 3);
        
        let amount = T::MinimumEscrowAmount::get() * 10u32.into();
        let deadline = T::Time::now() + T::MinimumDeadline::get() * 2u32.into();
        let description = frame::deps::sp_std::vec![b'x'; 100];
        
        let _ = Pallet::<T>::create_escrow(
            RawOrigin::Signed(buyer.clone()).into(),
            seller.clone(),
            amount,
            description,
            deadline,
            Some(arbitrator),
        );
        
        let count = EscrowCount::<T>::get();
        let escrow_id = T::Hashing::hash_of(&(&buyer, &seller, count));
        
        let reason = frame::deps::sp_std::vec![b'x'; T::MaxDisputeReasonLength::get() as usize];
        
    }: _(RawOrigin::Signed(buyer), escrow_id, reason)
    verify {
        let escrow = Escrows::<T>::get(escrow_id).unwrap();
        assert_eq!(escrow.status, EscrowStatus::Disputed);
    }
    
    resolve_dispute {
        let buyer = create_funded_user::<T>("buyer", 1);
        let seller = create_funded_user::<T>("seller", 2);
        let arbitrator = create_funded_user::<T>("arbitrator", 3);
        
        let amount = T::MinimumEscrowAmount::get() * 10u32.into();
        let deadline = T::Time::now() + T::MinimumDeadline::get() * 2u32.into();
        let description = frame::deps::sp_std::vec![b'x'; 100];
        
        let _ = Pallet::<T>::create_escrow(
            RawOrigin::Signed(buyer.clone()).into(),
            seller.clone(),
            amount,
            description,
            deadline,
            Some(arbitrator.clone()),
        );
        
        let count = EscrowCount::<T>::get();
        let escrow_id = T::Hashing::hash_of(&(&buyer, &seller, count));
        
        let _ = Pallet::<T>::raise_dispute(
            RawOrigin::Signed(buyer.clone()).into(),
            escrow_id,
            frame::deps::sp_std::vec![b'x'; 100],
        );
        
    }: _(RawOrigin::Signed(arbitrator), escrow_id, seller.clone())
    verify {
        let escrow = Escrows::<T>::get(escrow_id).unwrap();
        assert_eq!(escrow.status, EscrowStatus::Resolved);
    }
    
    set_arbitrator_fee {
        let arbitrator: T::AccountId = whitelisted_caller();
        let fee = Percent::from_percent(3);
        
    }: _(RawOrigin::Signed(arbitrator.clone()), fee)
    verify {
        assert_eq!(ArbitratorFees::<T>::get(&arbitrator), fee);
    }
    
    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}