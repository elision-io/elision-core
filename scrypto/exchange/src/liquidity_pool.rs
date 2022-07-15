use scrypto::prelude::*;
use crate::utils::*;

blueprint! {
  /// Structure representing a Liquidity Pool for the Elision Exchange
  ///
  /// # Contains:
  /// * `vaults`: HashMap<ResourceAddress, Vault> - Collection of addresses and associated vaults
  /// * `provider_token_address`: ResourceAddress - Token that providers receive for adding liquidity
  /// * `provider_token_admin_badge`: Vault - Badge that gives authority to mint and burn tokens
  /// * `pool_fee`: Decimal - Value between 0 and 100 defining fees paid to liquidity pool
  struct LiquidityPool {
    vaults: HashMap<ResourceAddress, Vault>,
    provider_token_address: ResourceAddress,
    provider_token_admin_badge: Vault,
    pool_fee: Decimal
  }

  impl LiquidityPool {
    /// Create new Liquidity Pool from given tokens passed into the function
    ///
    /// # Arguments:
    /// * `token0`: Bucket - Contains first token to initialize the pool
    /// * `token1`: Bucket - Contains second token to initialize the pool
    /// * `pool_fee`: Decimal - Fee imposed on all swaps from this liquidity pool (0-100).
    ///
    /// # Returns:
    /// * `ComponentAddress` - LiquidityPool component address for initialized pool
    /// * `Bucket` - Bucket containing the issued provider tokens to the liquidity pool creator
    pub fn new(
      token0: Bucket,
      token1: Bucket,
      pool_fee: Decimal
    ) -> (ComponentAddress, Bucket) {
      // Check to see if the liquidity pool has been created or not
      assert_ne!(
        token0.resource_address(), token1.resource_address(),
        "[Pool Creation]: Liquidity Pool must be created using two different tokens."
      );

      assert_ne!(
        borrow_resource_manager!(token0.resource_address()).resource_type(),
        ResourceType::NonFungible,
        "[Pool Creation]: Both assets must be fungible."
      );

      assert_ne!(
        borrow_resource_manager!(token1.resource_address()).resource_type(),
        ResourceType::NonFungible,
        "[Pool Creation]: Both assets must be fungible."
      );

      assert!(
        !token0.is_empty() & !token1.is_empty(),
        "[Pool Creation]: Cannot create a pool from an empty bucket."
      );

      assert!(
        (pool_fee >= Decimal::zero()) & (pool_fee <= dec!("100")),
        "[Pool Creation]: Fee must be between 0 and 100."
      );

      // Sort buckets and create hashmap between vaults and buckets
      let (bucket0, bucket1): (Bucket, Bucket) = sort_buckets(token0, token1);
      let addresses: (ResourceAddress, ResourceAddress) = (bucket0.resource_address(), bucket1.resource_address());
      let pid: String = format!("{}-{}", addresses.0, addresses.1);
      let pair_name: String = pair_symbol(addresses.0, addresses.1);

      info!(
        "[Pool Creation]: Creating new pool from Tokens: {}, Name: {}, Ration: {}:{}",
        pid, pair_name, bucket0.amount(), bucket1.amount()
      );

      let mut vaults: HashMap<ResourceAddress, Vault> = HashMap::new();
      vaults.insert(bucket0.resource_address(), Vault::with_bucket(bucket0));
      vaults.insert(bucket1.resource_address(), Vault::with_bucket(bucket1));

      // Create admin badge for the liquidity pool to give authority to mint and burn
      let provider_token_admin_badge: Bucket = ResourceBuilder::new_fungible()
        .divisibility(DIVISIBILITY_NONE)
        .metadata("name", "Provider Token Admin Badge")
        .metadata("symbol", "PTAB")
        .metadata("description", "Admin Badge with the authority to mint and burn provider tokens")
        .metadata("pid", format!("{}", pid))
        .initial_supply(1);

      // Create provider tokens and mint amount owed to initial liquidity provider
      let provider_tokens: Bucket = ResourceBuilder::new_fungible()
        .divisibility(DIVISIBILITY_MAXIMUM)
        .metadata("name", format!("{} LP Provider Token", pair_name))
        .metadata("symbol", "PT")
        .metadata("description", "Token used to track liquidity provider ownership percentage over liquidity pool.")
        .metadata("pid", format!("{}", pid))
        .mintable(rule!(require(provider_token_admin_badge.resource_address())), LOCKED)
        .burnable(rule!(require(provider_token_admin_badge.resource_address())), LOCKED)
        .initial_supply(100);

      // Create and instantiate liquidity pool component
      let liquidity_pool: ComponentAddress = Self {
        vaults: vaults,
        provider_token_address: provider_tokens.resource_address(),
        provider_token_admin_badge: Vault::with_bucket(provider_token_admin_badge),
        pool_fee: pool_fee
      }
      .instantiate()
      .globalize();

      return (liquidity_pool, provider_tokens);
    }

    /// Verifies that the given address belongs to liquidity pool or not
    ///
    /// # Arguments:
    /// * `address`: ResourceAddress - Address that is going to be verified
    ///
    /// # Returns:
    /// * `bool` - True if address belongs to pool, false otherwise
    pub fn belongs_to_pool(
      &self,
      address: ResourceAddress
    ) -> bool {
      return self.vaults.contains_key(&address);
    }

    /// Asserts that the given address belongs to the pool
    ///
    /// # Arguments:
    /// * `address`: ResourceAddress - Address to verify belongs to the pool
    /// * `label`: String - Label that called the assertion method
    pub fn assert_belongs_to_pool(
      &self,
      address: ResourceAddress,
      label: String
    ) {
      assert!(
        self.belongs_to_pool(address),
        "[{}]: Address does not belong to the pool",
        label
      );
    }

    /// Obtain token addresses in liquidity pool and returns them as a `Vec<ResourceAddress>`
    ///
    /// # Returns:
    /// * `Vec<ResourceAddress>` - Vector of addresses that belong to the pool
    pub fn addresses(&self) -> Vec<ResourceAddress> {
      return self.vaults.keys().cloned().collect::<Vec<ResourceAddress>>();
    }

    /// Obtain name of liquidity pool based on pair symbol.
    ///
    /// # Returns:
    /// * `String` - Pair symbol string
    pub fn name(&self) -> String {
      let addresses: Vec<ResourceAddress> = self.addresses();
      return pair_symbol(addresses[0], addresses[1]);
    }


    /// Retrieve address of other resource if address belongs to the pool
    /// # Arguments
    /// * `resource_address`: ResourceAddress - Address for token from the pool
    ///
    /// # Returns:
    /// * `ResourceAddress` - Address of other token in the pool
    pub fn other_resource_address(
      &self,
      resource_address: ResourceAddress
    ) -> ResourceAddress {
      // Verify address belongs to the pool
      self.assert_belongs_to_pool(resource_address, String::from("Other Resource ResourceAddress"));

      // Check which address was passed in and return the other address
      let addresses: Vec<ResourceAddress> = self.addresses();
      return if addresses[0] == resource_address {addresses[1]} else {addresses[0]};
    }

    /// Calculates Market Maker Equation: `x * y = k`.
    ///
    /// # Returns:
    /// * `Decimal` - Reserve amount of Token0 & Token1 multiplied by each other
    pub fn k(&self) -> Decimal {
      let addresses: Vec<ResourceAddress> = self.addresses();
      return self.vaults[&addresses[0]].amount() * self.vaults[&addresses[1]].amount()
    }

    /// Calculates amount of output that can be given based on the amount of input
    /// # Arguments:
    /// * `input_address`: ResourceAddress - Input token address
    /// * `input_amount`: Decimal - Input amount to calculate output with
    ///
    /// # Returns:
    /// * `Decimal` - Calculated output amount
    ///
    /// # Note:
    /// * `x` - The amount of reserves of token x (the input token)
    /// * `y` - The amount of reserves of token y (the output token)
    /// * `dx` - The amount of input tokens
    /// * `dy` - The amount of output tokens
    /// * `r` - The fee modifier where `r = (100 - fee) / 100`
    pub fn calculate_output_amount(
      &self,
      input_resource_address: ResourceAddress,
      input_amount: Decimal
    ) -> Decimal {
      // Checking if the passed resource address belongs to this pool.
      self.assert_belongs_to_pool(input_resource_address, String::from("Calculate Output"));

      let x: Decimal = self.vaults[&input_resource_address].amount();
      let y: Decimal = self.vaults[&self.other_resource_address(input_resource_address)].amount();
      let dx: Decimal = input_amount;
      let r: Decimal = (dec!("100") - self.pool_fee) / dec!("100");

      let dy: Decimal = (dx * r * y) / ( x + r * dx );
      return dy;
    }

    /// Calculates amount of input that can be given based on the amount of output
    /// # Arguments:
    /// * `output_address`: ResourceAddress - Output token address
    /// * `output_amount`: Decimal - Output amount to calculate input with
    ///
    /// # Returns:
    /// * `Decimal` - Calculated input amount
    ///
    /// # Note:
    /// * `x` - The amount of reserves of token x (the input token)
    /// * `y` - The amount of reserves of token y (the output token)
    /// * `dx` - The amount of input tokens
    /// * `dy` - The amount of output tokens
    /// * `r` - The fee modifier where `r = (100 - fee) / 100`
    pub fn calculate_input_amount(
      &self,
      output_resource_address: ResourceAddress,
      output_amount: Decimal
    ) -> Decimal {
      // Checking if the passed resource address belongs to this pool.
      self.assert_belongs_to_pool(output_resource_address, String::from("Calculate Input"));

      let x: Decimal = self.vaults[&self.other_resource_address(output_resource_address)].amount();
      let y: Decimal = self.vaults[&output_resource_address].amount();
      let dy: Decimal = output_amount;
      let r: Decimal = (dec!("100") - self.pool_fee) / dec!("100");

      let dx: Decimal = (dy * x) / (r * (y - dy));
      return dx;
    }

    /// Deposits a bucket of tokens into this liquidity pool
    ///
    /// # Arguments:
    /// * `bucket`: Bucket - Contains the tokens to deposit into the liquidity pool
    fn deposit(
      &mut self,
      bucket: Bucket
    ) {
      // Verify that the resource belongs to the liquidity pool
      self.assert_belongs_to_pool(bucket.resource_address(), String::from("Deposit"));
      self.vaults.get_mut(&bucket.resource_address()).unwrap().put(bucket);
    }

    /// Withdraws tokens from the liquidity pool
    ///
    /// # Arguments:
    /// * `resource_address`: ResourceAddress - Resource address to withdraw from pool
    /// * `amount`: Decimal - Amount of withdraw from pool
    ///
    /// # Returns:
    /// * `Bucket` - Contains withdrawn tokens
    fn withdraw(
      &mut self,
      resource_address: ResourceAddress,
      amount: Decimal
    ) -> Bucket {
      // Performing the checks to ensure tha the withdraw can actually go through
      self.assert_belongs_to_pool(resource_address, String::from("Withdraw"));

      // Getting the vault of that resource and checking if there is enough liquidity to perform the withdraw.
      let vault: &mut Vault = self.vaults.get_mut(&resource_address).unwrap();
      assert!(
        vault.amount() >= amount,
        "[Withdraw]: Not enough liquidity available for the withdraw."
      );

      return vault.take(amount);
    }

    /// Adds liquidity to the pool in exchange for liquidity provider tokens
    ///
    /// # Arguments:
    /// * `token0`: Bucket - Contains the amount of the first token to add to the pool
    /// * `token1`: Bucket - Contains the amount of the second token to add to the pool
    ///
    /// # Returns:
    /// * `Bucket` - Contains remaining tokens of the `token0`
    /// * `Bucket` - Contains remaining tokens of the `token1`
    /// * `Bucket` - Contains provider tokens issued to the liquidity provider
    pub fn add_liquidity(
      &mut self,
      token0: Bucket,
      token1: Bucket,
    ) -> (Bucket, Bucket, Bucket) {
      // Verify if the tokens belong to this liquidity pool.
      self.assert_belongs_to_pool(token0.resource_address(), String::from("Add Liquidity"));
      self.assert_belongs_to_pool(token1.resource_address(), String::from("Add Liquidity"));

      // Verify that the buckets passed are not empty
      assert!(!token0.is_empty(), "[Add Liquidity]: Cannot add liquidity from an empty bucket");
      assert!(!token1.is_empty(), "[Add Liquidity]: Cannot add liquidity from an empty bucket");
      info!(
        "[Add Liquidity]: Requested adding liquidity of amounts, {}: {}, {}: {}",
        token0.resource_address(), token0.amount(), token1.resource_address(), token1.amount()
      );

      // Sorting out the two buckets passed and getting the values of `dm` and `dn`.
      let (mut bucket0, mut bucket1): (Bucket, Bucket) = sort_buckets(token0, token1);
      let dm: Decimal = bucket0.amount();
      let dn: Decimal = bucket1.amount();

      // Getting the values of m and n from the liquidity pool vaults
      let m: Decimal = self.vaults[&bucket0.resource_address()].amount();
      let n: Decimal = self.vaults[&bucket1.resource_address()].amount();
      info!(
        "[Add Liquidity]: Current reserves: {}: {}, {}: {}",
        bucket0.resource_address(), m, bucket1.resource_address(), n
      );

      // Compute amount to deposit into liquidity pool based on each bucket passed in
      let (amount0, amount1): (Decimal, Decimal) = if (
        (m == Decimal::zero()) | (n == Decimal::zero())) | ((m / n) == (dm / dn)
      ) {
        (dm, dn)
      } else if (m / n) < (dm / dn) {
        (dn * m / n, dn)
      } else {
        (dm, dm * n / m)
      };

      info!(
        "[Add Liquidity]: Liquidity amount to add: {}: {}, {}: {}",
        bucket0.resource_address(), amount0, bucket1.resource_address(), amount1
      );

      // Deposit calculated token amounts into liquidity pool
      self.deposit(bucket0.take(amount0));
      self.deposit(bucket1.take(amount1));

      // Compute and mint the amount of provider tokens that the liquidity provider is owed
      let provider_tokens_manager: &ResourceManager = borrow_resource_manager!(self.provider_token_address);
      let provider_amount: Decimal = if provider_tokens_manager.total_supply() == Decimal::zero() {
        dec!("100.00")
      } else {
        amount0 * provider_tokens_manager.total_supply() / m
      };
      let provider_tokens: Bucket = self.provider_token_admin_badge.authorize(|| {
        provider_tokens_manager.mint(provider_amount)
      });
      info!("[Add Liquidity]: Owed amount of provider tokens: {}", provider_amount);

      // Return remaining provider tokens, token0, and token1
      return (bucket0, bucket1, provider_tokens);
    }

    /// Removes the percentage of the liquidity owed to this liquidity provider
    /// # Arguments:
    /// * `provider_tokens`: Bucket - Contains provider tokens to exchange for share of liquidity
    ///
    /// # Returns:
    /// * `Bucket` - Contains share of liquidity provider of the first token.
    /// * `Bucket` - Contains share of liquidity provider of the second token.
    pub fn remove_liquidity(
      &mut self,
      provider_tokens: Bucket
    ) -> (Bucket, Bucket) {
      // Verify that the provider tokens belong to this exchange
      assert_eq!(
        provider_tokens.resource_address(),
        self.provider_token_address,
        "[Remove Liquidity]: Provider token does not belong to this liquidity pool"
      );

      // Calculating the percentage ownership that provider tokens correspond to
      let provider_tokens_manager: &ResourceManager = borrow_resource_manager!(self.provider_token_address);
      let percentage: Decimal = provider_tokens.amount() / provider_tokens_manager.total_supply();

      // Burning the provider tokens
      self.provider_token_admin_badge.authorize(|| {
        provider_tokens.burn();
      });

      // Withdrawing the amount of tokens owed to this liquidity provider
      let addresses: Vec<ResourceAddress> = self.addresses();
      let bucket0: Bucket = self.withdraw(addresses[0], self.vaults[&addresses[0]].amount() * percentage);
      let bucket1: Bucket = self.withdraw(addresses[1], self.vaults[&addresses[1]].amount() * percentage);

      return (bucket0, bucket1);
    }

    /// Execute token swap and take pool fee
    ///
    /// # Arguments:
    /// * `tokens`: Bucket - Contains the input tokens that will be swapped for other tokens
    ///
    /// # Returns:
    /// * `Bucket` - Contains the other tokens
    pub fn swap(
      &mut self,
      tokens: Bucket
    ) -> Bucket {
      // Verify that tokens belong to this liquidity pool
      self.assert_belongs_to_pool(tokens.resource_address(), String::from("Swap"));
      info!("[Swap]: K before swap: {}", self.k());

      // Calculating the output amount for the given input amount of tokens and withdrawing it from the vault
      let output_amount: Decimal = self.calculate_output_amount(tokens.resource_address(), tokens.amount());
      let output_tokens: Bucket = self.withdraw(
        self.other_resource_address(tokens.resource_address()),
        output_amount
      );

      // Deposit tokens into liquidity pool and return bucket of swapped tokens
      self.deposit(tokens);
      info!("[Swap]: K after swap: {}", self.k());
      return output_tokens;
    }

    /// Swaps exact input tokens for output tokens
    ///
    /// # Arguments:
    /// * `tokens`: Bucket - Contains input tokens that will be swapped
    /// * `min_amount_out`: Decimal - Minimum amount of tokens caller will accept
    ///
    /// # Returns:
    /// * `Bucket` - Contains other tokens
    pub fn swap_exact_tokens_for_tokens(
      &mut self,
      tokens: Bucket,
      min_amount_out: Decimal
    ) -> Bucket {
      // Verify that the bucket passed belongs to liquidity pool
      self.assert_belongs_to_pool(tokens.resource_address(), String::from("Swap Exact"));
      let output_tokens: Bucket = self.swap(tokens);
      assert!(output_tokens.amount() >= min_amount_out, "[Swap Exact]: min_amount_out not satisfied.");

      return output_tokens;
    }

    /// Swaps input tokens for an exact amount of output tokens
    ///
    /// # Arguments:
    /// * `tokens`: Bucket - Contains tokens that the user wishes to swap
    /// * `output_amount`: Decimal - Specific amount of output that the user wishes to receive
    ///
    /// # Returns:
    /// * `Bucket` - Contains output tokens
    /// * `Bucket` - Contains remaining input tokens
    pub fn swap_tokens_for_exact_tokens(
      &mut self,
      mut tokens: Bucket,
      output_amount: Decimal
    ) -> (Bucket, Bucket) {
      // Verify that the bucket passed does belong to this liquidity pool
      self.assert_belongs_to_pool(tokens.resource_address(), String::from("Swap For Exact"));

      // Calculate amount of input tokens required for output token amount
      let input_required: Decimal = self.calculate_input_amount(
        self.other_resource_address(tokens.resource_address()),
        output_amount
      );
      assert!(
        tokens.amount() >= input_required,
        "[Swap For Exact]: Not enough input for the desired amount of output."
      );

      // Depositing the amount of input required into the vaults and taking out the requested amount
      info!("[Swap For Exact]: K before swap: {}", self.k());
      self.deposit(tokens.take(input_required));
      let output_tokens: Bucket = self.withdraw(
        self.other_resource_address(tokens.resource_address()),
        output_amount
      );
      info!("[Swap For Exact]: K after swap: {}", self.k());
      info!("[Swap For Exact]: Amount gievn out: {}", output_tokens.amount());
      return (output_tokens, tokens);
    }
  }
}