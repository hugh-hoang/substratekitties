use support::{
    decl_storage,
    decl_module,
    decl_event,
    StorageValue,
    StorageMap,
    dispatch::Result,
    ensure
};
use system::ensure_signed;
use runtime_primitives::traits::{As, Hash};
use parity_codec_derive::{Encode, Decode};
use parity_codec::Encode;

#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct Kitty<Hash, Balance> {
    id: Hash,
    dna: Hash,
    price: Balance,
    gen: u64
}

pub trait Trait: balances::Trait {
  type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_event!(
  pub enum Event<T>
  where
    <T as system::Trait>::AccountId,
    <T as system::Trait>::Hash,
    <T as balances::Trait>::Balance
    {
      Created(AccountId, Hash),
      PriceSet(AccountId, Hash, Balance),
    }
);

decl_storage! {
    trait Store for Module<T: Trait> as Kitty {
        Kitties get(kitty): map T::Hash => Kitty<T::Hash, T::Balance>;
        KittyOwner get(owner_of): map T::Hash => Option<T::AccountId>;

        AllKittiesArray get(all_kitties_by_index): map u64 => T::Hash;
        AllKittiesCount get(all_kitties_count): u64;
        AllKittiesIndex: map T::Hash => u64;

        UserKittiesArray get(owned_kitty_by_index): map (T::AccountId, u64) => T::Hash;
        UserKittiesCount get(owned_kitty_count): map T::AccountId => u64;
        UserKittiesIndex: map T::Hash => u64;
        
        Nonce: u64;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {

        fn deposit_event<T>() = default;

        fn create_kitty(origin) -> Result {
            let sender = ensure_signed(origin)?;

            // Nonce and seed a new random hash
            let nonce = <Nonce<T>>::get();
            let random_hash = (<system::Module<T>>::random_seed(), &sender, nonce)
                .using_encoded(<T as system::Trait>::Hashing::hash);

            // New kitty
            let new_kitty = Kitty {
                id: random_hash,
                dna: random_hash,
                price: <T::Balance as As<u64>>::sa(0),
                gen: 0
            };

            // Do the state stuff
            Self::_mint(sender, random_hash, new_kitty)?;

            // Update nonce
            <Nonce<T>>::mutate(|n| *n += 1);

            Ok(())
        }

        fn set_price(origin, kitty_id: T::Hash, new_price: T::Balance) -> Result {
            let sender = ensure_signed(origin)?;

            // Check that the kitty with `kitty_id` exists
            ensure!(<Kitties<T>>::exists(kitty_id), "This kitty does not exist.");

            // Check if owner exists for `kitty_id`
            //      - If it does, check that `sender` is the `owner`
            //      - If it doesn't, return an `Err()` that no `owner` exists
            let owner = Self::owner_of(kitty_id).ok_or("This kitty has no owner.")?;
            ensure!(owner == sender, "You are not the owner of this kitty.");

            let mut kitty = Self::kitty(kitty_id);

            // Set the new price for the kitty
            kitty.price = new_price;

            // Update the kitty in storage
            <Kitties<T>>::insert(kitty_id, kitty);

            // Deposit a `PriceSet` event with relevant data
            //      - owner
            //      - kitty id
            //      - the new price
            Self::deposit_event(RawEvent::PriceSet(owner, kitty_id, new_price));

            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
  fn _mint(to: T::AccountId, kitty_id: T::Hash, new_kitty: Kitty<T::Hash, T::Balance>) -> Result {

    ensure!(!<Kitties<T>>::exists(kitty_id), "The kitty already exists");
    
    let owned_kitty_count = Self::owned_kitty_count(&to);
    let new_owned_kitty_count = owned_kitty_count.checked_add(1)
      .ok_or("Over 18 quintillion for the user!!! Too many kiities!")?;

    let all_kitties_count = Self::all_kitties_count();
    let new_all_kitties_count = all_kitties_count.checked_add(1)
      .ok_or("Over 18 quintillion!!! Too many kiities!")?;

    // Update new kitty store
    <Kitties<T>>::insert(kitty_id, new_kitty);
    <KittyOwner<T>>::insert(kitty_id, &to);

    // Update global kitties tracking
    <AllKittiesArray<T>>::insert(all_kitties_count, kitty_id);
    <AllKittiesIndex<T>>::insert(kitty_id, all_kitties_count);
    <AllKittiesCount<T>>::put(new_all_kitties_count);

    <UserKittiesArray<T>>::insert((to.clone(), owned_kitty_count), kitty_id);
    <UserKittiesIndex<T>>::insert(kitty_id, owned_kitty_count);
    <UserKittiesCount<T>>::insert(&to, new_owned_kitty_count);

    Self::deposit_event(RawEvent::Created(to, kitty_id));

    Ok(())
  }
}
