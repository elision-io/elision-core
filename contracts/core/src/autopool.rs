use scrypto::prelude::*;

/// Elision AutoPools are liquidity pools that auto-compound token rewards back into the pool.
/// Assets are deposited and distributed to different liquidity providers based on the pool Strategy.
///
/// `Strategies` are contracts responsible for providing logic in which the assets are dispersed.
/// There are different various Strategies that can be selected and even customized by the user.
/// This provides the user control on which Strategies they'd like to use. Strategies can be
/// customized and defined by a user. `Strategists` may receive rewards if their strategy is adopted.
///
/// Autopools will have assets stored in an `Unallocated` Vault. This allows funds to be accessed
/// for withdrawal without interfering with the Strategies. If there are no funds available in the
/// `Unallocated` Vault, funds will then be withdrawn from the least impacted Strategy or Strategies.
blueprint! {
  struct AutoPool {

  }
}