# Milestones - Commerce Rails

## M0 - Commercial Authority Home

- [x] Create `~/dev/reflective/movement/commerce-rails/`
      with AGENTS, README, MILESTONES, and KB.
- [x] Define the first Commerce Rails contract surface: partner account,
      customer org, app listing, app installation, subscription, entitlement,
      revenue share, payout obligation, ledger entry, and webhook receipt.
- [x] Decide the first Stripe Connect adapter boundary without making Stripe
      the domain model.
- [x] Adopt movement terminology for the rail control model: mainspring,
      gear train, escapement, balance, caliber, and complication.
- [x] Specify executable command safety: idempotency, webhook verification,
      replay protection, reconciliation, audit events, Arbiter policy checks,
      and HITL gates for high-risk commerce actions.
- [x] Add command/result types for the first partner piggy-back loop.
- [x] Add tests for identifier stability, provider reference mapping, and
      webhook receipt replay keys.

## M1 - Partner Piggy-Back Proof

- [x] Partner app listing can be represented.
- [x] Customer app installation can be represented.
- [x] Subscription can grant an entitlement.
- [x] Revenue-share agreement can produce a payout obligation.
- [x] Stripe Connect adapter can map provider events into Commerce Rails
      receipts without replacing Commerce Rails IDs.
