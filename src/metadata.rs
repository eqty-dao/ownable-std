use cosmwasm_std::Uint128;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// convert an ownable package name into a display title
/// e.g. `ownable-my-first` -> `My First`
pub fn package_title_from_name(name: &str) -> String {
    name.trim_start_matches("ownable-")
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

/// returns a hex color in string format from a hash
pub fn get_random_color(hash: String) -> String {
    let (red, green, blue) = derive_rgb_values(hash);
    rgb_hex(red, green, blue)
}

/// takes a hex-encoded hash and derives a seemingly-random rgb tuple
pub fn derive_rgb_values(hash: String) -> (u8, u8, u8) {
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

// from github.com/CosmWasm/cw-nfts/blob/main/contracts/cw721-metadata-onchain
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
/// Standard NFT metadata object.
pub struct Metadata {
    pub image: Option<String>,
    pub image_data: Option<String>,
    pub external_url: Option<String>,
    pub description: Option<String>,
    pub name: Option<String>,
    pub background_color: Option<String>,
    pub animation_url: Option<String>,
    pub youtube_url: Option<String>,
}

/// NFT reference used by ownables.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct NFT {
    pub network: String,
    pub id: Uint128,
    pub address: String,
    pub lock_service: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rgb_hex_formats_correctly() {
        assert_eq!(rgb_hex(0, 0, 0), "#000000");
        assert_eq!(rgb_hex(255, 255, 255), "#FFFFFF");
        assert_eq!(rgb_hex(255, 0, 0), "#FF0000");
        assert_eq!(rgb_hex(0, 128, 255), "#0080FF");
    }

    #[test]
    fn derive_rgb_values_reads_last_three_bytes_reversed() {
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
        assert_eq!(
            derive_rgb_values("aabbccdd11223344".to_string()),
            (0x44, 0x33, 0x22)
        );
    }

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
}
