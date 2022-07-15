enum TrancheState {
  Invalid,
  Open,
  Closed
}

struct Tranche {
  state: TrancheState,
  amount: Decimal,
  share: Decimal,
  created: Timestamp,
  hedged: Boolean,
}