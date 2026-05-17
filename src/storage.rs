use cosmwasm_std::{
    Api, Addr, CanonicalAddr, Empty, Order, OwnedDeps, Querier, RecoverPubkeyError, StdError,
    StdResult, Storage, VerificationError,
};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::collections::{BTreeMap, HashMap};
use std::iter;
use std::marker::PhantomData;
use std::ops::{Bound, RangeBounds};

const CANONICAL_LENGTH: usize = 54;

#[derive(Default)]
/// In-memory [`Storage`] implementation used for tests and host-side execution.
pub struct MemoryStorage {
    data: BTreeMap<Vec<u8>, Vec<u8>>,
}

impl MemoryStorage {
    /// Creates an empty in-memory storage backend.
    pub fn new() -> Self {
        Self::default()
    }
}

impl Storage for MemoryStorage {
    fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        self.data.get(key).cloned()
    }

    fn set(&mut self, key: &[u8], value: &[u8]) {
        if value.is_empty() {
            panic!(
                "TL;DR: Value must not be empty in Storage::set but in most cases you can use Storage::remove instead. Long story: Getting empty values from storage is not well supported at the moment. Some of our internal interfaces cannot differentiate between a non-existent key and an empty value. Right now, you cannot rely on the behaviour of empty values. To protect you from trouble later on, we stop here. Sorry for the inconvenience! We highly welcome you to contribute to CosmWasm, making this more solid one way or the other."
            );
        }

        self.data.insert(key.to_vec(), value.to_vec());
    }

    fn remove(&mut self, key: &[u8]) {
        self.data.remove(key);
    }

    fn range<'a>(
        &'a self,
        start: Option<&[u8]>,
        end: Option<&[u8]>,
        order: Order,
    ) -> Box<dyn Iterator<Item = (Vec<u8>, Vec<u8>)> + 'a> {
        let bounds = range_bounds(start, end);

        match (bounds.start_bound(), bounds.end_bound()) {
            (Bound::Included(start), Bound::Excluded(end)) if start > end => {
                return Box::new(iter::empty());
            }
            _ => {}
        }

        let iter = self.data.range(bounds);
        match order {
            Order::Ascending => Box::new(iter.map(clone_item)),
            Order::Descending => Box::new(iter.rev().map(clone_item)),
        }
    }
}

fn range_bounds(start: Option<&[u8]>, end: Option<&[u8]>) -> impl RangeBounds<Vec<u8>> {
    (
        start.map_or(Bound::Unbounded, |x| Bound::Included(x.to_vec())),
        end.map_or(Bound::Unbounded, |x| Bound::Excluded(x.to_vec())),
    )
}

type BTreeMapRecordRef<'a> = (&'a Vec<u8>, &'a Vec<u8>);

fn clone_item(item_ref: BTreeMapRecordRef) -> (Vec<u8>, Vec<u8>) {
    let (key, value) = item_ref;
    (key.clone(), value.clone())
}

/// Builds in-memory dependencies for contract execution, optionally preloaded from IndexedDB dump data.
pub fn load_owned_deps(
    state_dump: Option<IdbStateDump>,
) -> OwnedDeps<MemoryStorage, EmptyApi, EmptyQuerier, Empty> {
    match state_dump {
        None => OwnedDeps {
            storage: MemoryStorage::default(),
            api: EmptyApi::default(),
            querier: EmptyQuerier::default(),
            custom_query_type: PhantomData,
        },
        Some(dump) => {
            let idb_storage = IdbStorage::load(dump);
            OwnedDeps {
                storage: idb_storage.storage,
                api: EmptyApi::default(),
                querier: EmptyQuerier::default(),
                custom_query_type: PhantomData,
            }
        }
    }
}

/// Wrapper around [`MemoryStorage`] with helpers to load state from browser IndexedDB dumps.
pub struct IdbStorage {
    pub storage: MemoryStorage,
}

impl IdbStorage {
    /// Creates a new [`IdbStorage`] and populates it from a serialized state dump.
    pub fn load(idb: IdbStateDump) -> Self {
        let mut store = IdbStorage {
            storage: MemoryStorage::new(),
        };
        store.load_to_mem_storage(idb);
        store
    }

    /// takes a IdbStateDump and loads the values into MemoryStorage
    pub fn load_to_mem_storage(&mut self, idb_state: IdbStateDump) {
        for (k, v) in idb_state.state_dump {
            self.storage.set(&k, &v);
        }
    }
}

#[serde_as]
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
/// Serialized contract storage dump used to move state between JS and Rust.
pub struct IdbStateDump {
    #[serde_as(as = "Vec<(serde_with::Bytes, serde_with::Bytes)>")]
    pub state_dump: HashMap<Vec<u8>, Vec<u8>>,
}

