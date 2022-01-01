#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*, traits::Randomness, ensure};
	use frame_system::{pallet_prelude::*, ensure_signed, Origin};
	// for serialization
	use codec::{Encode, Decode};
	// for storage hashing
	use sp_io::hashing::blake2_128;

//============================================================

	// Serialization, define kitty struct, 16 bytes u8 vector(16 * u8)
	#[derive(Encode, Decode)]
	pub struct Kitty(pub [u8; 16]);

	// define index of kitty, u32
	type KittyIndex = u32;

//============================================================

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		// require introduce Randomness for random seed
		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;
	}

//============================================================

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

//============================================================

	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		KittyCreate(T::AccountId, KittyIndex),
		KittyTransfer(T::AccountId, T::AccountId, KittyIndex)
	}

//============================================================

	// define storage: kitty number count
	#[pallet::storage]
	#[pallet::getter(fn kitties_count)]
	pub type KittiesCount<T> = StorageValue<_, u32>;

	// Map(index -> kitty) by Blacke2_12Concat
	#[pallet::storage]
	#[pallet::getter(fn kitties)]
	pub type Kitties<T> = StorageMap<
		_,
		Blake2_128Concat,
		KittyIndex,
		Option<Kitty>,
		ValueQuery
	>;

	// Map(index -> owener account id) by Blacke2_12Concat
	#[pallet::storage]
	#[pallet::getter(fn owner)]
	pub type Owner<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		KittyIndex,
		Option<T::AccountId>,
		ValueQuery
	>;

//============================================================

	#[pallet::error]
	pub enum Error<T> {
		KittiesCountOverflow,
		NotOwner,
		SameKittyIndex,
		InvalidKittyIndex
	}

//============================================================

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		pub fn create(origin: OriginFor<T>) -> DispatchResult {
			// make sure account id signed
			let who = ensure_signed(origin)?;
			/* 
				Kitty id start with 0.
				Make sure kitty id do not reach the maximum, 
				so that kitties count is not overflow
			*/ 
			let kitty_id = match Self::kitties_count() {
				Some(kitties_count) => {
					ensure!(kitties_count != KittyIndex::max_value(), Error::<T>::KittiesCountOverflow);
					kitties_count
				},
				None => {0}
			};
			// generate random kitty dna by account id
			let dna = Self::random_value(&who);
			// storage kitties map
			Kitties::<T>::insert(kitty_id, Some(Kitty(dna)));
			// storage owner map
			Owner::<T>::insert(kitty_id, Some(who.clone()));
			// storage kitties count, which is kitty id + 1
			KittiesCount::<T>::put(kitty_id + 1);
			// trigger event
			Self::deposit_event(Event::KittyCreate(who, kitty_id));
	
			Ok(())
		}

		#[pallet::weight(0)]
		pub fn transfer(
			origin: OriginFor<T>, 
			// destination account id
			dest: T::AccountId, 
			kitty_id: KittyIndex
		) -> DispatchResult {
			// make sure account id signed
			let who = ensure_signed(origin)?;
			// check if sender is owner of the kitty
			ensure!(Some(who.clone()) == Owner::<T>::get(kitty_id), Error::<T>::NotOwner);
			// change kitty owner
			Owner::<T>::insert(kitty_id, Some(dest.clone()));
			// trigger event
			Self::deposit_event(Event::KittyTransfer(who, dest, kitty_id));
			
			Ok(())

		}

		#[pallet::weight(0)]
		pub fn breed(
			origin: OriginFor<T>, 
			kitty_id_1: KittyIndex, 
			kitty_id_2: KittyIndex
		) -> DispatchResult {
			// make sure account id signed
			let who = ensure_signed(origin)?;
			ensure!(kitty_id_1 != kitty_id_2, Error::<T>::SameKittyIndex);
			let kitty1 = Self::kitties(kitty_id_1).ok_or(Error::<T>::InvalidKittyIndex)?;
			let kitty2 = Self::kitties(kitty_id_2).ok_or(Error::<T>::InvalidKittyIndex)?;
			let kitty_id = match Self::kitties_count() {
				Some(kitties_count) => {
					ensure!(kitties_count != KittyIndex::max_value(), Error::<T>::KittiesCountOverflow);
					kitties_count
				},
				None => {0}
			};
			let dna_1 = kitty1.0;
			let dna_2 = kitty2.0;
			let selector = Self::random_value(&who);
			let mut new_dna = [0u8; 16];
			for i in 0..dna_1.len() {
				new_dna[i] = (selector[i] & dna_1[i]) | (!selector[i] & dna_2[i]); 
			}

			// storage kitties map
			Kitties::<T>::insert(kitty_id, Some(Kitty(new_dna)));
			// storage owner map
			Owner::<T>::insert(kitty_id, Some(who.clone()));
			// storage kitties count, which is kitty id + 1
			KittiesCount::<T>::put(kitty_id + 1);
			// trigger event
			Self::deposit_event(Event::KittyCreate(who, kitty_id));
			Ok(())
		}

	}

	impl<T: Config> Pallet<T> {
		fn random_value(sender: &T::AccountId) -> [u8; 16] {
			let payload = (
				T::Randomness::random_seed(),
				&sender,
				// transaction index in block(metadata.extrinsic)
				<frame_system::Pallet<T>>::extrinsic_index()
			);
			// hashing -> 128 bit hashcode / 16 -> 8bit(2 hex, 1 byte) -> u8
			payload.using_encoded(blake2_128)
		}
	}

}
