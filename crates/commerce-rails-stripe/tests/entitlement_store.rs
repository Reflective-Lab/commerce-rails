//! Integration tests for `EntitlementStore` + `CommerceRails::is_entitled`.
//!
//! All tests are hermetic — no network, no Stripe API key required.
//! `CommerceRailsConfig::local()` leaves Stripe unconfigured; only the
//! in-memory entitlement store is exercised.

use commerce_rails_stripe::{
    BillingPlan, CommerceRails, CommerceRailsConfig, CommerceWebhookAction, SubscriptionProjection,
};

fn make_service() -> CommerceRails {
    let config = CommerceRailsConfig::local();
    // HERMETIC: client is never invoked; all paths under test touch only the
    // in-memory EntitlementStore, not the HTTP layer.
    #[allow(clippy::disallowed_methods)]
    let client = reqwest::Client::new();
    CommerceRails::new(client, config)
}

/// A brand-new service with no state must deny every entitlement query.
#[test]
fn fresh_service_says_not_entitled() {
    let service = make_service();
    assert!(!service.is_entitled("user-1", "quorum"));
}

/// Linking a customer ref alone (no subscription yet) must not grant
/// entitlements — there is no `SubscriptionProjection` in the store.
#[test]
fn link_customer_alone_does_not_entitle() {
    let service = make_service();
    service.apply_webhook_action(&CommerceWebhookAction::LinkCustomerRef {
        firebase_uid: "user-1".to_string(),
        customer_ref: "cus_abc".to_string(),
    });
    assert!(!service.is_entitled("user-1", "quorum"));
}

/// `LinkCustomerRef` followed by an active Starter subscription must grant
/// the `"quorum"` entitlement and only that entitlement (v1 scope).
#[test]
fn link_plus_active_starter_grants_quorum() {
    let service = make_service();
    service.apply_webhook_action(&CommerceWebhookAction::LinkCustomerRef {
        firebase_uid: "user-1".to_string(),
        customer_ref: "cus_abc".to_string(),
    });
    service.apply_webhook_action(&CommerceWebhookAction::ApplySubscriptionProjection {
        customer_ref: "cus_abc".to_string(),
        projection: SubscriptionProjection {
            plan: BillingPlan::Starter,
            subscription_status: "active".to_string(),
            ..Default::default()
        },
    });

    assert!(service.is_entitled("user-1", "quorum"));
    // No other app is granted in v1.
    assert!(!service.is_entitled("user-1", "wolfgang"));
}

/// After a subscription is canceled the entitlement must be revoked.
/// Sequence: link → active subscription (entitled) → cancel → not entitled.
#[test]
fn canceled_subscription_revokes_entitlement() {
    let service = make_service();
    service.apply_webhook_action(&CommerceWebhookAction::LinkCustomerRef {
        firebase_uid: "user-1".to_string(),
        customer_ref: "cus_abc".to_string(),
    });
    service.apply_webhook_action(&CommerceWebhookAction::ApplySubscriptionProjection {
        customer_ref: "cus_abc".to_string(),
        projection: SubscriptionProjection {
            plan: BillingPlan::Starter,
            subscription_status: "active".to_string(),
            ..Default::default()
        },
    });
    assert!(service.is_entitled("user-1", "quorum"), "pre-condition: entitled before cancel");

    service.apply_webhook_action(&CommerceWebhookAction::UpdateSubscriptionStatus {
        customer_ref: "cus_abc".to_string(),
        subscription_status: "canceled".to_string(),
    });
    assert!(!service.is_entitled("user-1", "quorum"));
}

/// `BillingPlan::Free` with an `"active"` status still must not grant any
/// app entitlements because `Free::apps()` returns an empty list.
#[test]
fn free_plan_does_not_grant_quorum() {
    let service = make_service();
    service.apply_webhook_action(&CommerceWebhookAction::LinkCustomerRef {
        firebase_uid: "user-1".to_string(),
        customer_ref: "cus_abc".to_string(),
    });
    service.apply_webhook_action(&CommerceWebhookAction::ApplySubscriptionProjection {
        customer_ref: "cus_abc".to_string(),
        projection: SubscriptionProjection {
            plan: BillingPlan::Free,
            subscription_status: "active".to_string(),
            ..Default::default()
        },
    });
    assert!(!service.is_entitled("user-1", "quorum"));
}
