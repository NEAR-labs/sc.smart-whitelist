use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, LookupSet};
use near_sdk::{
  env, near_bindgen, AccountId, PanicOnDefault, PublicKey,
};

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct SmartWhitelistContract {
  /// Whitelist administrator public key.
  pub admin_pk: PublicKey,
  /// Service accounts.
  pub service_accounts: LookupSet<AccountId>,
  /// Applicant public keys for whitelisting.
  pub applicants: LookupMap<AccountId, PublicKey>,
  /// Whitelisted account IDs that completed KYC verification.
  pub whitelist: LookupSet<AccountId>,
}

#[near_bindgen]
impl SmartWhitelistContract {
  /// Initializes the contract with the specified public key of the whitelist administrator.
  #[init]
  pub fn new(admin_pk: PublicKey) -> Self {
    Self {
      admin_pk,
      service_accounts: LookupSet::new(b"s".to_vec()),
      applicants: LookupMap::new(b"a"),
      whitelist: LookupSet::new(b"w".to_vec()),
    }
  }

  /**
    Getters
  **/

  /// Returns 'true' if the given identifier is a service identifier.
  pub fn is_service_account_whitelisted(&self, service_account_id: AccountId) -> bool {
    self.service_accounts.contains(&service_account_id)
  }

  /// Returns the public key for the applicant's account
  pub fn get_applicant_pk(&self, applicant_account_id: AccountId) -> Option<PublicKey> {
    self.applicants.get(&applicant_account_id)
  }

  /// Returns 'true' if the given account ID is whitelisted.
  pub fn is_whitelisted(&self, account_id: AccountId) -> bool {
    self.whitelist.contains(&account_id)
  }

  /**
    Administrator
  **/

  /// Adds the given service account ID.
  pub fn add_service_account(&mut self, service_account_id: AccountId) -> bool {
    self.assert_called_by_admin();
    self.service_accounts.insert(&service_account_id)
  }

  /// Removes the given service account ID.
  pub fn remove_service_account(&mut self, service_account_id: AccountId) -> bool {
    self.assert_called_by_admin();
    self.service_accounts.remove(&service_account_id)
  }

  /**
    Applicant
  **/

  /// Storing the public key of the applicant's account ID.
  pub fn register_applicant(&mut self) -> Option<PublicKey> {
    let applicant_account_id = env::signer_account_id();
    if self.applicants.contains_key(&applicant_account_id) {
      env::panic_str("Applicant account ID is already exists");
    }
    if self.is_whitelisted(applicant_account_id.clone()) {
      env::panic_str("Account ID is already whitelisted");
    }
    self.applicants.insert(&applicant_account_id, &env::signer_account_pk())
  }

  /// Removes applicant account ID information.
  pub fn remove_applicant(&mut self) -> Option<PublicKey> {
    self.internal_remove_applicant(env::signer_account_id())
  }

  /**
    Service
  **/

  /// Adds a verified account ID to the whitelist.
  pub fn add_account(&mut self, account_id: AccountId) -> bool {
    self.assert_called_by_service();
    self.internal_remove_applicant(account_id.clone());
    self.whitelist.insert(&account_id)
  }

  /// Removes the given account ID from the whitelist.
  pub fn remove_account(&mut self, account_id: AccountId) -> bool {
    self.assert_called_by_service();
    self.whitelist.remove(&account_id)
  }

  /**
    Internal
  **/

  /// An internal method for deleting the public key of the applicant's account.
  fn internal_remove_applicant(&mut self, applicant_account_id: AccountId) -> Option<PublicKey> {
    if !self.applicants.contains_key(&applicant_account_id) {
      env::panic_str("Unknown applicant");
    }
    self.applicants.remove(&applicant_account_id)
  }

  /// Internal method to verify the signer was the whitelist administrator account ID.
  fn assert_called_by_admin(&self) {
    assert_eq!(
      &env::signer_account_pk(),
      &self.admin_pk,
      "Can only be called by whitelist administrator"
    );
  }

  /// Internal method to verify the predecessor was the service account ID.
  fn assert_called_by_service(&self) {
    if !self.is_service_account_whitelisted(env::predecessor_account_id()) {
      env::panic_str("Can only be called by whitelist service account");
    };
  }
}

#[cfg(test)]
mod tests {
  mod test_utils;

  use super::*;
  use near_sdk::testing_env;
  use test_utils::*;

  #[test]
  fn test_whitelist_flow() {
    let mut context = get_context(
      admin_account().to_string(),
      admin_account().to_string(),
      admin_pk().into_bytes(),
    );
    testing_env!(context.clone());
    let mut contract = SmartWhitelistContract::new(admin_pk());

    testing_env!(context.clone());
    assert!(!contract.is_service_account_whitelisted(service_account()));

    // Adding service account
    testing_env!(context.clone());
    assert!(contract.add_service_account(service_account()));
    testing_env!(context.clone());
    assert!(contract.is_service_account_whitelisted(service_account()));

    testing_env!(context.clone());
    assert!(!contract.is_whitelisted(user_account()));

    context = get_context(
      user_account().to_string(),
      user_account().to_string(),
      user_pk().into_bytes(),
    );
    testing_env!(context.clone());
    contract.register_applicant();
    testing_env!(context.clone());
    let mut result = contract.get_applicant_pk(user_account());
    assert_eq!(result.unwrap(), user_pk());

    context = get_context(
      service_account().to_string(),
      service_account().to_string(),
      vec![0, 1, 2],
    );
    testing_env!(context.clone());
    assert!(contract.add_account(user_account()));

    testing_env!(context.clone());
    result = contract.get_applicant_pk(user_account());
    assert!(result.is_none());

    testing_env!(context.clone());
    assert!(contract.is_whitelisted(user_account()));
  }

