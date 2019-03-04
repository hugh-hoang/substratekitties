use parity_codec::Encode;
use parity_codec_derive::{Decode, Encode};
use runtime_primitives::traits::{As, Hash, Zero};
use support::{
  decl_event, decl_module, decl_storage, dispatch::Result, ensure, StorageMap, StorageValue,
};
use system::ensure_signed;

#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct Kitty<Hash, Balance> {
  id: Hash,
  dna: Hash,
  price: Balance,
  gen: u64,
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
      Transferred(AccountId, AccountId, Hash),
      Bought(AccountId, AccountId, Hash, Balance),
    }
);

decl_storage! {
    trait Store for Module<T: Trait> as Kitty {
        Kitties get(kitty): map T::Hash => Kitty<T::Hash, T::Balance>;
        KittyOwner get(owner_of): map T::Hash => Option<T::AccountId>;

        AllKittiesArray get(all_kitties_by_index): map u64 => T::Hash;
        AllKittiesCount get(all_kitties_count): u64;
        AllKittiesIndex: map T::Hash => u64;

        OwnedKittiesArray get(owned_kitty_by_index): map (T::AccountId, u64) => T::Hash;
        OwnedKittiesCount get(owned_kitty_count): map T::AccountId => u64;
        OwnedKittiesIndex: map T::Hash => u64;

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
            let owner = Self::owner_of(kitty_id).ok_or("No owner for this kitty.")?;
            ensure!(owner == sender, "You do not own this kitty.");

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

        fn transfer(origin, to: T::AccountId, kitty_id: T::Hash) -> Result {
            let sender = ensure_signed(origin)?;

            let owner = Self::owner_of(kitty_id).ok_or("No owner for this kitty.")?;
            ensure!(owner == sender, "You do not own this kitty.");

            Self::_transfer_from(sender, to, kitty_id)?;

            Ok(())
        }

        fn buy_kitty(origin, kitty_id: T::Hash, max_price: T::Balance) -> Result {
            let buyer = ensure_signed(origin)?;

            // ACTION: Check the kitty `exists()`
            ensure!(<Kitties<T>>::exists(kitty_id), "This kitty does not exist.");

            // ACTION: Get the `owner` of the kitty if it exists, otherwise return an `Err()`
            let owner = Self::owner_of(kitty_id).ok_or("No owner for this kitty.")?;
            
            // ACTION: Check that the `sender` is not the `owner`
            ensure!(owner != buyer, "You cannot buy your own kitty.");

            let mut kitty = Self::kitty(kitty_id);

            // ACTION: Get the `kitty_price` and check that it is not zero
            //      HINT:  `runtime_primitives::traits::Zero` allows you to call `kitty_price.is_zero()` which returns a bool
            let kitty_price = kitty.price;
            ensure!(!kitty_price.is_zero(), "The kitty is not for sale.");

            // ACTION: Check `kitty_price` is less than or equal to max_price
            ensure!(kitty_price <= max_price, "The kitty costs more than the price you offer.");

            // ACTION: Use the `Balances` module's `make_transfer()` function to safely transfer funds
            <balances::Module<T>>::make_transfer(&buyer, &owner, kitty_price)?;

            // ACTION: Transfer the kitty
             Self::_transfer_from(owner.clone(), buyer.clone(), kitty_id)?;

            // ACTION: Reset kitty price back to zero, and update the storage
            kitty.price = <T::Balance as As<u64>>::sa(0);
            <Kitties<T>>::insert(kitty_id, kitty);

            // ACTION: Create an event for the cat being bought with relevant details
            //      - new owner
            //      - old owner
            //      - the kitty id
            //      - the price sold for

            Self::deposit_event(RawEvent::Bought(buyer, owner, kitty_id, kitty_price));

            Ok(())
        }

        fn breed_kitty(origin, kitty_id_1: T::Hash, kitty_id_2: T::Hash) -> Result{
            let sender = ensure_signed(origin)?;

            // ACTION: Check both kitty 1 and kitty 2 "exists"
             ensure!(<Kitties<T>>::exists(kitty_id_1), "Kitty 1 does not exist.");
             ensure!(<Kitties<T>>::exists(kitty_id_2), "Kitty 2 does not exist.");

            // ACTION: Generate a `random_hash` using the <Nonce<T>>
            // Nonce and seed a new random hash
            let nonce = <Nonce<T>>::get();
            let random_hash = (<system::Module<T>>::random_seed(), &sender, nonce)
                .using_encoded(<T as system::Trait>::Hashing::hash);

            let kitty_1 = Self::kitty(kitty_id_1);
            let kitty_2 = Self::kitty(kitty_id_2);

            // Our gene splicing algorithm, feel free to make it your own
            let mut final_dna = kitty_1.dna;

            for (i, (dna_2_element, r)) in kitty_2.dna.as_ref().iter().zip(random_hash.as_ref().iter()).enumerate() {
                if r % 2 == 0 {
                    final_dna.as_mut()[i] = *dna_2_element;
                }
            }

            // ACTION: Create a `new_kitty` using: 
            //      - `random_hash` as `id`
            //      - `final_dna` as `dna`
            //      - 0 as `price`
            //      - the max of the parent's `gen` + 1
            //          - Hint: `rstd::cmp::max(1, 5) + 1` is `6`

            // New kitty
            let new_kitty = Kitty {
                id: random_hash,
                dna: final_dna,
                price: <T::Balance as As<u64>>::sa(0),
                gen: rstd::cmp::max(kitty_1.gen, kitty_2.gen) + 1
            };

            // ACTION: `_mint()` your new kitty
            // Do the state stuff
            Self::_mint(sender, random_hash, new_kitty)?;

            // ACTION: Update the <Nonce<T>>
            <Nonce<T>>::mutate(|n| *n += 1);

            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
  fn _mint(to: T::AccountId, kitty_id: T::Hash, new_kitty: Kitty<T::Hash, T::Balance>) -> Result {
    ensure!(!<Kitties<T>>::exists(kitty_id), "The kitty already exists");

    let owned_kitty_count = Self::owned_kitty_count(&to);
    let new_owned_kitty_count = owned_kitty_count
      .checked_add(1)
      .ok_or("Over 18 quintillion for the user!!! Too many kitties!")?;

    let all_kitties_count = Self::all_kitties_count();
    let new_all_kitties_count = all_kitties_count
      .checked_add(1)
      .ok_or("Over 18 quintillion!!! Too many kitties!")?;

    // Update new kitty store
    <Kitties<T>>::insert(kitty_id, new_kitty);
    <KittyOwner<T>>::insert(kitty_id, &to);

    // Update global kitties tracking
    <AllKittiesArray<T>>::insert(all_kitties_count, kitty_id);
    <AllKittiesIndex<T>>::insert(kitty_id, all_kitties_count);
    <AllKittiesCount<T>>::put(new_all_kitties_count);

    <OwnedKittiesArray<T>>::insert((to.clone(), owned_kitty_count), kitty_id);
    <OwnedKittiesIndex<T>>::insert(kitty_id, owned_kitty_count);
    <OwnedKittiesCount<T>>::insert(&to, new_owned_kitty_count);

    Self::deposit_event(RawEvent::Created(to, kitty_id));

    Ok(())
  }

  fn _transfer_from(from: T::AccountId, to: T::AccountId, kitty_id: T::Hash) -> Result {
    // Check if owner exists for `kitty_id`
    //      - If it does, sanity check that `from` is the `owner`
    //      - If it doesn't, return an `Err()` that no `owner` exists
    let owner = Self::owner_of(kitty_id).ok_or("No owner for this kitty.")?;
    ensure!(owner == from, "The 'from' account does not own this kitty.");

    let owned_kitty_count_from = Self::owned_kitty_count(&from);
    let owned_kitty_count_to = Self::owned_kitty_count(&to);

    // Used `checked_add()` to increment the `owned_kitty_count_to` by one into `new_owned_kitty_count_to`
    // Used `checked_sub()` to increment the `owned_kitty_count_from` by one into `new_owned_kitty_count_from`
    //      - Return an `Err()` if overflow or underflow
    let new_owned_kitty_count_to = owned_kitty_count_to
      .checked_add(1)
      .ok_or("Over 18 quintillion!!! Too many kitties!")?;

    let new_owned_kitty_count_from = owned_kitty_count_from
      .checked_sub(1)
      .ok_or("No kitty available to transfer from this account.")?;

    // "Swap and pop"
    // We our convenience storage items to help simplify removing an element from the OwnedKittiesArray
    // We switch the last element of OwnedKittiesArray with the element we want to remove
    let kitty_index = <OwnedKittiesIndex<T>>::get(kitty_id);
    if kitty_index != new_owned_kitty_count_from {
      let last_kitty_id = <OwnedKittiesArray<T>>::get((from.clone(), new_owned_kitty_count_from));
      <OwnedKittiesArray<T>>::insert((from.clone(), kitty_index), last_kitty_id);
      <OwnedKittiesIndex<T>>::insert(last_kitty_id, kitty_index);
    }
    // Now we can remove this item by removing the last element

    // Update KittyOwner for `kitty_id`
    <KittyOwner<T>>::insert(kitty_id, &to);
    // Update OwnedKittiesIndex for `kitty_id`
    <OwnedKittiesIndex<T>>::insert(kitty_id, owned_kitty_count_to);

    // Update OwnedKittiesArray to remove the element from `from`, and add an element to `to`
    //      - HINT: The last element in OwnedKittiesArray(from) is `new_owned_kitty_count_from`
    //              The last element in OwnedKittiesArray(to) is `owned_kitty_count_to`
    <OwnedKittiesArray<T>>::remove((from.clone(), new_owned_kitty_count_from));
    <OwnedKittiesArray<T>>::insert((to.clone(), owned_kitty_count_to), kitty_id);

    // Update the OwnedKittiesCount for `from` and `to`
    <OwnedKittiesCount<T>>::insert(&from, new_owned_kitty_count_from);
    <OwnedKittiesCount<T>>::insert(&to, new_owned_kitty_count_to);

    // Deposit a `Transferred` event with the relevant data:
    //      - from
    //      - to
    //      - kitty_id
    Self::deposit_event(RawEvent::Transferred(from, to, kitty_id));

    Ok(())
  }
}
