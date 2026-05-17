---
tags: [architecture, authority, runway, movement, commerce]
source: mixed
---
# Runway Movement Boundary

Runway owns platform identity and runtime authority. Movement owns Reflective
commercial authority.

The boundary is decided by who has authority over the consequence, not by which
system first receives a request or event.

## Ownership

| Area | Owner | Rule |
|---|---|---|
| Users | Runway | Canonical identity, authentication, sessions, invites, roles, and membership. |
| Organizations | Runway | Canonical tenant and organization container. |
| Customer commercial org | Movement | Commercial buyer/account projection of a Runway organization. |
| DevOps | Runway | Deployments, secrets, environments, runtime config, telemetry, and operational substrate. |
| Subscriptions | Movement | Plans, prices, subscription state, billing state, and entitlement grants. |
| Billing | Movement | Invoices, charges, refunds, revenue share, payout obligations, ledger, and reconciliation. |
| Stripe transport | Runway | Secret access, webhook ingress plumbing, deployment config, and runtime observability. |
| Stripe commerce adapter | Movement | Provider mapping, idempotency, webhook receipts, commercial state transitions, and reconciliation semantics. |

## Organization Model

Runway owns the login and tenancy container:

```text
RunwayOrg
  id
  name
  members
  auth and security configuration
```

Movement owns the commercial projection:

```text
CustomerOrg
  id
  runway_org_id
  legal or commercial name
  billing status
  provider refs
```

Runway answers who can act for an organization. Movement answers what that
organization can buy, owes, receives, or is entitled to use.

## Stripe Split

Stripe crosses the boundary, but the responsibilities are not shared
ambiguously.

```text
Stripe webhook HTTP request
  -> Runway routes it, provides secret access, and observes runtime health
  -> Movement Stripe adapter verifies provider semantics and records receipt
  -> Movement escapement applies idempotency, replay, policy, and HITL gates
  -> Movement updates Subscription, EntitlementGrant, LedgerEntry, or payout state
```

Runway gets the Stripe event safely to the application. Movement decides what
the Stripe event means commercially.

## Rule

If the question is who can log in, where code runs, where secrets live, or how
the runtime is operated, it belongs to Runway.

If the question is who pays, what is owed, what is granted, what is refundable,
what must be reconciled, or what commercial state is accepted, it belongs to
Movement.