  #[test]
  fn test_remove_service_account() {
    let context = get_context(
      admin_account().to_string(),
      admin_account().to_string(),
      admin_pk().into_bytes(),
    );
    testing_env!(context.clone());
    let mut contract = SmartWhitelistContract::new(admin_pk());
    assert!(!contract.is_service_account_whitelisted(service_account()));
    assert!(contract.add_service_account(service_account()));
    assert!(contract.is_service_account_whitelisted(service_account()));

    testing_env!(context.clone());
    assert!(contract.remove_service_account(service_account()));

    testing_env!(context.clone());
    assert!(!contract.is_service_account_whitelisted(service_account()));
  }

  #[test]
  fn test_remove_applicant_manually() {
    let mut context = get_context(
      admin_account().to_string(),
      admin_account().to_string(),
      admin_pk().into_bytes(),
    );
    testing_env!(context.clone());
    let mut contract = SmartWhitelistContract::new(admin_pk());
    assert!(contract.add_service_account(service_account()));

    context = get_context(
      user_account().to_string(),
      user_account().to_string(),
      user_pk().into_bytes(),
    );
    testing_env!(context.clone());
    contract.register_applicant();
    let mut result = contract.get_applicant_pk(user_account());
    assert_eq!(result.unwrap(), user_pk());

    testing_env!(context.clone());
    result = contract.remove_applicant();
    assert_eq!(result.unwrap(), user_pk());

    testing_env!(context.clone());
    result = contract.get_applicant_pk(user_account());
    assert!(result.is_none());
  }

  #[test]
  fn test_remove_account_manually() {
    let mut context = get_context(
      admin_account().to_string(),
      admin_account().to_string(),
      admin_pk().into_bytes(),
    );
    testing_env!(context.clone());
    let mut contract = SmartWhitelistContract::new(admin_pk());
    assert!(contract.add_service_account(service_account()));

    context = get_context(
      user_account().to_string(),
      user_account().to_string(),
      user_pk().into_bytes(),
    );
    testing_env!(context.clone());
    contract.register_applicant();

    context = get_context(
      service_account().to_string(),
      service_account().to_string(),
      vec![0, 1, 2],
    );
    testing_env!(context.clone());
    assert!(contract.add_account(user_account()));
    assert!(contract.is_whitelisted(user_account()));

    testing_env!(context.clone());
    assert!(contract.remove_account(user_account()));

    testing_env!(context.clone());
    assert!(!contract.is_whitelisted(user_account()));
  }

  #[test]
  #[should_panic(expected = "Can only be called by whitelist administrator")]
  fn test_not_enough_admin_permissions() {
    let context = get_context(
      user_account().to_string(),
      user_account().to_string(),
      user_pk().into_bytes(),
    );
    testing_env!(context.clone());
    let mut contract = SmartWhitelistContract::new(admin_pk());
    contract.add_service_account(service_account());
  }

  #[test]
  #[should_panic(expected = "Can only be called by whitelist service account")]
  fn test_not_enough_service_permissions() {
    let mut context = get_context(
      admin_account().to_string(),
      admin_account().to_string(),
      admin_pk().into_bytes(),
    );
    testing_env!(context.clone());
    let mut contract = SmartWhitelistContract::new(admin_pk());
    contract.add_service_account(service_account());

    context = get_context(
      user_account().to_string(),
      user_account().to_string(),
      user_pk().into_bytes(),
    );
    testing_env!(context.clone());
    contract.register_applicant();

    testing_env!(context.clone());
    contract.add_account(user_account());
  }

  #[test]
  #[should_panic(expected = "Applicant account ID is already exists")]
  fn test_retry_registration_incomplete_applicant() {
    let mut context = get_context(
      admin_account().to_string(),
      admin_account().to_string(),
      admin_pk().into_bytes(),
    );
    testing_env!(context.clone());
    let mut contract = SmartWhitelistContract::new(admin_pk());
    contract.add_service_account(service_account());

    context = get_context(
      user_account().to_string(),
      user_account().to_string(),
      user_pk().into_bytes(),
    );
    testing_env!(context.clone());
    contract.register_applicant();

    testing_env!(context.clone());
    contract.register_applicant();
  }

  #[test]
  #[should_panic(expected = "Account ID is already whitelisted")]
  fn test_registration_applicant_is_already_whitelisted() {
    let mut context = get_context(
      admin_account().to_string(),
      admin_account().to_string(),
      admin_pk().into_bytes(),
    );
    testing_env!(context.clone());
    let mut contract = SmartWhitelistContract::new(admin_pk());
    contract.add_service_account(service_account());

    context = get_context(
      user_account().to_string(),
      user_account().to_string(),
      user_pk().into_bytes(),
    );
    testing_env!(context.clone());
    contract.register_applicant();

    context = get_context(
      service_account().to_string(),
      service_account().to_string(),
      vec![0, 1, 2],
    );
    testing_env!(context.clone());
    contract.add_account(user_account());

    context = get_context(
      user_account().to_string(),
      user_account().to_string(),
      user_pk().into_bytes(),
    );
    testing_env!(context.clone());
    contract.register_applicant();
  }

  #[test]
  #[should_panic(expected = "Unknown applicant")]
  fn test_attempt_to_whitelist_unregistered_applicant() {
    let mut context = get_context(
      admin_account().to_string(),
      admin_account().to_string(),
      admin_pk().into_bytes(),
    );
    testing_env!(context.clone());
    let mut contract = SmartWhitelistContract::new(admin_pk());
    contract.add_service_account(service_account());

    context = get_context(
      service_account().to_string(),
      service_account().to_string(),
      vec![0, 1, 2],
    );
    testing_env!(context.clone());
    contract.add_account(user_account());
  }
}
