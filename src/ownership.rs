use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Core ownable ownership metadata.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct OwnableInfo {
    pub owner: Addr,
    pub issuer: Addr,
    pub ownable_type: Option<String>,
}

/// Response payload for ownable info queries.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InfoResponse {
    pub owner: Addr,
    pub issuer: Addr,
    pub ownable_type: Option<String>,
}

/// Types that can provide an owner address for ownership checks.
pub trait OwnerAddress {
    fn owner_address(&self) -> &Addr;
}

impl OwnerAddress for Addr {
    fn owner_address(&self) -> &Addr {
        self
    }
}

impl OwnerAddress for OwnableInfo {
    fn owner_address(&self) -> &Addr {
        &self.owner
    }
}

/// Verifies that `sender` is the owner and returns a caller-provided unauthorized error otherwise.
pub fn ensure_owner<T, E>(
    owner: &T,
    sender: &Addr,
    unauthorized: impl FnOnce() -> E,
) -> Result<(), E>
where
    T: OwnerAddress + ?Sized,
{
    if sender == owner.owner_address() {
        Ok(())
    } else {
        Err(unauthorized())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    enum TestError {
        Unauthorized(&'static str),
    }

    #[test]
    fn ensure_owner_accepts_owner_addr() {
        let owner = Addr::unchecked("owner");
        let sender = Addr::unchecked("owner");
        let result = ensure_owner(&owner, &sender, || TestError::Unauthorized("forbidden"));
        assert_eq!(result, Ok(()));
    }

    #[test]
    fn ensure_owner_rejects_non_owner_addr() {
        let owner = Addr::unchecked("owner");
        let sender = Addr::unchecked("not-owner");
        let result = ensure_owner(&owner, &sender, || TestError::Unauthorized("forbidden"));
        assert_eq!(result, Err(TestError::Unauthorized("forbidden")));
    }

    #[test]
    fn ensure_owner_accepts_owner_struct() {
        let ownable_info = OwnableInfo {
            owner: Addr::unchecked("owner"),
            issuer: Addr::unchecked("issuer"),
            ownable_type: Some("basic".to_string()),
        };
        let sender = Addr::unchecked("owner");
        let result = ensure_owner(&ownable_info, &sender, || {
            TestError::Unauthorized("forbidden")
        });
        assert_eq!(result, Ok(()));
    }

    #[test]
    fn info_response_serializes_only_supported_fields() {
        let response = InfoResponse {
            owner: Addr::unchecked("owner"),
            issuer: Addr::unchecked("issuer"),
            ownable_type: Some("basic".to_string()),
        };

        let value = serde_json::to_value(response).expect("serialize info response");
        let object = value.as_object().expect("info response object");
        assert_eq!(object.len(), 3);
        assert_eq!(
            object.get("owner").and_then(|value| value.as_str()),
            Some("owner")
        );
        assert_eq!(
            object.get("issuer").and_then(|value| value.as_str()),
            Some("issuer")
        );
        assert_eq!(
            object.get("ownable_type").and_then(|value| value.as_str()),
            Some("basic")
        );
    }
}
