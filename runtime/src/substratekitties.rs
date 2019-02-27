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
    <T as system::Trait>::Hash
    {
      Created(AccountId, Hash),
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

            let owned_kitty_count = Self::owned_kitty_count(&sender);
            let new_owned_kitty_count = owned_kitty_count.checked_add(1)
              .ok_or("Over 18 quintillion for the user!!! Too many kiities!")?;

            let all_kitties_count = Self::all_kitties_count();
            let new_all_kitties_count = all_kitties_count.checked_add(1)
              .ok_or("Over 18 quintillion!!! Too many kiities!")?;

            // Nonce and seed a new random hash
            let nonce = <Nonce<T>>::get();
            let random_hash = (<system::Module<T>>::random_seed(), &sender, nonce)
                .using_encoded(<T as system::Trait>::Hashing::hash);

            ensure!(!<Kitties<T>>::exists(random_hash), "The kitty already exists");

            let new_kitty = Kitty {
                id: random_hash,
                dna: random_hash,
                price: <T::Balance as As<u64>>::sa(0),
                gen: 0
            };

            // Update new kitty store
            <Kitties<T>>::insert(random_hash, new_kitty);
            <KittyOwner<T>>::insert(random_hash, &sender);

            // Update global kitties tracking
            <AllKittiesArray<T>>::insert(all_kitties_count, random_hash);
            <AllKittiesIndex<T>>::insert(random_hash, all_kitties_count);
            <AllKittiesCount<T>>::put(new_all_kitties_count);

            <UserKittiesArray<T>>::insert((sender.clone(), owned_kitty_count), random_hash);
            <UserKittiesIndex<T>>::insert(random_hash, owned_kitty_count);
            <UserKittiesCount<T>>::insert(&sender, new_owned_kitty_count);

            // Update nonce
            <Nonce<T>>::mutate(|n| *n += 1);

            Self::deposit_event(RawEvent::Created(sender, random_hash));

            Ok(())
        }
    }
}
