use scrypto::prelude::*;

blueprint! {
  struct Strategy {
    strategy_fee: Decimal;
    activation_epoch: u64;
    last_known_epoch: u64;
    total_gain: Decimal;
    total_loss: Decimal;
    total_debt: Decimal;
    debt_ration: Decimal;
    min_yield_debt: Decimal;
    max_yield_debt: Decimal;
  }

  impl Strategy {
    pub fn new(
      autopool: Vault,
      stategist: ResourceAddress,
      rewardsAddress: ResourceAddress,
      maintainer: HashMap<ResourceAddress, Vault>
    ) -> ComponentAddress {
      let strategy: ComponentAddress = Self {
        AutoPool: Vault,
        Strategist: ResourceAddress,
        RewardsAddress: ResourceAddress,
        Maintainer: HashMap::new()
      }
      .instatantiate()
      .globalize();
    }

    pub fn transfer(
      &mut self,
      output_address: ResourceAddress,
      amount: Decimal
    ) {
      self.
    }
  }
}