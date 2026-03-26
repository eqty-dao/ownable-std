use cosmwasm_std::{Addr, Api, BlockInfo, CanonicalAddr, ContractInfo, Empty, Env, MemoryStorage, OwnedDeps, Querier, RecoverPubkeyError, StdError, StdResult, Timestamp, VerificationError, Order, Storage, Uint128, Response};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::to_string;
use serde_with::serde_as;
use std::collections::HashMap;
use std::marker::PhantomData;
use wasm_bindgen::{JsValue, JsError};

const CANONICAL_LENGTH: usize = 54;

pub fn create_env() -> Env {
    create_ownable_env(String::new(), None)
}

pub fn create_ownable_env(chain_id: impl Into<String>, time: Option<Timestamp>) -> Env {
    Env {
        block: BlockInfo {
            height: 0,
            time: time.unwrap_or_else(|| Timestamp::from_seconds(0)),
            chain_id: chain_id.into(),
        },
        contract: ContractInfo {
            address: Addr::unchecked(""),
        },
        transaction: None,
    }
}

/// convert an ownable package name into a display title
/// e.g. `ownable-my-first` -> `My First`
pub fn package_title_from_name(name: &str) -> String {
    name
        .trim_start_matches("ownable-")
        .split(['-', '_'])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_ascii_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn load_owned_deps(state_dump: Option<IdbStateDump>) -> OwnedDeps<MemoryStorage, EmptyApi, EmptyQuerier, Empty> {
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

/// returns a hex color in string format from a hash
pub fn get_random_color(hash: String) -> String {
    let (red, green, blue) = derive_rgb_values(hash);
    rgb_hex(red, green, blue)
}


/// takes a hex-encoded hash and derives a seemingly-random rgb tuple
pub fn derive_rgb_values(hash: String) -> (u8, u8, u8) {
    // allow optional 0x and odd length
    let mut s = hash.trim().trim_start_matches("0x").to_string();
    if s.len() % 2 == 1 {
        s.insert(0, '0');
    }

    match hex::decode(&s) {
        Ok(mut bytes) => {
            bytes.reverse();
            let r = *bytes.get(0).unwrap_or(&0);
            let g = *bytes.get(1).unwrap_or(&0);
            let b = *bytes.get(2).unwrap_or(&0);
            (r, g, b)
        }
        Err(_) => (0, 0, 0),
    }
}

/// takes three u8 values representing rgb values (0-255)f
/// and returns a hex string
pub fn rgb_hex(r: u8, g: u8, b: u8) -> String {
    format!("#{:02X}{:02X}{:02X}", r, g, b)
}

/// takes a cw MemoryStorage and Response and returns a JsValue
/// response that contains the memory state dump and response
/// result
pub fn get_json_response(storage: MemoryStorage, response: Response) -> Result<JsValue, JsError> {
    let state_dump= IdbStateDump::from(storage);
    let ownable_state = to_string(&response)?;
    let response_map = js_sys::Map::new();
    response_map.set(
        &JsValue::from_str("mem"),
        &JsValue::from(to_string(&state_dump)?)
    );
    response_map.set(
        &JsValue::from_str("result"),
        &JsValue::from(ownable_state)
    );
    Ok(JsValue::from(response_map))
}

pub struct IdbStorage {
    pub storage: MemoryStorage,
}

impl IdbStorage {
    pub fn load(idb: IdbStateDump) -> Self {
        let mut store = IdbStorage {
            storage: MemoryStorage::new(),
        };
        store.load_to_mem_storage(idb);
        store
    }

    /// takes a IdbStateDump and loads the values into MemoryStorage
    pub fn load_to_mem_storage(&mut self, idb_state: IdbStateDump) {
        for (k, v) in idb_state.state_dump.into_iter() {
            self.storage.set(&k, &v);
        }
    }
}

#[serde_as]
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct IdbStateDump {
    // map of the indexed db key value pairs of the state object store
    #[serde_as(as = "Vec<(_, _)>")]
    pub state_dump: HashMap<Vec<u8>, Vec<u8>>,
}

impl IdbStateDump {
    /// generates a state dump from all key-value pairs in MemoryStorage
    pub fn from(store: MemoryStorage) -> IdbStateDump {
        let mut state: HashMap<Vec<u8>, Vec<u8>> = HashMap::new();

        for (key, value) in store.range(None,None, Order::Ascending) {
            state.insert(key, value);
        }
        IdbStateDump {
            state_dump: state,
        }
    }
}

// EmptyApi that is meant to conform the traits by the cosmwasm standard contract syntax. The functions of this implementation are not meant to be used or produce any sensible results.
#[derive(Copy, Clone)]
pub struct EmptyApi {
    /// Length of canonical addresses created with this API. Contracts should not make any assumtions
    /// what this value is.
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
        // Dummy input validation. This is more sophisticated for formats like bech32, where format and checksum are validated.
        if human.len() < 3 {
            return Err(StdError::generic_err(
                "Invalid input: human address too short",
            ));
        }
        if human.len() > self.canonical_length {
            return Err(StdError::generic_err(
                "Invalid input: human address too long",
            ));
        }

        let mut out = Vec::from(human);

        // pad to canonical length with NULL bytes
        out.resize(self.canonical_length, 0x00);
        // // content-dependent rotate followed by shuffle to destroy
        // // the most obvious structure (https://github.com/CosmWasm/cosmwasm/issues/552)
        // let rotate_by = digit_sum(&out) % self.canonical_length;
        // out.rotate_left(rotate_by);
        // for _ in 0..SHUFFLES_ENCODE {
        //     out = riffle_shuffle(&out);
        // }
        Ok(out.into())
    }

    fn addr_humanize(&self, canonical: &CanonicalAddr) -> StdResult<Addr> {
        if canonical.len() != self.canonical_length {
            return Err(StdError::generic_err(
                "Invalid input: canonical address length not correct",
            ));
        }

        let tmp: Vec<u8> = canonical.clone().into();
        // // Shuffle two more times which restored the original value (24 elements are back to original after 20 rounds)
        // for _ in 0..SHUFFLES_DECODE {
        //     tmp = riffle_shuffle(&tmp);
        // }
        // // Rotate back
        // let rotate_by = digit_sum(&tmp) % self.canonical_length;
        // tmp.rotate_right(rotate_by);
        // Remove NULL bytes (i.e. the padding)
        let trimmed = tmp.into_iter().filter(|&x| x != 0x00).collect();
        // decode UTF-8 bytes into string
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
        println!("{}", message);
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

    // rgb_hex

    #[test]
    fn rgb_hex_formats_correctly() {
        assert_eq!(rgb_hex(0, 0, 0), "#000000");
        assert_eq!(rgb_hex(255, 255, 255), "#FFFFFF");
        assert_eq!(rgb_hex(255, 0, 0), "#FF0000");
        assert_eq!(rgb_hex(0, 128, 255), "#0080FF");
    }

    // derive_rgb_values

    #[test]
    fn derive_rgb_values_reads_last_three_bytes_reversed() {
        // bytes: [0x01, 0x02, 0x03] → reversed → [0x03, 0x02, 0x01] → r=3, g=2, b=1
        assert_eq!(derive_rgb_values("010203".to_string()), (3, 2, 1));
    }

    #[test]
    fn derive_rgb_values_strips_0x_prefix() {
        assert_eq!(
            derive_rgb_values("0x010203".to_string()),
            derive_rgb_values("010203".to_string())
        );
    }

    #[test]
    fn derive_rgb_values_pads_odd_length_input() {
        // "abc" → padded to "0abc" → bytes [0x0a, 0xbc] → reversed [0xbc, 0x0a]
        assert_eq!(derive_rgb_values("abc".to_string()), (0xbc, 0x0a, 0));
    }

    #[test]
    fn derive_rgb_values_returns_zeros_for_invalid_hex() {
        assert_eq!(derive_rgb_values("xyz".to_string()), (0, 0, 0));
    }

    #[test]
    fn derive_rgb_values_returns_zeros_for_empty_input() {
        assert_eq!(derive_rgb_values("".to_string()), (0, 0, 0));
    }

    #[test]
    fn derive_rgb_values_uses_last_three_bytes_of_long_input() {
        // 8 bytes: [0xaa, 0xbb, 0xcc, 0xdd, 0x11, 0x22, 0x33, 0x44]
        // reversed: [0x44, 0x33, 0x22, 0x11, 0xdd, 0xcc, 0xbb, 0xaa]
        // r=0x44, g=0x33, b=0x22
        assert_eq!(
            derive_rgb_values("aabbccdd11223344".to_string()),
            (0x44, 0x33, 0x22)
        );
    }

    // get_random_color

    #[test]
    fn get_random_color_returns_hash_prefixed_hex() {
        let color = get_random_color("010203".to_string());
        assert!(color.starts_with('#'));
        assert_eq!(color.len(), 7);
    }

    #[test]
    fn get_random_color_is_deterministic() {
        let hash = "deadbeef".to_string();
        assert_eq!(get_random_color(hash.clone()), get_random_color(hash));
    }

    // IdbStateDump / IdbStorage round-trip

    #[test]
    fn idb_state_dump_round_trips_through_storage() {
        let mut storage = MemoryStorage::new();
        storage.set(b"key1", b"value1");
        storage.set(b"key2", b"value2");

        let dump = IdbStateDump::from(storage);
        assert_eq!(dump.state_dump.get(b"key1".as_ref()), Some(&b"value1".to_vec()));
        assert_eq!(dump.state_dump.get(b"key2".as_ref()), Some(&b"value2".to_vec()));
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

    // create_ownable_env

    #[test]
    fn create_env_produces_default_env() {
        let env = create_env();
        assert_eq!(env.block.height, 0);
        assert_eq!(env.block.chain_id, "");
    }

    #[test]
    fn create_ownable_env_sets_chain_id() {
        let env = create_ownable_env("my-chain", None);
        assert_eq!(env.block.chain_id, "my-chain");
    }

    #[test]
    fn create_ownable_env_sets_timestamp() {
        use cosmwasm_std::Timestamp;
        let ts = Timestamp::from_seconds(12345);
        let env = create_ownable_env("", Some(ts));
        assert_eq!(env.block.time, ts);
    }
}

// from github.com/CosmWasm/cw-nfts/blob/main/contracts/cw721-metadata-onchain
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
pub struct Metadata {
    pub image: Option<String>,
    pub image_data: Option<String>,
    pub external_url: Option<String>,
    pub description: Option<String>,
    pub name: Option<String>,
    // pub attributes: Option<Vec<Trait>>,
    pub background_color: Option<String>,
    pub animation_url: Option<String>,
    pub youtube_url: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct ExternalEventMsg {
    // CAIP-2 format: <namespace + ":" + reference>
    // e.g. ethereum: eip155:1
    pub network: Option<String>,
    pub event_type: String,
    pub attributes: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct OwnableInfo {
    pub owner: Addr,
    pub issuer: Addr,
    pub ownable_type: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct NFT {
    pub network: String,    // eip155:1
    pub id: Uint128,
    pub address: String, // 0x341...
    pub lock_service: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InfoResponse {
    pub owner: Addr,
    pub issuer: Addr,
    pub nft: Option<NFT>,
    pub ownable_type: Option<String>,
}
