#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>

pub use pallet::*;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{pallet_prelude::*};
	use frame_system::pallet_prelude::*;
	use frame_support::{
		traits::{Randomness, Currency, tokens::ExistenceRequirement},
		dispatch::DispatchResult,
		// let failure change be discarded
		transactional
	};
	use frame_support::traits::ReservableCurrency;
	// for serialization
	use codec::{Encode, Decode};
	// for storage hashing
	use sp_io::hashing::blake2_128;
	// for derive macro
	use scale_info::TypeInfo;
	#[cfg(feature = "std")]
	use frame_support::serde::{Deserialize, Serialize};
	

//============================================================

	// Write a Struct to hold Kitty information.
	// define special type from frame
	type AccountOf<T> = <T as frame_system::Config>::AccountId;
	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
	// define kitty struct
	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct Kitty<T: Config>{
		// 16 bytes u8 vector(16 * u8)
		pub dna: [u8; 16],
		pub price: Option<BalanceOf<T>>,
		pub gender: Gender,
		pub owner: AccountOf<T>
	}

	// Implementation to handle Gender type in Kitty struct.
	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
	// Enum declaration for Gender.
	pub enum Gender {
		Male,
		Female
	}
	// define index of kitty, u32
	type KittyIndex = u64;
//============================================================

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

//============================================================

	// Configure the pallet by specifying the parameters and types it depends on.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		// require introduce Randomness for random seed, Specify the type for Randomness we want to specify for runtime.
		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;
		// The Currency handler for the Kitties pallet
		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
		// Add MaxKittyOwned constant
		#[pallet::constant]
		type MaxKittyOwned: Get<u32>;
		#[pallet::constant]
		type ReservationFee: Get<BalanceOf<Self>>;
	}

//============================================================

	// Remaining storage items.
	// define storage: kitty number count
	#[pallet::storage]
	#[pallet::getter(fn kitties_count)]
	pub(super) type KittiesCount<T: Config> = StorageValue<_, u64, ValueQuery>;

	// Map(kitties id -> kitty) by change Blacke2_12Concat to Twox64Concat
	#[pallet::storage]
	#[pallet::getter(fn kitties)]
	pub(super) type Kitties<T: Config> = StorageMap<
		_,
		Twox64Concat,
		KittyIndex,
		Kitty<T>
	>;

	// Map(account id -> [kitties id] with Max bound) by Twox64Concat
	#[pallet::storage]
	#[pallet::getter(fn kitties_owned)]
	pub(super) type KittiesOwned<T: Config> = StorageMap<
		_,
		Twox64Concat,
		T::AccountId,
		BoundedVec<KittyIndex, T::MaxKittyOwned>,
		ValueQuery
	>;

	// Map(kitties id -> account id) by change Blacke2_12Concat to Twox64Concat
	#[pallet::storage]
	#[pallet::getter(fn owner)]
	pub(super) type Owner<T: Config> = StorageMap<
		_,
		Twox64Concat,
		KittyIndex,
		T::AccountId,
		ValueQuery
	>;

//============================================================

	// genesis configuration: set genesis state of storage items
	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub kitties: Vec<(T::AccountId, [u8; 16], Gender)>
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> GenesisConfig<T> {
			GenesisConfig { kitties: vec![] }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			for (acct, dna, gender) in &self.kitties {
				let _ = <Pallet<T>>::mint(acct, Some(dna.clone()), Some(gender.clone()));
			}
		}
	}

//============================================================

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		// A new Kitty was sucessfully created. \[sender, kitty_id\]
		KittyCreated(T::AccountId, KittyIndex, BalanceOf<T>),
		// Kitty price was sucessfully set. \[sender, kitty_id, new_price\]
		KittyPriceSet(T::AccountId, KittyIndex, Option<BalanceOf<T>>),
		// A Kitty was sucessfully transferred. \[from, to, kitty_id\]
		KittyTransferred(T::AccountId, T::AccountId, KittyIndex),
		// A Kitty was sucessfully bought. \[buyer, seller, kitty_id, bid_price\]
		KittyBought(T::AccountId, T::AccountId, KittyIndex, BalanceOf<T>, BalanceOf<T>),
		// a kitty was breed
		KittyBreed(T::AccountId, KittyIndex)
	}

