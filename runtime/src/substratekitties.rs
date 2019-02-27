use support::{decl_storage, decl_module, StorageMap, dispatch::Result};
use system::ensure_signed;
use runtime_primitives::traits::{As, Hash};
use parity_codec_derive::{Encode, Decode};

pub trait Trait: balances::Trait {}

#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct KittyStruct<Hash, Balance> {
    id: Hash,
    dna: Hash,
    price: Balance,
    gen: u64
}

decl_storage! {
    trait Store for Module<T: Trait> as KittyStorage {
        // Declare storage and getter functions here
        Kitty: map T::AccountId  => KittyStruct<T::Hash, T::Balance>;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        // Declare public functions here

        fn create_kitty(origin) -> Result {
            let sender = ensure_signed(origin)?;

            let new_kitty = KittyStruct {
                id: <T as system::Trait>::Hashing::hash_of(&0),
                dna: <T as system::Trait>::Hashing::hash_of(&0),
                price: <T::Balance as As<u64>>::sa(0),
                gen: 0
            };

            <Kitty<T>>::insert(sender, new_kitty);

            Ok(())
        }
    }
}
