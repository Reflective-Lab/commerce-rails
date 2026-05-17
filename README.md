# Commerce Rails

Reflective Commerce Rails is the Reflective-owned business layer for commercial
authority.

It exists so builders and SMBs can launch on the Reflective stack without first
building billing, entitlement, partner marketplace, revenue-share, payout,
refund, dispute, and reconciliation infrastructure.

This repository owns Reflective Labs commercial contracts. It consumes Bedrock
and Mosaic, but it does not live inside them.

## What It Owns

- Customer and partner commercial accounts.
- App listings and app installations.
- Plans, prices, subscriptions, invoices, and charges.
- Entitlement grants across Reflective-hosted apps.
- Revenue-share agreements, transfer intents, and payout obligations.
- Refund decisions, disputes, webhook receipts, and commercial audit.
- Provider reconciliation, starting with Stripe Connect.

## What It Does Not Own

- Canonical user identity, authentication, sessions, invites, roles, or org
  membership.
- Canonical organization tenancy.
- DevOps substrate: deployments, environments, runtime config, telemetry, or
  secret storage.
- Generic provider capabilities, storage, search, fetch, or tool execution.
- Source-specific evidence ports.
- Customer-owned CRM, accounting, HR, support, identity, signing, or commerce
  writeback.
- Product-domain workflows such as escrow, lending, sourcing, or business ops.
- Deployment infrastructure, secret storage implementation, or cloud topology.

## Relationship to Runway

[Runway](../../runway/) is the sibling authority. It owns platform identity, runtime, and devops substrate. Commerce Rails owns commercial authority.

See [`kb/Architecture/Runway Movement Boundary.md`](kb/Architecture/Runway%20Movement%20Boundary.md) for the full authority table.

## Runway Boundary

Runway owns platform identity and runtime authority. Commerce Rails, inside
Movement, owns commercial authority.

Organizations are split by authority:

- Runway owns the canonical organization, users, membership, auth, and security
  configuration.
- Movement owns the customer commercial organization projection used for
  billing, subscriptions, entitlements, provider references, and reconciliation.

Stripe is also split by authority. Runway gets Stripe traffic safely to the app
with secrets, routing, deployment config, and observability. Movement verifies
and maps provider events, records `WebhookReceipt` values, applies idempotency
and replay gates, and decides accepted commercial state.

## Stack Position

```text
Reflective Commerce Rails
  -> Helms for approval and operator visibility
  -> Axiom for commerce Truths and invariants
  -> Organism for formation selection
  -> Converge for proposals, promoted facts, receipts, and replay
  -> Mosaic for specialist policy, evidence, analytics, memory, providers, and solvers
  -> Runway for deployment, secrets, auth, storage, telemetry
```

## Current Surface

The first contract crate is `commerce-rails-contracts`.

It defines the initial vocabulary:

- `PartnerAccount`
- `CustomerOrg`
- `AppListing`
- `AppInstallation`
- `Subscription`
- `EntitlementGrant`
- `RevenueShareAgreement`
- `PayoutObligation`
- `LedgerEntry`
- `WebhookReceipt`
- `CommercialCommand`
- `PartnerPiggyBackCommand`

Stripe Connect is intentionally absent from those names. Stripe-specific state
belongs behind the adapter boundary.

## Command Safety

Every consequential command is wrapped in a Commerce Rails command envelope
before it can execute. The envelope carries the command ID, idempotency key,
actor, scope, origin, policy requirement, HITL requirement, audit requirement,
and reconciliation requirement.

The first executable gear train is the partner piggy-back loop: list partner app,
install app for a customer, create subscription, grant entitlement, record
revenue share, and stage partner payout.

## Movement Terminology

Commerce Rails uses mechanical-watch language for the rail control model:

- `movement` is the whole Reflective-owned rail operating together.
- `mainspring` is accumulated commercial force waiting to be released.
- `gear train` is the deterministic commerce sequence.
- `escapement` is the controlled-release gate for consequential work.
- `balance` regulates timing, invariants, ledger balance, and reconciliation.
- `caliber` is a named precision profile for a rail path.
- `complication` is an optional advanced commerce behavior layered onto the
  core movement.

The metaphor applies to orchestration and safety. Business objects keep plain
names such as `Subscription`, `LedgerEntry`, and `WebhookReceipt`.

## Development

```sh
cargo check
cargo test
```
