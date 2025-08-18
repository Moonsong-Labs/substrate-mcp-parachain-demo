#![cfg_attr(not(feature = "std"), no_std)]

use frame::{
    deps::{
        frame_support::{
            dispatch::DispatchResult,
            pallet_prelude::*,
            traits::{Currency, ExistenceRequirement, ReservableCurrency, Time},
            PalletId,
        },
        frame_system::pallet_prelude::*,
        sp_runtime::{
            traits::{AccountIdConversion, Saturating, Zero},
            Percent,
        },
    },
    prelude::*,
};

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;
pub use weights::*;

type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
type MomentOf<T> = <<T as Config>::Time as Time>::Moment;

#[frame::pallet]
pub mod pallet {
    use super::*;

    #[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub enum EscrowStatus {
        Active,
        Released,
        Refunded,
        Expired,
        Disputed,
        Resolved,
    }

    #[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct EscrowInfo<T: Config> {
        pub buyer: T::AccountId,
        pub seller: T::AccountId,
        pub arbitrator: Option<T::AccountId>,
        pub amount: BalanceOf<T>,
        pub description: BoundedVec<u8, T::MaxDescriptionLength>,
        pub deadline: MomentOf<T>,
        pub status: EscrowStatus,
        pub platform_fee: BalanceOf<T>,
        pub arbitrator_fee: Option<BalanceOf<T>>,
    }

    #[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct DisputeInfo<T: Config> {
        pub initiator: T::AccountId,
        pub reason: BoundedVec<u8, T::MaxDisputeReasonLength>,
        pub timestamp: MomentOf<T>,
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        
        type Currency: ReservableCurrency<Self::AccountId>;
        
        type Time: Time;
        
        #[pallet::constant]
        type PalletId: Get<PalletId>;
        
        #[pallet::constant]
        type MinimumEscrowAmount: Get<BalanceOf<Self>>;
        
        #[pallet::constant]
        type MaximumEscrowAmount: Get<BalanceOf<Self>>;
        
        #[pallet::constant]
        type MinimumDeadline: Get<MomentOf<Self>>;
        
        #[pallet::constant]
        type MaximumDeadline: Get<MomentOf<Self>>;
        
        #[pallet::constant]
        type MaxDescriptionLength: Get<u32>;
        
        #[pallet::constant]
        type MaxDisputeReasonLength: Get<u32>;
        
        #[pallet::constant]
        type MaxActiveEscrowsPerUser: Get<u32>;
        
        type PlatformFeePercent: Get<Percent>;
        
        type DefaultArbitratorFeePercent: Get<Percent>;
        
        type WeightInfo: WeightInfo;
    }

    #[pallet::storage]
    #[pallet::getter(fn escrows)]
    pub type Escrows<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::Hash,
        EscrowInfo<T>,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn disputes)]
    pub type Disputes<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::Hash,
        DisputeInfo<T>,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn user_escrows)]
    pub type UserEscrows<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Blake2_128Concat,
        T::Hash,
        (),
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn arbitrator_fees)]
    pub type ArbitratorFees<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Percent,
        ValueQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn escrow_count)]
    pub type EscrowCount<T: Config> = StorageValue<_, u64, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn active_escrow_count)]
    pub type ActiveEscrowCount<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        u32,
        ValueQuery,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        EscrowCreated {
            escrow_id: T::Hash,
            buyer: T::AccountId,
            seller: T::AccountId,
            amount: BalanceOf<T>,
            deadline: MomentOf<T>,
        },
        EscrowReleased {
            escrow_id: T::Hash,
            amount: BalanceOf<T>,
            platform_fee: BalanceOf<T>,
        },
        EscrowRefunded {
            escrow_id: T::Hash,
            amount: BalanceOf<T>,
        },
        EscrowExpired {
            escrow_id: T::Hash,
        },
        DisputeRaised {
            escrow_id: T::Hash,
            initiator: T::AccountId,
        },
        DisputeResolved {
            escrow_id: T::Hash,
            winner: T::AccountId,
            amount: BalanceOf<T>,
        },
        ArbitratorFeeSet {
            arbitrator: T::AccountId,
            fee_percent: Percent,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        EscrowNotFound,
        NotAuthorized,
        InvalidAmount,
        InvalidDeadline,
        DescriptionTooLong,
        DisputeReasonTooLong,
        EscrowNotActive,
        NoArbitratorAssigned,
        AlreadyDisputed,
        NotArbitrator,
        CannotArbitrateSelf,
        TooManyActiveEscrows,
        InsufficientBalance,
        DeadlineNotReached,
        InvalidStatus,
        SelfEscrow,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::create_escrow())]
        pub fn create_escrow(
            origin: OriginFor<T>,
            seller: T::AccountId,
            amount: BalanceOf<T>,
            description: Vec<u8>,
            deadline: MomentOf<T>,
            arbitrator: Option<T::AccountId>,
        ) -> DispatchResult {
            let buyer = ensure_signed(origin)?;
            
            ensure!(buyer != seller, Error::<T>::SelfEscrow);
            ensure!(
                amount >= T::MinimumEscrowAmount::get() && amount <= T::MaximumEscrowAmount::get(),
                Error::<T>::InvalidAmount
            );
            
            let bounded_description: BoundedVec<u8, T::MaxDescriptionLength> = description
                .try_into()
                .map_err(|_| Error::<T>::DescriptionTooLong)?;
            
            let now = T::Time::now();
            let min_deadline = now.saturating_add(T::MinimumDeadline::get());
            let max_deadline = now.saturating_add(T::MaximumDeadline::get());
            ensure!(
                deadline >= min_deadline && deadline <= max_deadline,
                Error::<T>::InvalidDeadline
            );
            
            if let Some(ref arb) = arbitrator {
                ensure!(arb != &buyer && arb != &seller, Error::<T>::CannotArbitrateSelf);
            }
            
            let active_count = ActiveEscrowCount::<T>::get(&buyer);
            ensure!(
                active_count < T::MaxActiveEscrowsPerUser::get(),
                Error::<T>::TooManyActiveEscrows
            );
            
            let platform_fee = T::PlatformFeePercent::get() * amount;
            let arbitrator_fee = arbitrator.as_ref().map(|arb| {
                let fee_percent = ArbitratorFees::<T>::get(arb);
                if fee_percent == Percent::zero() {
                    T::DefaultArbitratorFeePercent::get() * amount
                } else {
                    fee_percent * amount
                }
            });
            
            let total_amount = amount
                .saturating_add(platform_fee)
                .saturating_add(arbitrator_fee.unwrap_or_else(Zero::zero));
            
            T::Currency::reserve(&buyer, total_amount)?;
            
            let escrow_count = EscrowCount::<T>::mutate(|count| {
                *count = count.saturating_add(1);
                *count
            });
            
            let escrow_id = T::Hashing::hash_of(&(buyer.clone(), seller.clone(), escrow_count));
            
            let escrow_info = EscrowInfo {
                buyer: buyer.clone(),
                seller: seller.clone(),
                arbitrator: arbitrator.clone(),
                amount,
                description: bounded_description,
                deadline,
                status: EscrowStatus::Active,
                platform_fee,
                arbitrator_fee,
            };
            
            Escrows::<T>::insert(&escrow_id, &escrow_info);
            UserEscrows::<T>::insert(&buyer, &escrow_id, ());
            UserEscrows::<T>::insert(&seller, &escrow_id, ());
            if let Some(ref arb) = arbitrator {
                UserEscrows::<T>::insert(arb, &escrow_id, ());
            }
            
            ActiveEscrowCount::<T>::mutate(&buyer, |count| *count = count.saturating_add(1));
            
            Self::deposit_event(Event::EscrowCreated {
                escrow_id,
                buyer,
                seller,
                amount,
                deadline,
            });
            
            Ok(())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::release_funds())]
        pub fn release_funds(origin: OriginFor<T>, escrow_id: T::Hash) -> DispatchResult {
            let buyer = ensure_signed(origin)?;
            
            let mut escrow = Escrows::<T>::get(&escrow_id).ok_or(Error::<T>::EscrowNotFound)?;
            
            ensure!(escrow.buyer == buyer, Error::<T>::NotAuthorized);
            ensure!(escrow.status == EscrowStatus::Active, Error::<T>::EscrowNotActive);
            
            let total_reserved = escrow.amount
                .saturating_add(escrow.platform_fee)
                .saturating_add(escrow.arbitrator_fee.unwrap_or_else(Zero::zero));
            
            T::Currency::unreserve(&buyer, total_reserved);
            
            T::Currency::transfer(
                &buyer,
                &escrow.seller,
                escrow.amount,
                ExistenceRequirement::AllowDeath,
            )?;
            
            let pallet_account = Self::account_id();
            T::Currency::transfer(
                &buyer,
                &pallet_account,
                escrow.platform_fee,
                ExistenceRequirement::AllowDeath,
            )?;
            
            if let (Some(arbitrator), Some(arbitrator_fee)) = (&escrow.arbitrator, escrow.arbitrator_fee) {
                T::Currency::transfer(
                    &buyer,
                    arbitrator,
                    arbitrator_fee,
                    ExistenceRequirement::AllowDeath,
                )?;
            }
            
            escrow.status = EscrowStatus::Released;
            Escrows::<T>::insert(&escrow_id, &escrow);
            
            ActiveEscrowCount::<T>::mutate(&buyer, |count| *count = count.saturating_sub(1));
            
            Self::deposit_event(Event::EscrowReleased {
                escrow_id,
                amount: escrow.amount,
                platform_fee: escrow.platform_fee,
            });
            
            Ok(())
        }

        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::refund())]
        pub fn refund(origin: OriginFor<T>, escrow_id: T::Hash) -> DispatchResult {
            let seller = ensure_signed(origin)?;
            
            let mut escrow = Escrows::<T>::get(&escrow_id).ok_or(Error::<T>::EscrowNotFound)?;
            
            ensure!(escrow.seller == seller, Error::<T>::NotAuthorized);
            ensure!(escrow.status == EscrowStatus::Active, Error::<T>::EscrowNotActive);
            
            let total_reserved = escrow.amount
                .saturating_add(escrow.platform_fee)
                .saturating_add(escrow.arbitrator_fee.unwrap_or_else(Zero::zero));
            
            T::Currency::unreserve(&escrow.buyer, total_reserved);
            
            escrow.status = EscrowStatus::Refunded;
            Escrows::<T>::insert(&escrow_id, &escrow);
            
            ActiveEscrowCount::<T>::mutate(&escrow.buyer, |count| *count = count.saturating_sub(1));
            
            Self::deposit_event(Event::EscrowRefunded {
                escrow_id,
                amount: escrow.amount,
            });
            
            Ok(())
        }

        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::claim_expired())]
        pub fn claim_expired(origin: OriginFor<T>, escrow_id: T::Hash) -> DispatchResult {
            let buyer = ensure_signed(origin)?;
            
            let mut escrow = Escrows::<T>::get(&escrow_id).ok_or(Error::<T>::EscrowNotFound)?;
            
            ensure!(escrow.buyer == buyer, Error::<T>::NotAuthorized);
            ensure!(escrow.status == EscrowStatus::Active, Error::<T>::EscrowNotActive);
            
            let now = T::Time::now();
            ensure!(now >= escrow.deadline, Error::<T>::DeadlineNotReached);
            
            let total_reserved = escrow.amount
                .saturating_add(escrow.platform_fee)
                .saturating_add(escrow.arbitrator_fee.unwrap_or_else(Zero::zero));
            
            T::Currency::unreserve(&buyer, total_reserved);
            
            escrow.status = EscrowStatus::Expired;
            Escrows::<T>::insert(&escrow_id, &escrow);
            
            ActiveEscrowCount::<T>::mutate(&buyer, |count| *count = count.saturating_sub(1));
            
            Self::deposit_event(Event::EscrowExpired { escrow_id });
            
            Ok(())
        }

        #[pallet::call_index(4)]
        #[pallet::weight(T::WeightInfo::raise_dispute())]
        pub fn raise_dispute(
            origin: OriginFor<T>,
            escrow_id: T::Hash,
            reason: Vec<u8>,
        ) -> DispatchResult {
            let initiator = ensure_signed(origin)?;
            
            let mut escrow = Escrows::<T>::get(&escrow_id).ok_or(Error::<T>::EscrowNotFound)?;
            
            ensure!(
                escrow.buyer == initiator || escrow.seller == initiator,
                Error::<T>::NotAuthorized
            );
            ensure!(escrow.status == EscrowStatus::Active, Error::<T>::EscrowNotActive);
            ensure!(escrow.arbitrator.is_some(), Error::<T>::NoArbitratorAssigned);
            ensure!(!Disputes::<T>::contains_key(&escrow_id), Error::<T>::AlreadyDisputed);
            
            let bounded_reason: BoundedVec<u8, T::MaxDisputeReasonLength> = reason
                .try_into()
                .map_err(|_| Error::<T>::DisputeReasonTooLong)?;
            
            let dispute_info = DisputeInfo {
                initiator: initiator.clone(),
                reason: bounded_reason,
                timestamp: T::Time::now(),
            };
            
            Disputes::<T>::insert(&escrow_id, dispute_info);
            
            escrow.status = EscrowStatus::Disputed;
            Escrows::<T>::insert(&escrow_id, &escrow);
            
            Self::deposit_event(Event::DisputeRaised {
                escrow_id,
                initiator,
            });
            
            Ok(())
        }

        #[pallet::call_index(5)]
        #[pallet::weight(T::WeightInfo::resolve_dispute())]
        pub fn resolve_dispute(
            origin: OriginFor<T>,
            escrow_id: T::Hash,
            winner: T::AccountId,
        ) -> DispatchResult {
            let arbitrator = ensure_signed(origin)?;
            
            let mut escrow = Escrows::<T>::get(&escrow_id).ok_or(Error::<T>::EscrowNotFound)?;
            
            ensure!(
                escrow.arbitrator == Some(arbitrator.clone()),
                Error::<T>::NotArbitrator
            );
            ensure!(escrow.status == EscrowStatus::Disputed, Error::<T>::InvalidStatus);
            ensure!(
                winner == escrow.buyer || winner == escrow.seller,
                Error::<T>::NotAuthorized
            );
            
            let total_reserved = escrow.amount
                .saturating_add(escrow.platform_fee)
                .saturating_add(escrow.arbitrator_fee.unwrap_or_else(Zero::zero));
            
            T::Currency::unreserve(&escrow.buyer, total_reserved);
            
            if winner == escrow.seller {
                T::Currency::transfer(
                    &escrow.buyer,
                    &escrow.seller,
                    escrow.amount,
                    ExistenceRequirement::AllowDeath,
                )?;
                
                let pallet_account = Self::account_id();
                T::Currency::transfer(
                    &escrow.buyer,
                    &pallet_account,
                    escrow.platform_fee,
                    ExistenceRequirement::AllowDeath,
                )?;
            }
            
            if let Some(arbitrator_fee) = escrow.arbitrator_fee {
                T::Currency::transfer(
                    &escrow.buyer,
                    &arbitrator,
                    arbitrator_fee,
                    ExistenceRequirement::AllowDeath,
                )?;
            }
            
            escrow.status = EscrowStatus::Resolved;
            Escrows::<T>::insert(&escrow_id, &escrow);
            Disputes::<T>::remove(&escrow_id);
            
            ActiveEscrowCount::<T>::mutate(&escrow.buyer, |count| *count = count.saturating_sub(1));
            
            Self::deposit_event(Event::DisputeResolved {
                escrow_id,
                winner,
                amount: escrow.amount,
            });
            
            Ok(())
        }

        #[pallet::call_index(6)]
        #[pallet::weight(T::WeightInfo::set_arbitrator_fee())]
        pub fn set_arbitrator_fee(origin: OriginFor<T>, fee_percent: Percent) -> DispatchResult {
            let arbitrator = ensure_signed(origin)?;
            
            ArbitratorFees::<T>::insert(&arbitrator, fee_percent);
            
            Self::deposit_event(Event::ArbitratorFeeSet {
                arbitrator,
                fee_percent,
            });
            
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        pub fn account_id() -> T::AccountId {
            T::PalletId::get().into_account_truncating()
        }
    }
}