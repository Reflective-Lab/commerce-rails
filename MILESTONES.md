# Milestones - Commerce Rails

## M0 - Commercial Authority Home

- [x] Create `~/dev/reflective/commerce-rails/`
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

## M2 - Wolfgang Deployed Integration Driver

Done when Wolfgang is the first application proving Commerce Rails in a deployed Runtime Runway environment.

- [ ] Wolfgang is deployed through Runtime Runway and consumes Runtime Runway-owned user, organization, auth, secrets, telemetry, and runtime configuration.
- [ ] Wolfgang reads subscription, billing, and entitlement state from Commerce Rails instead of app-local Stripe state.
- [ ] Commerce Rails exposes executable command handlers for the partner piggy-back loop with idempotency, replay protection, audit events, Arbiter policy checks, HITL gates, and reconciliation.
- [ ] Stripe integration maps checkout, subscription, invoice, payment, and webhook events into Commerce Rails receipts, subscriptions, entitlements, ledger entries, transfer intents, and payout obligations without making Stripe IDs primary domain IDs.
- [ ] Make.com integration can trigger or observe approved commerce commands through a scoped webhook or API boundary with audit, idempotency, replay protection, and secret handling delegated to Runtime Runway.
- [ ] Wolfgang shows Runtime Runway-backed user and organization context plus Commerce Rails-backed subscription badge and entitlement state in the app shell.
- [ ] End-to-end smoke passes: a user signs in to Wolfgang, starts checkout or subscription, Stripe webhook is received, Commerce Rails grants entitlement, Wolfgang reflects access, and Make.com receives or sends the expected integration event.
- [ ] Documentation names Wolfgang as the first integration driver and keeps Runtime Runway runtime authority separate from Commerce Rails commercial authority.
