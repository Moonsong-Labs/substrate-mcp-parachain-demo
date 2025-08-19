#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    dispatch::DispatchResult,
    pallet_prelude::*,
    traits::{Currency, ExistenceRequirement::AllowDeath, Get, ReservableCurrency, ConstU16},
    PalletId,
};
use frame_system::pallet_prelude::*;
use sp_runtime::traits::{AccountIdConversion, Saturating, Zero};
use sp_std::vec::Vec;

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;
pub use weights::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    pub type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
    pub type EscrowId = u64;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// The currency trait.
        type Currency: ReservableCurrency<Self::AccountId>;

        /// The escrow pallet id, used for deriving its sovereign account ID.
        #[pallet::constant]
        type PalletId: Get<PalletId>;

        /// Minimum amount for an escrow.
        #[pallet::constant]
        type MinEscrow: Get<BalanceOf<Self>>;

        /// Maximum amount for an escrow.
        #[pallet::constant]
        type MaxEscrow: Get<BalanceOf<Self>>;

        /// Maximum number of active escrows per user.
        #[pallet::constant]
        type MaxActiveEscrows: Get<u32>;

        /// Minimum deadline in blocks (approximately 1 hour).
        #[pallet::constant]
        type MinDeadline: Get<BlockNumberFor<Self>>;

        /// Maximum deadline in blocks (approximately 90 days).
        #[pallet::constant]
        type MaxDeadline: Get<BlockNumberFor<Self>>;

        /// Maximum length of escrow description.
        #[pallet::constant]
        type MaxDescriptionLen: Get<u32>;

        /// Origin that can update the platform fee.
        type GovernanceOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        /// Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// The next available escrow ID.
    #[pallet::storage]
    #[pallet::getter(fn next_escrow_id)]
    pub type NextEscrowId<T> = StorageValue<_, EscrowId, ValueQuery>;

    /// Map from escrow ID to escrow details.
    #[pallet::storage]
    #[pallet::getter(fn escrows)]
    pub type Escrows<T: Config> = StorageMap<_, Blake2_128Concat, EscrowId, Escrow<T>>;
    
    /// Active escrows for each user (buyer).
    #[pallet::storage]
    #[pallet::getter(fn active_by_user)]
    pub type ActiveByUser<T: Config> = StorageMap<
        _, 
        Blake2_128Concat, 
        T::AccountId, 
        BoundedVec<EscrowId, T::MaxActiveEscrows>, 
        ValueQuery
    >;
    
    /// Platform fee in basis points (100 = 1%).
    #[pallet::storage]
    #[pallet::getter(fn platform_fee)]
    pub type PlatformFee<T> = StorageValue<_, u16, ValueQuery, ConstU16<50>>; // Default 0.5%

    /// Escrow status enum.
    #[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
    pub enum EscrowStatus { 
        Pending, 
        Released, 
        Refunded, 
        Reclaimed 
    }

    /// Escrow details.
    #[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct Escrow<T: Config> {
        pub id: EscrowId,
        pub buyer: T::AccountId,
        pub seller: T::AccountId,
        pub amount: BalanceOf<T>,
        pub deadline: BlockNumberFor<T>,
        pub description: BoundedVec<u8, T::MaxDescriptionLen>,
        pub status: EscrowStatus,
        pub created_at: BlockNumberFor<T>,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Escrow created. [escrow_id, buyer, seller, amount, deadline]
        EscrowCreated {
            escrow_id: EscrowId,
            buyer: T::AccountId,
            seller: T::AccountId,
            amount: BalanceOf<T>,
            deadline: BlockNumberFor<T>,
        },
        /// Escrow released to seller. [escrow_id, buyer, seller, amount_to_seller, fee]
        EscrowReleased {
            escrow_id: EscrowId,
            buyer: T::AccountId,
            seller: T::AccountId,
            amount: BalanceOf<T>,
            fee: BalanceOf<T>,
        },
        /// Escrow refunded to buyer. [escrow_id, buyer, seller, amount]
        EscrowRefunded {
            escrow_id: EscrowId,
            buyer: T::AccountId,
            seller: T::AccountId,
            amount: BalanceOf<T>,
        },
        /// Escrow reclaimed by buyer after deadline. [escrow_id, buyer, amount]
        EscrowReclaimed {
            escrow_id: EscrowId,
            buyer: T::AccountId,
            amount: BalanceOf<T>,
        },
        /// Platform fee changed. [old_fee, new_fee]
        PlatformFeeChanged {
            old_fee: u16,
            new_fee: u16,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Escrow not found.
        EscrowNotFound,
        /// Not the buyer of this escrow.
        NotBuyer,
        /// Not the seller of this escrow.
        NotSeller,
        /// Not authorized to perform this action.
        NotAuthorized,
        /// Escrow is not in pending status.
        EscrowNotPending,
        /// Deadline has not passed yet.
        DeadlineNotPassed,
        /// Amount is out of allowed bounds.
        AmountOutOfBounds,
        /// Deadline is out of allowed bounds.
        DeadlineOutOfBounds,
        /// Maximum active escrows reached for user.
        MaxActiveEscrowsReached,
        /// Description exceeds maximum length.
        DescriptionTooLong,
        /// Fee is out of allowed bounds (0.5% - 2%).
        FeeOutOfBounds,
        /// Arithmetic overflow.
        ArithmeticOverflow,
        /// Insufficient balance.
        InsufficientBalance,
        /// Cannot escrow to self.
        CannotEscrowToSelf,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Create a new escrow.
        ///
        /// The dispatch origin for this call must be `Signed` by the buyer.
        ///
        /// Parameters:
        /// - `seller`: The account that will receive funds upon release.
        /// - `amount`: The amount to be escrowed.
        /// - `deadline`: The block number after which the buyer can reclaim funds.
        /// - `description`: A description of the escrow (max 500 chars).
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::create_escrow())]
        pub fn create_escrow(
            origin: OriginFor<T>,
            seller: T::AccountId,
            amount: BalanceOf<T>,
            deadline: BlockNumberFor<T>,
            description: Vec<u8>,
        ) -> DispatchResult {
            let buyer = ensure_signed(origin)?;
            
            // Validate inputs
            ensure!(buyer != seller, Error::<T>::CannotEscrowToSelf);
            ensure!(
                amount >= T::MinEscrow::get() && amount <= T::MaxEscrow::get(), 
                Error::<T>::AmountOutOfBounds
            );
            ensure!(
                description.len() <= T::MaxDescriptionLen::get() as usize, 
                Error::<T>::DescriptionTooLong
            );
            
            let now = <frame_system::Pallet<T>>::block_number();
            let min_deadline = now.saturating_add(T::MinDeadline::get());
            let max_deadline = now.saturating_add(T::MaxDeadline::get());
            ensure!(
                deadline >= min_deadline && deadline <= max_deadline, 
                Error::<T>::DeadlineOutOfBounds
            );
            
            // Check active escrows limit
            let mut user_escrows = ActiveByUser::<T>::get(&buyer);
            ensure!(
                user_escrows.len() < T::MaxActiveEscrows::get() as usize, 
                Error::<T>::MaxActiveEscrowsReached
            );
            
            // Create escrow
            let escrow_id = NextEscrowId::<T>::get();
            let desc_bounded: BoundedVec<u8, T::MaxDescriptionLen> = 
                description.try_into().map_err(|_| Error::<T>::DescriptionTooLong)?;
            
            let escrow = Escrow::<T> {
                id: escrow_id,
                buyer: buyer.clone(),
                seller: seller.clone(),
                amount,
                deadline,
                description: desc_bounded,
                status: EscrowStatus::Pending,
                created_at: now,
            };
            
            // Transfer funds to pallet account
            T::Currency::transfer(&buyer, &Self::account_id(), amount, AllowDeath)?;
            
            // Store escrow
            Escrows::<T>::insert(escrow_id, escrow);
            user_escrows.try_push(escrow_id).map_err(|_| Error::<T>::MaxActiveEscrowsReached)?;
            ActiveByUser::<T>::insert(&buyer, user_escrows);
            NextEscrowId::<T>::put(escrow_id.saturating_add(1));
            
            // Emit event
            Self::deposit_event(Event::EscrowCreated {
                escrow_id,
                buyer,
                seller,
                amount,
                deadline,
            });
            
            Ok(())
        }

        /// Release escrowed funds to the seller.
        ///
        /// The dispatch origin for this call must be `Signed` by the buyer.
        ///
        /// Parameters:
        /// - `escrow_id`: The ID of the escrow to release.
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::release_escrow())]
        pub fn release_escrow(origin: OriginFor<T>, escrow_id: EscrowId) -> DispatchResult {
            let who = ensure_signed(origin)?;
            
            let mut escrow = Escrows::<T>::get(escrow_id).ok_or(Error::<T>::EscrowNotFound)?;
            ensure!(escrow.buyer == who, Error::<T>::NotBuyer);
            ensure!(escrow.status == EscrowStatus::Pending, Error::<T>::EscrowNotPending);
            
            // Calculate fee
            let fee_basis_points = PlatformFee::<T>::get();
            let fee = escrow.amount
                .saturating_mul(fee_basis_points.into())
                / 10000u32.into();
            let amount_to_seller = escrow.amount.saturating_sub(fee);
            
            // Transfer to seller
            T::Currency::transfer(&Self::account_id(), &escrow.seller, amount_to_seller, AllowDeath)?;
            
            // Transfer fee to fee account (if non-zero)
            if !fee.is_zero() {
                T::Currency::transfer(&Self::account_id(), &Self::fee_account(), fee, AllowDeath)?;
            }
            
            // Update status
            escrow.status = EscrowStatus::Released;
            Escrows::<T>::insert(escrow_id, &escrow);
            
            // Remove from active list
            Self::remove_from_active_list(&escrow.buyer, escrow_id);
            
            // Emit event
            Self::deposit_event(Event::EscrowReleased {
                escrow_id,
                buyer: escrow.buyer,
                seller: escrow.seller,
                amount: amount_to_seller,
                fee,
            });
            
            Ok(())
        }

        /// Refund escrowed funds to the buyer (initiated by seller).
        ///
        /// The dispatch origin for this call must be `Signed` by the seller.
        ///
        /// Parameters:
        /// - `escrow_id`: The ID of the escrow to refund.
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::refund_escrow())]
        pub fn refund_escrow(origin: OriginFor<T>, escrow_id: EscrowId) -> DispatchResult {
            let who = ensure_signed(origin)?;
            
            let mut escrow = Escrows::<T>::get(escrow_id).ok_or(Error::<T>::EscrowNotFound)?;
            ensure!(escrow.seller == who, Error::<T>::NotSeller);
            ensure!(escrow.status == EscrowStatus::Pending, Error::<T>::EscrowNotPending);
            
            // Full refund, no fee
            T::Currency::transfer(&Self::account_id(), &escrow.buyer, escrow.amount, AllowDeath)?;
            
            // Update status
            escrow.status = EscrowStatus::Refunded;
            Escrows::<T>::insert(escrow_id, &escrow);
            
            // Remove from active list
            Self::remove_from_active_list(&escrow.buyer, escrow_id);
            
            // Emit event
            Self::deposit_event(Event::EscrowRefunded {
                escrow_id,
                buyer: escrow.buyer.clone(),
                seller: escrow.seller,
                amount: escrow.amount,
            });
            
            Ok(())
        }

        /// Reclaim escrowed funds after deadline has passed.
        ///
        /// The dispatch origin for this call must be `Signed` by the buyer.
        ///
        /// Parameters:
        /// - `escrow_id`: The ID of the escrow to reclaim.
        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::reclaim_escrow())]
        pub fn reclaim_escrow(origin: OriginFor<T>, escrow_id: EscrowId) -> DispatchResult {
            let who = ensure_signed(origin)?;
            
            let mut escrow = Escrows::<T>::get(escrow_id).ok_or(Error::<T>::EscrowNotFound)?;
            ensure!(escrow.buyer == who, Error::<T>::NotBuyer);
            ensure!(escrow.status == EscrowStatus::Pending, Error::<T>::EscrowNotPending);
            
            let now = <frame_system::Pallet<T>>::block_number();
            ensure!(now > escrow.deadline, Error::<T>::DeadlineNotPassed);
            
            // Full refund, no fee
            T::Currency::transfer(&Self::account_id(), &escrow.buyer, escrow.amount, AllowDeath)?;
            
            // Update status
            escrow.status = EscrowStatus::Reclaimed;
            Escrows::<T>::insert(escrow_id, &escrow);
            
            // Remove from active list
            Self::remove_from_active_list(&escrow.buyer, escrow_id);
            
            // Emit event
            Self::deposit_event(Event::EscrowReclaimed {
                escrow_id,
                buyer: escrow.buyer.clone(),
                amount: escrow.amount,
            });
            
            Ok(())
        }

        /// Set the platform fee (governance only).
        ///
        /// The dispatch origin for this call must be the configured `GovernanceOrigin`.
        ///
        /// Parameters:
        /// - `new_fee`: The new fee in basis points (50 = 0.5%, 200 = 2%).
        #[pallet::call_index(4)]
        #[pallet::weight(T::WeightInfo::set_fee())]
        pub fn set_fee(origin: OriginFor<T>, new_fee: u16) -> DispatchResult {
            T::GovernanceOrigin::ensure_origin(origin)?;
            
            // Validate fee bounds (0.5% - 2%)
            ensure!(new_fee >= 50 && new_fee <= 200, Error::<T>::FeeOutOfBounds);
            
            let old_fee = PlatformFee::<T>::get();
            PlatformFee::<T>::put(new_fee);
            
            // Emit event
            Self::deposit_event(Event::PlatformFeeChanged {
                old_fee,
                new_fee,
            });
            
            Ok(())
        }
    }

    // Helper functions
    impl<T: Config> Pallet<T> {
        /// Get the pallet's account ID.
        pub fn account_id() -> T::AccountId {
            T::PalletId::get().into_account_truncating()
        }

        /// Get the fee collection account.
        pub fn fee_account() -> T::AccountId {
            // In production, this should be configurable
            Self::account_id()
        }

        /// Remove escrow from user's active list.
        fn remove_from_active_list(who: &T::AccountId, escrow_id: EscrowId) {
            let mut vec = ActiveByUser::<T>::get(who);
            if let Some(pos) = vec.iter().position(|&id| id == escrow_id) {
                vec.swap_remove(pos);
            }
            ActiveByUser::<T>::insert(who, vec);
        }
    }
}