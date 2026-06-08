---
tags: [log]
source: mixed
---
# KB Mutation Log

| Date | File | Change | Author |
|---|---|---|---|
| 2026-05-30 | README.md, AGENTS.md, kb/Architecture/Runtime Runway Commerce Rails Boundary.md, kb/Home.md, kb/INDEX.md | Renamed the sibling platform authority from Runway to Runtime Runway after the workspace moved to `runtime-runway/` | codex |
| 2026-05-29 | README.md, AGENTS.md, MILESTONES.md, kb/Architecture/*, kb/Home.md, kb/INDEX.md | Moved the repo to `~/dev/reflective/commerce-rails/` and removed the extra layer indirection from current docs | codex |
| 2026-05-28 | crates/commerce-rails-stripe/src/lib.rs, README.md, INDEX.md | Moved Stripe provider config, API calls, webhook signature mechanics, receipts, and event mapping out of Runtime Runway and into Commerce Rails | codex |
| 2026-05-17 | MILESTONES.md | Prepared M2 for Wolfgang as the deployed Runtime Runway application driver using Commerce Rails with Stripe and Make.com integrations | mixed |
| 2026-05-17 | crates/commerce-rails-contracts/src/lib.rs, MILESTONES.md | Completed M1 contract proof for installation, subscription entitlement, revenue-share payout obligation, and Stripe event receipt mapping | mixed |
| 2026-05-17 | crates/commerce-rails-contracts/src/lib.rs, MILESTONES.md | Started M1 with a contract test proving partner app listing, plan, price, and revenue-share representation | mixed |
| 2026-05-17 | crates/commerce-rails-contracts/src/lib.rs, MILESTONES.md | Added contract tests for CommerceId stability, provider refs, webhook replay keys, provider webhook command gates, command effects, and payout reconciliation | mixed |
| 2026-05-17 | Architecture/Executable Command Safety.md, README.md, AGENTS.md, Contracts/Commerce Rail Surface.md, Adapters/Stripe Connect Boundary.md, Home.md, INDEX.md, MILESTONES.md | Specified executable command safety and first partner piggy-back command loop | mixed |
| 2026-05-17 | Architecture/Runtime Runway Commerce Rails Boundary.md, README.md, AGENTS.md, Architecture/Operating Authority Boundary.md, Adapters/Stripe Connect Boundary.md, Contracts/Commerce Rail Surface.md, Home.md, INDEX.md | Documented Runtime Runway identity/runtime authority vs Commerce Rails commercial authority | mixed |
| 2026-05-17 | Architecture/Rail Terminology.md, README.md, AGENTS.md, Home.md, INDEX.md, Contracts/Commerce Rail Surface.md, MILESTONES.md | Added mechanical-watch terminology for the Commerce Rails control model | mixed |
| 2026-05-17 | Architecture/Operating Authority Boundary.md | Created Commerce Rails stack-placement boundary | mixed |
| 2026-05-17 | Contracts/Commerce Rail Surface.md | Created first contract surface | mixed |
| 2026-05-17 | Adapters/Stripe Connect Boundary.md | Decided Stripe Connect adapter boundary | mixed |
| 2026-05-17 | Home.md, INDEX.md | Created KB entrypoint and catalog | mixed |

## 2026-06-08 — Plan 4 (Track B): EntitlementStore + is_entitled API landed on `next`

Closes the Commerce Rails gap that Quorum will consume (Plan 3b, in
the quorum-sense repo). The existing M1 work shipped
`accept_stripe_webhook` which returns a typed `CommerceWebhookAction`,
but nothing actually persisted those actions or answered "is this
user entitled?" Plan 4 adds:

- `EntitlementStore` — in-memory mappings of `firebase_uid` →
  `customer_ref` and `customer_ref` → `SubscriptionProjection`,
  updated by the 3 concrete `CommerceWebhookAction` variants
  (`LinkCustomerRef`, `ApplySubscriptionProjection`,
  `UpdateSubscriptionStatus`).
- `CommerceRails::apply_webhook_action(&action) -> bool` — the
  webhook handler's persist call.
- `CommerceRails::is_entitled(firebase_uid, app) -> bool` — active
  subscription + plan grants the app. Active = `subscription_status`
  in {`"active"`, `"trialing"`}.
- `BillingPlan::apps()` updated to return `"quorum"` instead of the
  `"marquee"` placeholder. v1 has a single paid product; all paid
  plans (Starter, Team, Enterprise) grant Quorum. When app #2 ships,
  extend the per-plan list.

The store is `Arc<EntitlementStore>` on `CommerceRails` (the service
derives `Clone` and `Mutex<HashMap>` isn't `Clone`-safe — Arc keeps
shared state across clones, matching how `reqwest::Client` works
internally).

5 integration tests cover the lifecycle: fresh service denies;
LinkCustomerRef alone is insufficient (no subscription); active
Starter + link grants Quorum but not other apps; cancellation
revokes; Free plan never grants Quorum.

In-memory only. v2 promotes the store to StorageKit-backed
persistence so state survives restarts. For Karl's friends/family v1
audience, in-memory + signup-loop testing is acceptable — production
deployment + persistence is a separate plan.

Held for follow-ups:
- Persistent storage (StorageKit-backed)
- Per-plan app lists (Wolfgang and other future apps)
- Webhook replay from Stripe for cold-start state rehydration
- HTTP wrapper service (today this stays a library; the consumer
  embeds it OR builds its own HTTP wrapper)

Spec: `marquee-apps/quorum-sense/docs/superpowers/specs/2026-06-06-quorum-shippable-v1-design.md`.
Plan: `docs/superpowers/plans/2026-06-08-plan-4-labs-entitlement-store.md`.

Unblocks Plan 3b (in quorum-sense repo): Quorum's
`/api/session/start` adds `commerce-rails-stripe` as a dep and calls
`is_entitled(firebase_uid, "quorum")` as a gate before opening a
session. Without this plan, that gate had nowhere to call.
