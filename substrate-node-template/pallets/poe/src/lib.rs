// compile with std or no_std mode
#![cfg_attr(not(feature = "std"), no_std)]

// A module for proof of existence

//expose module
pub use pallet::*;

// import mock/tests with test config
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

// use pallect macro
#[frame_support::pallet]
// import dependencies
pub mod pallet {
    use frame_support::{
        dispatch::DispatchResultWithPostInfo,
        pallet_prelude::*, traits::Get
    };
    use frame_system::pallet_prelude::*;
    // new add dependency
    use sp_std::vec::Vec;

    // pallet module configure interface
    // inherit from frame_system::Config
    #[pallet::config]
    pub trait Config: frame_system::Config {
        // inherit from frame_system::Config::Event
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        // implement with method get to get constant
        #[pallet::constant]
        type MaxVecLen: Get<u32>;
    }

    // pallet::pallet macro define
    #[pallet::pallet]
    // dependent on storage unit
    #[pallet::generate_store(pub(super) trait Store)]
    // struct include functional module
    pub struct Pallet<T>(_);

    // define storage type
    #[pallet::storage]
    #[pallet::getter(fn proofs)]
    // type = StorageMap
    pub type Proofs<T: Config> = StorageMap<
        _,
        // encryption
        Blake2_128Concat,
        // key(u8 hash)
        Vec<u8>,
        // value(tuple both come from system)
        (T::AccountId, T::BlockNumber)
    >;

    // define event enum type
    #[pallet::event]
	/* out of version
    // convert AccountId to "AccountId" for front-end
    // #[pallet::metadata(T::AccountId = "AccountId")]
	*/
    // event activate method deposit_event
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        ClaimCreated(T::AccountId, Vec<u8>),
        ClaimRevoked(T::AccountId, Vec<u8>),
        ClaimTransaction(T::AccountId, Vec<u8>, T::AccountId)
    }

    // define error enum type
    #[pallet::error]
    pub enum Error<T> {
        ProofAlreadyExist,
        ClaimNotExist,
        NotClaimOwner,
        NotDestination,
        ClaimSizeOutOfBound
    }

    // functions active in specific period
    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    // define callable functions
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        // weight macro. 
        // in real, weight need to be tested for a reasonable value
        #[pallet::weight(0)]
        // create deposit
        pub fn create_claim(
            // who send the transaction
            origin: OriginFor<T>, 
            // hash
            claim: Vec<u8>
            // result include weight
        ) -> DispatchResultWithPostInfo {
            ensure!((claim.len() as u32) <= T::MaxVecLen::get(), Error::<T>::ClaimSizeOutOfBound);
            // calibrate sender
            let sender = ensure_signed(origin)?;
            // check if claim exist
            ensure!(!Proofs::<T>::contains_key(&claim), Error::<T>::ProofAlreadyExist);
            // store data as StorageMap
            Proofs::<T>::insert(
                &claim, 
                (sender.clone(), frame_system::Pallet::<T>::block_number())
            );
            // event activate
            Self::deposit_event(Event::ClaimCreated(sender, claim));
            // result
            Ok(().into())
        }

        #[pallet::weight(0)]
        // create revocation
        pub fn revoke_claim(
            origin: OriginFor<T>,
            claim: Vec<u8>
        ) -> DispatchResultWithPostInfo {
            // check sender valid
            let sender = ensure_signed(origin)?;
            // check claim exist
            let (owner, _) = Proofs::<T>::get(&claim).ok_or(Error::<T>::ClaimNotExist)?;
            // check if sender own the claim
            ensure!(owner == sender, Error::<T>::NotClaimOwner);
            // remove the claim
            Proofs::<T>::remove(&claim);
            // event active
            Self::deposit_event(Event::ClaimRevoked(sender, claim));
            // result
            Ok(().into())
        }

        #[pallet::weight(0)]
        // create transaction
        pub fn transfer_claim(
            origin: OriginFor<T>,
            claim: Vec<u8>,
            dest: T::AccountId
        ) -> DispatchResultWithPostInfo {
            // check sender valid
            let sender = ensure_signed(origin)?;
            // check claim exist
            let (owner, _) = Proofs::<T>::get(&claim).ok_or(Error::<T>::ClaimNotExist)?;
            // check if sender own the claim
            ensure!(owner == sender, Error::<T>::NotClaimOwner);
            // check if sender is same to destination
            ensure!(dest != sender, Error::<T>::NotDestination);
            /* Redundant logic, insert contain remove logic
            //flipping two steps below causes claim-lose for some reason
            // remove the claim 
            // Proofs::<T>::remove(&claim);
             */
            // store data as StorageMap
            Proofs::<T>::insert(
                &claim, 
                (dest.clone(), frame_system::Pallet::<T>::block_number())
            );
            // event active
            Self::deposit_event(Event::ClaimTransaction(sender, claim, dest));
            // result
            Ok(().into())
        }
    }
}
