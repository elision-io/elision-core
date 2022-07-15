use scrypto::prelude::*;
use sbor::*;

#[derive(NonFungibleData, TypeId, Encode, Decode, Describe)]
struct OptionTrade {
  option_type: OptionType
}

#[derive(TypeId, Encode, Decode, Describe)]
pub enum OptionType {
  Call,
  Put
}

#[derive(TypeId, Encode, Decode, Describe)]
pub enum OptionState {
  Invalid,
  Active,
  Exercised,
  Expired
}

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
  created: Timestamp,
  hedged: Boolean,
}

#[derive(TypeId, Encode, Decode, Describe)]
struct OptionPosition {
  state: OptionState,
  locked_amount: Decimal,
  hedge_premium: Decimal,
  unhedge_premium: Decimal,
  amount: Decimal,
  created: Timestamp,
  expired: Timestamp,
  strike: Decimal,
}


blueprint! {
  struct OptionsController {
    option_nft_address: ResourceAddress,
    strike: Decimal,
    premium: Decimal,
    origin_epoch: u64,
    expiry_epoch: u64,
    price: Decimal,
    exercised: bool,
    canceled: bool,
    ticker: String,
    options_nft_id: NonFungibleData,
    latest_cost: Decimal
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