impl IdbStateDump {
    /// generates a state dump from all key-value pairs in MemoryStorage
    pub fn from(store: MemoryStorage) -> IdbStateDump {
        let mut state: HashMap<Vec<u8>, Vec<u8>> = HashMap::new();

        for (key, value) in store.range(None, None, Order::Ascending) {
            state.insert(key, value);
        }
        IdbStateDump { state_dump: state }
    }
}

// EmptyApi that is meant to conform the traits by the cosmwasm standard contract syntax. The functions of this implementation are not meant to be used or produce any sensible results.
#[derive(Copy, Clone)]
pub struct EmptyApi {
    canonical_length: usize,
}

impl Default for EmptyApi {
    fn default() -> Self {
        EmptyApi {
            canonical_length: CANONICAL_LENGTH,
        }
    }
}

impl Api for EmptyApi {
    fn addr_validate(&self, human: &str) -> StdResult<Addr> {
        self.addr_canonicalize(human).map(|_canonical| ())?;
        Ok(Addr::unchecked(human))
    }

    fn addr_canonicalize(&self, human: &str) -> StdResult<CanonicalAddr> {
        if human.len() < 3 {
            return Err(StdError::msg("Invalid input: human address too short"));
        }
        if human.len() > self.canonical_length {
            return Err(StdError::msg("Invalid input: human address too long"));
        }

        let mut out = Vec::from(human);
        out.resize(self.canonical_length, 0x00);
        Ok(out.into())
    }

    fn addr_humanize(&self, canonical: &CanonicalAddr) -> StdResult<Addr> {
        if canonical.len() != self.canonical_length {
            return Err(StdError::msg(
                "Invalid input: canonical address length not correct",
            ));
        }

        let tmp: Vec<u8> = canonical.clone().into();
        let trimmed = tmp.into_iter().filter(|&x| x != 0x00).collect();
        let human = String::from_utf8(trimmed)?;
        Ok(Addr::unchecked(human))
    }

    fn secp256k1_verify(
        &self,
        _message_hash: &[u8],
        _signature: &[u8],
        _public_key: &[u8],
    ) -> Result<bool, VerificationError> {
        Err(VerificationError::unknown_err(0))
    }

    fn secp256k1_recover_pubkey(
        &self,
        _message_hash: &[u8],
        _signature: &[u8],
        _recovery_param: u8,
    ) -> Result<Vec<u8>, RecoverPubkeyError> {
        Err(RecoverPubkeyError::unknown_err(0))
    }

    fn ed25519_verify(
        &self,
        _message: &[u8],
        _signature: &[u8],
        _public_key: &[u8],
    ) -> Result<bool, VerificationError> {
        Ok(true)
    }

    fn ed25519_batch_verify(
        &self,
        _messages: &[&[u8]],
        _signatures: &[&[u8]],
        _public_keys: &[&[u8]],
    ) -> Result<bool, VerificationError> {
        Ok(true)
    }

    fn debug(&self, message: &str) {
        println!("{message}");
    }
}

/// Empty Querier that is meant to conform the traits expected by the cosmwasm standard contract syntax. It should not be used whatsoever
#[derive(Default)]
pub struct EmptyQuerier {}

impl Querier for EmptyQuerier {
    fn raw_query(&self, _bin_request: &[u8]) -> cosmwasm_std::QuerierResult {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idb_state_dump_round_trips_through_storage() {
        let mut storage = MemoryStorage::new();
        storage.set(b"key1", b"value1");
        storage.set(b"key2", b"value2");

        let dump = IdbStateDump::from(storage);
        assert_eq!(
            dump.state_dump.get(b"key1".as_ref()),
            Some(&b"value1".to_vec())
        );
        assert_eq!(
            dump.state_dump.get(b"key2".as_ref()),
            Some(&b"value2".to_vec())
        );
    }

    #[test]
    fn idb_storage_load_restores_all_keys() {
        let mut storage = MemoryStorage::new();
        storage.set(b"foo", b"bar");
        storage.set(b"baz", b"qux");

        let dump = IdbStateDump::from(storage);
        let loaded = IdbStorage::load(dump);

        assert_eq!(loaded.storage.get(b"foo"), Some(b"bar".to_vec()));
        assert_eq!(loaded.storage.get(b"baz"), Some(b"qux".to_vec()));
    }

    #[test]
    fn idb_state_dump_empty_storage_produces_empty_map() {
        let storage = MemoryStorage::new();
        let dump = IdbStateDump::from(storage);
        assert!(dump.state_dump.is_empty());
    }
}
