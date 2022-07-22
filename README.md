# Elision
Elision Protocol smart contracts containing core features.

*__Elision Derivatives Protocol__* (EDP) is a derivatives trading platform that enables the decentralized exchange of various financial derivative smart contracts.
The contracts aggregate and secure liquidity, distribute rewards for liquidity providers, and implements the `peer-to-pool` model. 

*__Elision Leveraged Token Protocol__* (ELTP) provides a decentralized solution for creating and exchanging leveraged assets with the use of no-loss lending pools.
These no-loss lending pools provide users with the ability to earn interest in an easy, secure, and transparent way.

The smart contracts are written in Scrypto and are running on the Radix Network.


## Derivative Calculations
### Binomial Option Pricing Model (BOPM)
Binomial Option Pricing Model is a pricing model that is utilized in the calculation of American options.
This model takes an iterative approach with the result of moving up or moving down a [binomial tree](https://www.investopedia.com/terms/b/binomial_tree.asp).

Assumptions
1. At every point in time, the price can go to only two possible ways - up or down
2. The underlying asset pays no dividends
3. The interest rate (discount factor) is a constant throughout the period
4. Investors are risk-neutral, indifferent to risk; 
5. The risk-free rate remains constant.

```
Understanding the Model
- If we set the current (spot) price of an option as S, then the price can either go up to S+ or down to S-.
    u = s+ / s; 
    d = s- / s;
    
Call Option Contracts 
- Call options entitle the holder to purchase the underlying asset at exercise price (Px)
- A call option is "in-the-money" when the spot price is above the exercise price (S > Px)
- When upward price movement occurs, the payoff of the call option is max value between zero and
  the spot price, multiplied by the up factor and reduced with the exercise price.
    C+ = max(O,uS - Px);
    C- = max(O,dS - Px);
    
Put Option Contracts
- A put option entitles the holder to sell at the exercise price Px.
- When the price goes down or up, we calculate a put option like below
    P+ = max(Px - uS,O)
    P- = max(Px - dS,O)
```

### Collateralization Ratio (CR)
This is the ratio of the value of the collateral to the value of the asset being collaterlized.
The Minimum Collateralization Ratio (MCR) is the minimum required value by the Collateralization Ratio.
```
CR = C * C^rc / A * A^rc

Where:

C    = Units of collateral assets
A    = Units of underlying assets
C^rc = Price of a single collateral asset unit in fiat currency (USD)
A^rc = Price of a single underlying asset unit in fiat currency (USD)

```

