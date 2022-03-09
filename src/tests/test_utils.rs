use near_sdk::{AccountId, PublicKey, VMContext};

pub fn get_context(current_account: String, predecessor_account: String, signer_pk: Vec<u8>) -> VMContext {
  VMContext {
    current_account_id: current_account.parse().unwrap(),
    signer_account_id: predecessor_account.parse().unwrap(),
    signer_account_pk: signer_pk,
    predecessor_account_id: predecessor_account.parse().unwrap(),
    input: vec![],
    block_index: 0,
    block_timestamp: 0,
    account_balance: 10,
    account_locked_balance: 0,
    storage_usage: 10000,
    attached_deposit: 0,
    prepaid_gas: 10u64.pow(18),
    random_seed: vec![0, 1, 2],
    output_data_receivers: vec![],
    epoch_height: 19,
    view_config: None,
  }
}

pub fn admin_pk() -> PublicKey {
  "SaKC6KtLtuvUiSWL5jwurbmKXy1fQ8rgEjy9JisfTvQ".parse().unwrap()
}

pub fn user_pk() -> PublicKey {
  "67eWTJ7sfpz27HdUYwmDGsWyvLSHfteF3sBdcqiMc1Mo".parse().unwrap()
}

pub fn admin_account() -> AccountId {
  AccountId::new_unchecked("admin".to_string())
}

pub fn service_account() -> AccountId {
  AccountId::new_unchecked("service".to_string())
}

pub fn user_account() -> AccountId {
  AccountId::new_unchecked("user".to_string())
}
