---
tags: [architecture, authority, commerce]
source: mixed
---
# Operating Authority Boundary

Commerce Rails owns Reflective Labs commercial authority.

It is a business layer above Bedrock and Mosaic. It consumes the stack, but it
does not push Reflective business semantics downward into reusable platform
machinery.

## Owns

- Reflective billing and subscriptions.
- Customer org commercial state as a projection of a canonical Runway
  organization.
- Partner accounts and builder accounts.
- App listings and app installations.
- Entitlements across Reflective-hosted apps.
- Revenue-share agreements and partner payouts.
- Refunds, disputes, ledger entries, webhook receipts, and reconciliation.

## Does Not Own

- Canonical users, authentication, sessions, invites, roles, org membership, or
  tenant identity; those belong in Runway.
- Deployment, environment, runtime config, telemetry, and secret-storage
  authority; those belong in Runway.
- Source-specific evidence ports; those belong in Embassy.
- Generic provider, fetch, search, storage, vector, LLM, and tool capabilities;
  those belong in Manifold or Converge provider/tool contracts.
- Product-domain workflows such as escrow, lending, sourcing, or SMB ops.
- Customer-owned writeback to CRM, accounting, HR, support, signing, commerce,
  or identity systems.
- Deployment topology, secret-storage implementation, or cloud resources.

## Stack Use

| Layer | Commerce Rails use |
|---|---|
| Axiom | Commerce Truths, invariants, and compile-time checks |
| Organism | Formation selection for commercial intents |
| Converge | Proposals, promoted facts, receipts, audit, replay |
| Helms | HITL approvals, review, redirect, and operator visibility |
| Mosaic / Arbiter | Cedar policy, delegation, approval requirements |
| Mosaic / Embassy | Source-specific evidence where needed |
| Mosaic / Manifold | Generic provider and storage capabilities |
| Runway | Deployment, secrets, auth, storage, telemetry |

## Runway Rule

Runway answers who can act, where code runs, how secrets are accessed, and how
the runtime is operated.

Commerce Rails answers who pays, what is owed, what is granted, what is
refundable, what must be reconciled, and which commercial state is accepted.

## Rule

If Reflective bears the commercial consequence, Commerce Rails owns the
contract. Providers implement parts of the flow, but they do not define the
business model.
