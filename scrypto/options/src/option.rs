enum OptionState {
  Invalid,
  Active,
  Exercised,
  Expired
}

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