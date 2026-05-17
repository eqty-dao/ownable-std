use cosmwasm_std::{Addr, BlockInfo, ContractInfo, Env, Timestamp};

/// Creates a default [`Env`] for host-side execution.
pub fn create_env() -> Env {
    create_ownable_env(String::new(), None)
}

/// Creates an [`Env`] with a configurable chain id and optional timestamp.
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

#[cfg(test)]
mod tests {
    use super::*;

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
        let ts = Timestamp::from_seconds(12345);
        let env = create_ownable_env("", Some(ts));
        assert_eq!(env.block.time, ts);
    }
}
