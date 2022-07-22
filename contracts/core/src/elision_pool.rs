pub struct PriceOracle {}

#[derive(TypeId, Encode, Decode, Describe)]
pub enum TrancheState {
  Invalid,
  Open,
  Closed
}

#[derive(TypeId, Encode, Decode, Describe)]
pub struct Tranche {
  state: TrancheState,
  amount: Decimal,
  share: Decimal,
  creation_epoch: u64,
  hedged: Boolean,
}

blueprint! {
  struct ElisionPool {
    ep_admin_badge: Vault,
    price_oracle: Option<PriceOracle>,
    hedged_balance: Decimal,
    unhedged_balance: Decimal,
    options: HashMap<ResourceAddress, Option>
    tranches: Vec<Tranche>
  }

  impl ElisionPool {}
}