//============================================================

	#[pallet::error]
	pub enum Error<T> {
		// kitty count is over capacity
		KittiesCountOverflow,
		// kitty id is not exist
		InvalidKittyIndex,
		// kitty is out of personal limit
		ExceedMaxKittyOwned,
		// Buyer cannot be the owner.
		BuyerIsKittyOwner,
		// Cannot transfer a kitty to its owner.
		TransferToSelf,
		// Handles checking whether the Kitty exists.
		KittyNotExist,
		// Handles checking that the Kitty is owned by the account transferring, buying or setting a price for it.
		NotKittyOwner,
		// Ensures the Kitty is for sale.
		KittyNotForSale,
		// Ensures that the buying price is greater than the asking price.
		KittyBidPriceTooLow,
		// Ensures that an account has enough funds to purchase a Kitty.
		NotEnoughBalance
	}

//============================================================

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// create_kitty
		#[pallet::weight(100)]
		pub fn create_kitty(origin: OriginFor<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			// reserve amount of free balance when create a kitty
			T::Currency::reserve(&sender, T::ReservationFee::get()).map_err(|_| <Error<T>>::NotEnoughBalance)?;
			let kitty_id = Self::mint(&sender, None, None)?;
			// Logging to the console
			log::info!("A kitty is born with ID: {:?}.", &kitty_id);
			Self::deposit_event(Event::KittyCreated(sender, kitty_id, T::ReservationFee::get()));
			Ok(())
		}
		// set price
		#[pallet::weight(100)]
		pub fn set_price(
			origin: OriginFor<T>,
			kitty_id: KittyIndex,
			new_price: Option<BalanceOf<T>>
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			ensure!(Self::is_kitty_owner(&kitty_id, &sender)?, <Error<T>>::NotKittyOwner);
			let mut kitty = Self::kitties(&kitty_id).ok_or(<Error<T>>::KittyNotExist)?;
			kitty.price = new_price.clone();
			<Kitties<T>>::insert(&kitty_id, kitty);
			Self::deposit_event(Event::KittyPriceSet(sender, kitty_id, new_price));
			Ok(())
		}
		// transfer
		#[pallet::weight(0)]
		pub fn transfer(
			origin: OriginFor<T>, 
			// destination account id
			to: T::AccountId, 
			kitty_id: KittyIndex
		) -> DispatchResult {
			// make sure account id signed
			let from = ensure_signed(origin)?;
			// check if sender is owner of the kitty
			ensure!(Self::is_kitty_owner(&kitty_id, &from)?, <Error<T>>::NotKittyOwner);
			ensure!(from != to, <Error<T>>::TransferToSelf);
			let to_owned = <KittiesOwned<T>>::get(&to);
			ensure!((to_owned.len() as u32) < T::MaxKittyOwned::get(), <Error<T>>::ExceedMaxKittyOwned);
			Self::transfer_kitty_to(&kitty_id, &to)?;
			Self::deposit_event(Event::KittyTransferred(from, to , kitty_id));
			Ok(())
		}
		// buy kitty
		#[transactional]
		#[pallet::weight(100)]
		pub fn buy_kitty(
			origin: OriginFor<T>,
			kitty_id: KittyIndex,
			// highest price that a buyer is willing to pay for a goods
			bid_price: BalanceOf<T>
		) -> DispatchResult {
			let buyer = ensure_signed(origin)?;
			let kitty = Self::kitties(&kitty_id).ok_or(<Error<T>>::KittyNotExist)?;
			ensure!(kitty.owner != buyer, <Error<T>>::BuyerIsKittyOwner);
			if let Some(ask_price) = kitty.price {
				ensure!(ask_price <= bid_price, <Error<T>>::KittyBidPriceTooLow);
			} else {
				Err(<Error<T>>::KittyNotForSale)?;
			}
			ensure!(T::Currency::free_balance(&buyer) >= bid_price, <Error<T>>::NotEnoughBalance);
			let to_owned = <KittiesOwned<T>>::get(&buyer);
			ensure!((to_owned.len() as u32) < T::MaxKittyOwned::get(), <Error<T>>::ExceedMaxKittyOwned);
			let seller = kitty.owner.clone();
			T::Currency::transfer(&buyer, &seller, bid_price, ExistenceRequirement::KeepAlive)?;
			Self::transfer_kitty_to(&kitty_id, &buyer)?;
			let actual_unreserve = T::Currency::unreserve(&buyer, T::ReservationFee::get());
			Self::deposit_event(Event::KittyBought(buyer, seller, kitty_id, bid_price, actual_unreserve));
			Ok(())
		}
		// breed kitty
		#[pallet::weight(100)]
		pub fn breed_kitty(
			origin: OriginFor<T>,
			parent1: KittyIndex,
			parent2: KittyIndex
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			ensure!(Self::is_kitty_owner(&parent1, &sender)?, <Error<T>>::NotKittyOwner);
			ensure!(Self::is_kitty_owner(&parent2, &sender)?, <Error<T>>::NotKittyOwner);
			let new_dna = Self::breed_dna(&parent1, &parent2)?;
			let kitty_id = Self::mint(&sender, Some(new_dna), None)?;
			Self::deposit_event(Event::KittyBreed(sender, kitty_id));
			Ok(())
		}

	}
	// inner function, not called by extrinsic
	impl<T: Config> Pallet<T> {
		// helper function for Kitty struct
		// generate a gender
		fn gen_gender() -> Gender {
			let random = T::Randomness::random(&b"gender"[..]).0;
			match random.as_ref()[0] % 2 {
				0 => Gender::Male,
				_ => Gender::Female
			}
		}
		// funtion to randomly generate DNA
		fn gen_dna() -> [u8; 16] {
			let payload = (
				T::Randomness::random(&b"dna"[..]).0,
				<frame_system::Pallet<T>>::block_number()
			);
			payload.using_encoded(blake2_128)
		}
		// breed dna by two parents
		pub fn breed_dna(
			parent1: &KittyIndex, 
			parent2:&KittyIndex
		) -> Result<[u8; 16], Error<T>> {
			let dna1 = Self::kitties(parent1).ok_or(<Error<T>>::KittyNotExist)?.dna;
			let dna2 = Self::kitties(parent2).ok_or(<Error<T>>::KittyNotExist)?.dna;
			let mut new_dna = Self::gen_dna();
			for i in 0..new_dna.len() {
				//	Bitwise opt
				new_dna[i] = (new_dna[i] & dna1[i]) | (!new_dna[i] & dna2[i]);
			}
			Ok(new_dna)
		}
		// Helper to mint a Kitty(mint token)
		pub fn mint (
			owner: &T::AccountId,
			dna: Option<[u8; 16]>,
			gender: Option<Gender>
		) -> Result<KittyIndex, Error<T>> {
			let kitty = Kitty::<T> {
				dna: dna.unwrap_or_else(Self::gen_dna),
				price: None,
				gender: gender.unwrap_or_else(Self::gen_gender),
				owner: owner.clone()
			};
			
			// get kitty index
			let kitty_id = Self::kitties_count();
			// Performs this operation first as it may fail
			let new_count = &kitty_id.checked_add(1)
				.ok_or(<Error<T>>::KittiesCountOverflow)?;
			// update the kitty's owner vector, Performs this operation first as it may fail
			<KittiesOwned<T>>::try_mutate(&owner, |kitty_vec|{
				kitty_vec.try_push(kitty_id)
			}).map_err(|_| <Error<T>>::ExceedMaxKittyOwned)?;
			// store new kitty with kitty id, StorageMap Api
			<Kitties<T>>::insert(&kitty_id, kitty);
			<Owner<T>>::insert(&kitty_id, owner);
			// store new count, StorageValue Api
			<KittiesCount<T>>::put(new_count);
			Ok(kitty_id)
		}
		// check if kitty_id is in kitties collection
		pub fn is_kitty_owner(
			kitty_id: &KittyIndex, 
			acct: &T::AccountId
		) -> Result<bool, Error<T>> {
			match Self::kitties(kitty_id) {
				Some(kitty) => Ok(kitty.owner == *acct),
				None => Err(<Error<T>>::KittyNotExist)
			}
		}
		// transfer kitty to other
		#[transactional]
		pub fn transfer_kitty_to(
			kitty_id: &KittyIndex,
			to: &T::AccountId
		) -> Result<(), Error<T>> {
			let mut kitty = Self::kitties(&kitty_id).ok_or(<Error<T>>::KittyNotExist)?;
			let prev_owner = kitty.owner.clone();
			<KittiesOwned<T>>::try_mutate(&prev_owner, |owned| {
				if let Some(ind) = owned.iter().position(|&id| id == *kitty_id) {
					owned.swap_remove(ind);
					return Ok(());
				}
				Err(())
			}).map_err(|_| <Error<T>>::KittyNotExist)?;
			kitty.owner = to.clone();
			kitty.price = None;
			<Kitties<T>>::insert(kitty_id, kitty);
			<Owner<T>>::insert(&kitty_id, &to);
			<KittiesOwned<T>>::try_mutate(to, |vec| {
				vec.try_push(*kitty_id)
			}).map_err(|_| <Error<T>>::ExceedMaxKittyOwned)?;
			Ok(())
		}

	}

}
