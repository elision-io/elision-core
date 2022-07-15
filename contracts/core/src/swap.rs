use scrypto::prelude::*;
use crate::liquidity_pool::*;
use crate::utils::*;

blueprint! {
  /// Struct used to define the ElisionSwap
  ///
  /// # Contains:
  /// * `liquidity_pools`: Hashmap<(ResourceAddress, ResourceAddress), LiquidityPool>
  ///   - Collection of pools that exist on the Swap
  /// * `address_pair_map`: Hashmap<ResourceAddress,(ResourceAddress, ResourceAddress)>
  ///   - Collection of token pairs and associated provider tokens
  struct ElisionSwap {
    liquidity_pools: HashMap<(ResourceAddress, ResourceAddress), LiquidityPool>,
    address_pair_map: HashMap<ResourceAddress,(ResourceAddress, ResourceAddress)>
  }

  impl ElisionSwap {
    /// Instantiate a new ElisionSwap component
    ///
    /// # Returns:
    /// * `ComponentAddress`: Returns new ElisionSwap component address
    pub fn new() -> ComponentAddress {
      return Self {
        liquidity_pools: HashMap::new(),
        address_pair_map: HashMap::new()
      }
      .instantiate()
      .globalize();
    }

    /// Check to ensure that liquidity pool exists for given token pair
    ///
    /// # Arguments:
    /// * `address`: ResourceAddress - First token address
    /// * `address`: ResourceAddress - Second token address
    ///
    /// # Returns:
    /// * `bool` - True if exists for trading pair, false otherwise
    pub fn pool_exists(
      &self,
      address0: ResourceAddress,
      address1: ResourceAddress
    ) -> bool {
      let addresses: (ResourceAddress, ResourceAddress) = sort_addresses(address0, address1);
      self.liquidity_pools.contains_key(&addresses)
    }

    /// Assert that liquidity pool exists on Swap
    ///
    /// # Arguments:
    /// * `address`: ResourceAddress - First token address
    /// * `address`: ResourceAddress - Second token address
    /// * `label`: String - Label for the assertion output
    pub fn assert_exists(
      &self,
      address0: ResourceAddress,
      address1: ResourceAddress,
      label: String
    ) {
      assert!(
        self.pool_exists(address0, address1),
        "[{}]: Liquidity pool already exists with given address pair.",
        label
      );
    }


    /// Assert that liquidity pool does not exist on Swap
    ///
    /// # Arguments:
    /// * `address`: ResourceAddress - First token address
    /// * `address`: ResourceAddress - Second token address
    /// * `label`: String - Label for the assertion output
    pub fn assert_not_exists(
      &self,
      address0: ResourceAddress,
      address1: ResourceAddress,
      label: String
    ) {
      assert!(
        !self.pool_exists(address0, address1),
        "[{}]: Liquidity pool does not exist with given address pair.",
        label
      );
    }

    /// Create new Liquidity Pool for the Swap
    /// # Arguments:
    /// * `token0`: Bucket - Contains first token to initialize the pool
    /// * `token1`: Bucket - Contains second token to initialize the pool
    ///
    /// # Returns:
    /// * `Bucket` - Contains the provider tokens issued to the liquidity pool creator
    pub fn new_liquidity_pool(
      &mut self,
      token0: Bucket,
      token1: Bucket
    ) -> Bucket {
      // Check if liquidity pool already exists for token pair
      self.assert_not_exists(
        token0.resource_address(),
        token1.resource_address(),
        String::from("New Liquidity Pool")
      );

      // Sort the two buckets based and create liquidity pool from them
      let (bucket0, bucket1): (Bucket, Bucket) = sort_buckets(token0, token1);
      let addresses: (ResourceAddress, ResourceAddress) = (
        bucket0.resource_address(),
        bucket1.resource_address()
      );
      let (liquidity_pool, provider_tokens): (ComponentAddress, Bucket) = LiquidityPool::new(
        bucket0, bucket1, dec!("0.3")
      );

      // Add new liquidity pool to hashmap of all pools
      self.liquidity_pools.insert(addresses, liquidity_pool.into());

      // Add resource address of the provider tokens to the token pairs hashmap
      self.address_pair_map.insert(provider_tokens.resource_address(), addresses);

      return provider_tokens;
    }


    /// Adds liquidity to existing pool or creates new pool if it does not exist
    ///
    /// # Arguments:
    /// * `token0`: Bucket - Contains first token to add to pool
    /// * `token1`: Bucket - Contains second token to add to pool
    ///
    /// # Returns:
    /// * `Bucket` - Remaining tokens from `token0`
    /// * `Bucket` - Remaining tokens from `token1`
    /// * `Bucket` - Tracks tokens issued to the provider
    pub fn add_liquidity(
      &mut self,
      token0: Bucket,
      token1: Bucket
    ) -> (Option<Bucket>, Option<Bucket>, Bucket) {
      let (bucket0, bucket1): (Bucket, Bucket) = sort_buckets(token0, token1);
      let addresses: (ResourceAddress, ResourceAddress) = (bucket0.resource_address(), bucket1.resource_address());

      // Obtain LP component for given address pair
      let optional_lp: Option<&LiquidityPool> = self.liquidity_pools.get(&addresses);
      match optional_lp {
        Some(liquidity_pool) => {
          info!("[Swap Add Liquidity]: Pool for {:?} already exists - Adding liquidity.", addresses);
          let results: (Bucket, Bucket, Bucket) = liquidity_pool.add_liquidity(bucket0, bucket1);
          (Some(results.0), Some(results.1), results.2)
        }

        None => {
          info!("[Swap Add Liquidity]: Pool for {:?} does not exist - Creating new one.", addresses);
          (None, None, self.new_liquidity_pool(bucket0, bucket1))
        }
      }
    }


    /// Removes liquidity from existing pool
    ///
    /// # Arguments:
    /// * `provider_tokens`: Bucket - Contains tokens that provider wants to swap for liquidity
    ///
    /// # Returns:
    /// * `Bucket` - Provider's share of the first token
    /// * `Bucket` - Provider's share of the second token
    pub fn remove_liquidity(
      &mut self,
      provider_tokens: Bucket
    ) -> (Bucket, Bucket) {
      // Ensure that the provider tokens are valid for the Swap
      assert!(
        self.address_pair_map.contains_key(&provider_tokens.resource_address()),
        "[Swap Remove Liquidity]: Incorrect resource address for provider tokens"
      );

      // Obtain address pair of provider tokens and remove them from the pool
      let addresses: (ResourceAddress, ResourceAddress) = self.address_pair_map[&provider_tokens.resource_address()];
      return self.liquidity_pools[&addresses].remove_liquidity(provider_tokens);
    }


    /// Swaps input tokens for desired output tokens
    /// # Arguments:
    /// * `tokens`: Bucket - Bucket containing input tokens for swap
    /// * `output_address`: ResourceAddress - Address of receiving token from swap
    ///
    /// # Returns:
    /// * `Bucket` - Contains the other tokens
    pub fn swap(
      &mut self,
      tokens: Bucket,
      output_address: ResourceAddress
    ) -> Bucket {
      // Check if liquidity pool exists for token pair
      self.assert_exists(tokens.resource_address(), output_address, String::from("Swap"));

      // Sort given addresses, locate liquidity pool, and execute swap
      let addresses: (ResourceAddress, ResourceAddress) = sort_addresses(
        tokens.resource_address(),
        output_address
      );
      return self.liquidity_pools[&addresses].swap(tokens);
    }


    /// Swaps exact amount of input tokens to desired output tokens
    /// # Arguments:
    /// * `tokens`: Bucket - Bucket containing input tokens for swap
    /// * `output_address`: ResourceAddress - Address of receiving token from swap
    ///
    /// # Returns:
    /// * `Bucket` - Contains the other tokens
    pub fn swap_exact_tokens_for_tokens(
      &mut self,
      tokens: Bucket,
      output_address: ResourceAddress,
      min_output_amount: Decimal
    ) -> Bucket {
      // Check if liquidity pool exists for token pair
      self.assert_exists(tokens.resource_address(), output_address, String::from("Swap Exact for Tokens"));

      // Sort given addresses, locate liquidity pool, and execute exact for tokens swap
      let addresses: (ResourceAddress, ResourceAddress) = sort_addresses(
        tokens.resource_address(),
        output_address
      );
      return self.liquidity_pools[&addresses].swap_exact_tokens_for_tokens(tokens, min_output_amount);
    }


    /// Swaps input tokens for exact amount of desired output tokens
    ///
    /// # Arguments:
    /// * `tokens`: Bucket - Bucket containing input tokens for swap
    /// * `output_address`: ResourceAddress - Address of receiving token from swap
    ///
    /// # Returns:
    /// * `Bucket` - Contains the other tokens
    pub fn swap_tokens_for_exact_tokens(
      &mut self,
      tokens: Bucket,
      output_address: ResourceAddress,
      output_amount: Decimal
    ) -> (Bucket, Bucket) {
      // Check if liquidity pool exists for token pair
      self.assert_exists(tokens.resource_address(), output_address, String::from("Swap Tokens for Exact"));

      // Sort given addresses, locate liquidity pool, and execute tokens for exact swap
      let addresses: (ResourceAddress, ResourceAddress) = sort_addresses(
        tokens.resource_address(),
        output_address
      );
      return self.liquidity_pools[&addresses].swap_tokens_for_exact_tokens(tokens, output_amount);
    }
  }
}