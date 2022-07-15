use scrypto::prelude::*;
use sbor::*;

#[derive(NonFungibleData)]
pub struct LimitOrder {
  price: Decimal,
  amount: Decimal,
  amount_filled: Decimal,
  is_purchase: bool,
  next_order: NonFungibleData
}

blueprint! {
  enum OptionType { CALL, PUT }
  struct OptionsTrade {
    option_type: OptionType,
    strike: Decimal,
    premium: Decimal,
    expiry_epoch: u64,
    amount: Decimal,
    exercised: bool,
    canceled: bool,
    id: ResourceAddress,
    latest_cost: Decimal,
    writer_badge: Vault,
    buyer_badge: Vault
  }

  impl OptionsTrade {
    pub fn new(
      buyer: Address,
      writer: Address,
      option_type: OptionType,
      stike: Decimal,
      premium: Decimal,
      expiry_epoch: u64,
      amount: Decimal
    ) -> ComponentAddress {
      let option_id =
    }
  }
}