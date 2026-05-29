---
tags: [log]
source: mixed
---
# KB Mutation Log

| Date | File | Change | Author |
|---|---|---|---|
| 2026-05-29 | README.md, AGENTS.md, MILESTONES.md, kb/Architecture/*, kb/Home.md, kb/INDEX.md | Moved the repo to `~/dev/reflective/commerce-rails/` and removed the extra layer indirection from current docs | codex |
| 2026-05-28 | crates/commerce-rails-stripe/src/lib.rs, README.md, INDEX.md | Moved Stripe provider config, API calls, webhook signature mechanics, receipts, and event mapping out of Runway and into Commerce Rails | codex |
| 2026-05-17 | MILESTONES.md | Prepared M2 for Wolfgang as the deployed Runway application driver using Commerce Rails with Stripe and Make.com integrations | mixed |
| 2026-05-17 | crates/commerce-rails-contracts/src/lib.rs, MILESTONES.md | Completed M1 contract proof for installation, subscription entitlement, revenue-share payout obligation, and Stripe event receipt mapping | mixed |
| 2026-05-17 | crates/commerce-rails-contracts/src/lib.rs, MILESTONES.md | Started M1 with a contract test proving partner app listing, plan, price, and revenue-share representation | mixed |
| 2026-05-17 | crates/commerce-rails-contracts/src/lib.rs, MILESTONES.md | Added contract tests for CommerceId stability, provider refs, webhook replay keys, provider webhook command gates, command effects, and payout reconciliation | mixed |
| 2026-05-17 | Architecture/Executable Command Safety.md, README.md, AGENTS.md, Contracts/Commerce Rail Surface.md, Adapters/Stripe Connect Boundary.md, Home.md, INDEX.md, MILESTONES.md | Specified executable command safety and first partner piggy-back command loop | mixed |
| 2026-05-17 | Architecture/Runway Commerce Rails Boundary.md, README.md, AGENTS.md, Architecture/Operating Authority Boundary.md, Adapters/Stripe Connect Boundary.md, Contracts/Commerce Rail Surface.md, Home.md, INDEX.md | Documented Runway identity/runtime authority vs Commerce Rails commercial authority | mixed |
| 2026-05-17 | Architecture/Rail Terminology.md, README.md, AGENTS.md, Home.md, INDEX.md, Contracts/Commerce Rail Surface.md, MILESTONES.md | Added mechanical-watch terminology for the Commerce Rails control model | mixed |
| 2026-05-17 | Architecture/Operating Authority Boundary.md | Created Commerce Rails stack-placement boundary | mixed |
| 2026-05-17 | Contracts/Commerce Rail Surface.md | Created first contract surface | mixed |
| 2026-05-17 | Adapters/Stripe Connect Boundary.md | Decided Stripe Connect adapter boundary | mixed |
| 2026-05-17 | Home.md, INDEX.md | Created KB entrypoint and catalog | mixed |
