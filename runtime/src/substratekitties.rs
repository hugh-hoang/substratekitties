use support::{
    decl_storage,
    decl_module,
    StorageValue,
    StorageMap,
    dispatch::Result,
    ensure
};
use system::ensure_signed;
use runtime_primitives::traits::{As, Hash};
use parity_codec_derive::{Encode, Decode};
use parity_codec::Encode;

pub trait Trait: balances::Trait {}

#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct Kitty<Hash, Balance> {
    id: Hash,
    dna: Hash,
    price: Balance,
    gen: u64
}

decl_storage! {
    trait Store for Module<T: Trait> as KittyStorage {
        Kitties get(kitty): map T::Hash => Kitty<T::Hash, T::Balance>;
        KittyOwner get(owner_of): map T::Hash => Option<T::AccountId>;
        OwnedKitty get(kitty_of_owner): map T::AccountId => T::Hash;
        
        Nonce: u64;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        // Declare public functions here

        fn create_kitty(origin) -> Result {
            let sender = ensure_signed(origin)?;

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

            <Kitties<T>>::insert(random_hash, new_kitty);
            <KittyOwner<T>>::insert(random_hash, &sender);
            <OwnedKitty<T>>::insert(&sender, random_hash);

            // Update nonce
            <Nonce<T>>::mutate(|n| *n += 1);

            Ok(())
        }
    }
}
