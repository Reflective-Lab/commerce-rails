//! Stripe provider adapter for Commerce Rails.
//!
//! Runtime Runway owns HTTP ingress, auth context, deployment config, and the
//! eventually-consistent identity mirror. This crate owns the Stripe-specific
//! provider calls, signature mechanics, webhook receipt construction, and
//! commercial event interpretation.

#![forbid(unsafe_code)]

use chrono::Utc;
use commerce_rails_contracts::{
    CommerceId, ProviderName, ReplayKey, Timestamp, WebhookReceipt, WebhookReceiptStatus,
};
use hmac::{Hmac, Mac};
use serde::Deserialize;
use serde_json::Value;
use sha2::Sha256;
use std::fmt::Write as _;

const STRIPE_API_BASE: &str = "https://api.stripe.com/v1";
const WEBHOOK_TOLERANCE_SECONDS: i64 = 300;

#[derive(Debug, thiserror::Error)]
pub enum CommerceRailsError {
    #[error("commerce configuration error: {0}")]
    Configuration(String),
    #[error("stripe provider error: {0}")]
    Provider(String),
    #[error("invalid stripe webhook JSON: {0}")]
    InvalidWebhookJson(String),
}

impl CommerceRailsError {
    pub fn is_invalid_webhook_json(&self) -> bool {
        matches!(self, Self::InvalidWebhookJson(_))
    }
}

#[derive(Debug, Clone)]
pub struct CommerceRailsConfig {
    stripe: StripeConfig,
}

impl CommerceRailsConfig {
    pub fn new(
        stripe_webhook_secret: impl Into<String>,
        stripe_secret_key: impl Into<String>,
        stripe_price_team_monthly: impl Into<String>,
        stripe_price_starter_monthly: impl Into<String>,
    ) -> Self {
        Self {
            stripe: StripeConfig::new(
                stripe_webhook_secret,
                stripe_secret_key,
                stripe_price_team_monthly,
                stripe_price_starter_monthly,
            ),
        }
    }

    pub fn local() -> Self {
        Self::new("", "", "", "")
    }

    pub fn from_env(local_dev: bool) -> Result<Self, CommerceRailsError> {
        let stripe_webhook_secret = std::env::var("STRIPE_WEBHOOK_SECRET").unwrap_or_default();
        let stripe_secret_key = std::env::var("STRIPE_SECRET_KEY").unwrap_or_default();
        let stripe_price_team_monthly =
            std::env::var("STRIPE_PRICE_TEAM_MONTHLY").unwrap_or_default();
        let stripe_price_starter_monthly =
            std::env::var("STRIPE_PRICE_STARTER_MONTHLY").unwrap_or_default();

        if !local_dev {
            let missing = [
                ("STRIPE_WEBHOOK_SECRET", stripe_webhook_secret.as_str()),
                ("STRIPE_SECRET_KEY", stripe_secret_key.as_str()),
                (
                    "STRIPE_PRICE_TEAM_MONTHLY",
                    stripe_price_team_monthly.as_str(),
                ),
                (
                    "STRIPE_PRICE_STARTER_MONTHLY",
                    stripe_price_starter_monthly.as_str(),
                ),
            ]
            .into_iter()
            .filter_map(|(name, value)| value.trim().is_empty().then_some(name))
            .collect::<Vec<_>>();

            if !missing.is_empty() {
                return Err(CommerceRailsError::Configuration(format!(
                    "{} must be set in production",
                    missing.join(", ")
                )));
            }
        }

        Ok(Self::new(
            stripe_webhook_secret,
            stripe_secret_key,
            stripe_price_team_monthly,
            stripe_price_starter_monthly,
        ))
    }
}

#[derive(Debug, Clone)]
struct StripeConfig {
    webhook_secret: Option<String>,
    secret_key: Option<String>,
    price_team_monthly: Option<String>,
    price_starter_monthly: Option<String>,
}

impl StripeConfig {
    fn new(
        webhook_secret: impl Into<String>,
        secret_key: impl Into<String>,
        price_team_monthly: impl Into<String>,
        price_starter_monthly: impl Into<String>,
    ) -> Self {
        Self {
            webhook_secret: optional_string(webhook_secret),
            secret_key: optional_string(secret_key),
            price_team_monthly: optional_string(price_team_monthly),
            price_starter_monthly: optional_string(price_starter_monthly),
        }
    }

    fn plan_from_price_ids(&self, price_ids: &[&str]) -> BillingPlan {
        if let Some(team) = self.price_team_monthly.as_deref()
            && price_ids.contains(&team)
        {
            return BillingPlan::Team;
        }

        if let Some(starter) = self.price_starter_monthly.as_deref()
            && price_ids.contains(&starter)
        {
            return BillingPlan::Starter;
        }

        BillingPlan::Free
    }
}

fn optional_string(value: impl Into<String>) -> Option<String> {
    let value = value.into();
    if value.trim().is_empty() {
        None
    } else {
        Some(value)
    }
}

#[derive(Clone)]
pub struct CommerceRails {
    stripe: StripeAdapter,
}

impl CommerceRails {
    pub fn new(client: reqwest::Client, config: CommerceRailsConfig) -> Self {
        Self {
            stripe: StripeAdapter::new(client, config.stripe),
        }
    }

    pub fn is_billing_configured(&self) -> bool {
        self.stripe.is_configured()
    }

    pub async fn ensure_customer(
        &self,
        uid: &str,
        email: Option<&str>,
    ) -> Result<String, CommerceRailsError> {
        self.stripe.ensure_customer(uid, email).await
    }

    pub async fn create_checkout_session(
        &self,
        customer_ref: &str,
        price_ref: &str,
        success_url: &str,
        cancel_url: &str,
        firebase_uid: &str,
    ) -> Result<String, CommerceRailsError> {
        self.stripe
            .create_checkout_session(
                customer_ref,
                price_ref,
                "subscription",
                success_url,
                cancel_url,
                firebase_uid,
            )
            .await
    }

    pub async fn create_portal_session(
        &self,
        customer_ref: &str,
        return_url: &str,
    ) -> Result<String, CommerceRailsError> {
        self.stripe
            .create_portal_session(customer_ref, return_url)
            .await
    }

    pub fn verify_stripe_webhook_signature(&self, payload: &[u8], sig_header: &str) -> bool {
        self.stripe.verify_signature(payload, sig_header)
    }

    pub fn accept_stripe_webhook(
        &self,
        payload: &[u8],
    ) -> Result<AcceptedWebhook, CommerceRailsError> {
        self.stripe.accept_webhook(payload)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BillingPlan {
    #[default]
    Free,
    Starter,
    Team,
    Enterprise,
}

impl BillingPlan {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Free => "free",
            Self::Starter => "starter",
            Self::Team => "team",
            Self::Enterprise => "enterprise",
        }
    }

    pub fn apps(self) -> Vec<String> {
        match self {
            Self::Free => Vec::new(),
            Self::Starter | Self::Team | Self::Enterprise => vec!["marquee".to_string()],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubscriptionProjection {
    pub plan: BillingPlan,
    pub apps: Vec<String>,
    pub subscription_status: String,
    pub subscription_ref: Option<String>,
    pub current_period_end: Option<i64>,
}

impl SubscriptionProjection {
    fn updated(
        plan: BillingPlan,
        subscription_status: impl Into<String>,
        subscription_ref: impl Into<String>,
        current_period_end: Option<i64>,
    ) -> Self {
        Self {
            plan,
            apps: plan.apps(),
            subscription_status: subscription_status.into(),
            subscription_ref: Some(subscription_ref.into()),
            current_period_end,
        }
    }

    fn canceled() -> Self {
        Self {
            plan: BillingPlan::Free,
            apps: Vec::new(),
            subscription_status: "canceled".to_string(),
            subscription_ref: None,
            current_period_end: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommerceWebhookAction {
    LinkCustomerRef {
        firebase_uid: String,
        customer_ref: String,
    },
    ApplySubscriptionProjection {
        customer_ref: String,
        projection: SubscriptionProjection,
    },
    UpdateSubscriptionStatus {
        customer_ref: String,
        subscription_status: String,
    },
    Ignored,
}

#[derive(Debug, Clone)]
pub struct AcceptedWebhook {
    pub receipt: WebhookReceipt,
    pub event_type: String,
    pub action: CommerceWebhookAction,
}

#[derive(Clone)]
struct StripeAdapter {
    client: reqwest::Client,
    config: StripeConfig,
}

impl StripeAdapter {
    fn new(client: reqwest::Client, config: StripeConfig) -> Self {
        Self { client, config }
    }

    fn is_configured(&self) -> bool {
        self.config.secret_key.is_some()
    }

    fn key(&self) -> Result<&str, CommerceRailsError> {
        self.config.secret_key.as_deref().ok_or_else(|| {
            CommerceRailsError::Provider("STRIPE_SECRET_KEY not configured".to_string())
        })
    }

    async fn find_customer_ref(&self, uid: &str) -> Result<Option<String>, CommerceRailsError> {
        let Some(key) = self.config.secret_key.as_deref() else {
            return Ok(None);
        };
        let query = format!("metadata['firebase_uid']:'{}'", uid.replace('\'', "\\'"));
        let resp = self
            .client
            .get(format!("{STRIPE_API_BASE}/customers/search"))
            .bearer_auth(key)
            .query(&[("query", query.as_str())])
            .send()
            .await
            .map_err(|e| CommerceRailsError::Provider(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(CommerceRailsError::Provider(format!(
                "customer search failed: {}",
                resp.status()
            )));
        }

        let list: StripeList<StripeCustomer> = resp
            .json()
            .await
            .map_err(|e| CommerceRailsError::Provider(e.to_string()))?;
        Ok(list.data.into_iter().next().map(|customer| customer.id))
    }

    async fn ensure_customer(
        &self,
        uid: &str,
        email: Option<&str>,
    ) -> Result<String, CommerceRailsError> {
        if let Some(id) = self.find_customer_ref(uid).await? {
            return Ok(id);
        }

        let key = self.key()?;
        let mut form: Vec<(&str, String)> = vec![("metadata[firebase_uid]", uid.to_string())];
        if let Some(email) = email {
            form.push(("email", email.to_string()));
        }

        let resp = self
            .client
            .post(format!("{STRIPE_API_BASE}/customers"))
            .bearer_auth(key)
            .form(&form)
            .send()
            .await
            .map_err(|e| CommerceRailsError::Provider(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(CommerceRailsError::Provider(format!(
                "customer creation failed: {}",
                resp.status()
            )));
        }

        let customer: StripeCustomer = resp
            .json()
            .await
            .map_err(|e| CommerceRailsError::Provider(e.to_string()))?;
        Ok(customer.id)
    }

    async fn create_checkout_session(
        &self,
        customer_ref: &str,
        price_ref: &str,
        mode: &str,
        success_url: &str,
        cancel_url: &str,
        firebase_uid: &str,
    ) -> Result<String, CommerceRailsError> {
        let key = self.key()?;
        let idempotency_key = format!("checkout_{firebase_uid}_{}", uuid::Uuid::new_v4());
        let form: Vec<(&str, &str)> = vec![
            ("mode", mode),
            ("customer", customer_ref),
            ("success_url", success_url),
            ("cancel_url", cancel_url),
            ("line_items[0][price]", price_ref),
            ("line_items[0][quantity]", "1"),
            ("client_reference_id", firebase_uid),
            ("metadata[firebase_uid]", firebase_uid),
            ("allow_promotion_codes", "true"),
        ];

        let resp = self
            .client
            .post(format!("{STRIPE_API_BASE}/checkout/sessions"))
            .bearer_auth(key)
            .header("Idempotency-Key", &idempotency_key)
            .form(&form)
            .send()
            .await
            .map_err(|e| CommerceRailsError::Provider(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(CommerceRailsError::Provider(format!(
                "checkout session failed: {}",
                resp.status()
            )));
        }

        let session: StripeCheckoutSession = resp
            .json()
            .await
            .map_err(|e| CommerceRailsError::Provider(e.to_string()))?;
        session
            .url
            .ok_or_else(|| CommerceRailsError::Provider("no URL in checkout response".to_string()))
    }

    async fn create_portal_session(
        &self,
        customer_ref: &str,
        return_url: &str,
    ) -> Result<String, CommerceRailsError> {
        let key = self.key()?;
        let form: Vec<(&str, &str)> = vec![("customer", customer_ref), ("return_url", return_url)];
        let resp = self
            .client
            .post(format!("{STRIPE_API_BASE}/billing_portal/sessions"))
            .bearer_auth(key)
            .form(&form)
            .send()
            .await
            .map_err(|e| CommerceRailsError::Provider(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(CommerceRailsError::Provider(format!(
                "portal session failed: {}",
                resp.status()
            )));
        }

        let session: StripePortalSession = resp
            .json()
            .await
            .map_err(|e| CommerceRailsError::Provider(e.to_string()))?;
        Ok(session.url)
    }

    fn verify_signature(&self, payload: &[u8], sig_header: &str) -> bool {
        let Some(secret) = self.config.webhook_secret.as_deref() else {
            return true;
        };

        verify_stripe_signature(payload, sig_header, secret)
    }

    fn accept_webhook(&self, payload: &[u8]) -> Result<AcceptedWebhook, CommerceRailsError> {
        let event: Value = serde_json::from_slice(payload)
            .map_err(|e| CommerceRailsError::InvalidWebhookJson(e.to_string()))?;
        let event_id = event["id"]
            .as_str()
            .unwrap_or("missing-event-id")
            .to_string();
        let event_type = event["type"].as_str().unwrap_or("").to_string();
        let action = self.action_for_event(&event_type, &event);
        let receipt_status = if matches!(action, CommerceWebhookAction::Ignored) {
            WebhookReceiptStatus::Received
        } else {
            WebhookReceiptStatus::Accepted
        };

        Ok(AcceptedWebhook {
            receipt: WebhookReceipt {
                id: CommerceId::new(format!("webhook_receipt:stripe:{event_id}")),
                provider: ProviderName::StripeConnect,
                provider_event_id: event_id.clone(),
                replay_key: ReplayKey(format!("stripe:event:{event_id}")),
                signature_verified: true,
                received_at: Timestamp(Utc::now().to_rfc3339()),
                status: receipt_status,
            },
            event_type,
            action,
        })
    }

    fn action_for_event(&self, event_type: &str, event: &Value) -> CommerceWebhookAction {
        match event_type {
            "checkout.session.completed" => checkout_completed(event),
            "customer.subscription.created" | "customer.subscription.updated" => {
                self.subscription_updated(&event["data"]["object"])
            }
            "customer.subscription.deleted" => subscription_deleted(&event["data"]["object"]),
            "invoice.payment_failed" => invoice_payment_failed(&event["data"]["object"]),
            _ => CommerceWebhookAction::Ignored,
        }
    }

    fn subscription_updated(&self, subscription: &Value) -> CommerceWebhookAction {
        let Some(customer_ref) = subscription["customer"].as_str() else {
            tracing::warn!("subscription webhook missing customer");
            return CommerceWebhookAction::Ignored;
        };
        let Some(subscription_ref) = subscription["id"].as_str() else {
            tracing::warn!("subscription webhook missing subscription id");
            return CommerceWebhookAction::Ignored;
        };

        let status = subscription["status"].as_str().unwrap_or("unknown");
        let current_period_end = subscription["current_period_end"].as_i64();
        let price_ids: Vec<&str> = subscription["items"]["data"]
            .as_array()
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| item["price"]["id"].as_str())
                    .collect()
            })
            .unwrap_or_default();
        let plan = self.config.plan_from_price_ids(&price_ids);

        CommerceWebhookAction::ApplySubscriptionProjection {
            customer_ref: customer_ref.to_string(),
            projection: SubscriptionProjection::updated(
                plan,
                status,
                subscription_ref,
                current_period_end,
            ),
        }
    }
}

fn checkout_completed(event: &Value) -> CommerceWebhookAction {
    let session = &event["data"]["object"];
    let Some(firebase_uid) = session["client_reference_id"].as_str() else {
        tracing::warn!("checkout.session.completed missing client_reference_id");
        return CommerceWebhookAction::Ignored;
    };
    let Some(customer_ref) = session["customer"].as_str() else {
        tracing::warn!("checkout.session.completed missing customer");
        return CommerceWebhookAction::Ignored;
    };

    CommerceWebhookAction::LinkCustomerRef {
        firebase_uid: firebase_uid.to_string(),
        customer_ref: customer_ref.to_string(),
    }
}

fn subscription_deleted(subscription: &Value) -> CommerceWebhookAction {
    let Some(customer_ref) = subscription["customer"].as_str() else {
        tracing::warn!("subscription deleted webhook missing customer");
        return CommerceWebhookAction::Ignored;
    };

    CommerceWebhookAction::ApplySubscriptionProjection {
        customer_ref: customer_ref.to_string(),
        projection: SubscriptionProjection::canceled(),
    }
}

fn invoice_payment_failed(invoice: &Value) -> CommerceWebhookAction {
    let Some(customer_ref) = invoice["customer"].as_str() else {
        tracing::warn!("invoice payment failed webhook missing customer");
        return CommerceWebhookAction::Ignored;
    };

    CommerceWebhookAction::UpdateSubscriptionStatus {
        customer_ref: customer_ref.to_string(),
        subscription_status: "past_due".to_string(),
    }
}

fn verify_stripe_signature(payload: &[u8], sig_header: &str, secret: &str) -> bool {
    let mut timestamp: Option<&str> = None;
    let mut signatures: Vec<&str> = Vec::new();

    for part in sig_header.split(',') {
        let part = part.trim();
        if let Some(value) = part.strip_prefix("t=") {
            timestamp = Some(value);
        } else if let Some(value) = part.strip_prefix("v1=") {
            signatures.push(value);
        }
    }

    let Some(ts_str) = timestamp else {
        return false;
    };
    if signatures.is_empty() {
        return false;
    }
    let Ok(timestamp) = ts_str.parse::<i64>() else {
        return false;
    };
    if (Utc::now().timestamp() - timestamp).abs() > WEBHOOK_TOLERANCE_SECONDS {
        return false;
    }

    let signed_payload = format!("{ts_str}.{}", String::from_utf8_lossy(payload));
    let Ok(mut mac) = Hmac::<Sha256>::new_from_slice(secret.as_bytes()) else {
        return false;
    };
    mac.update(signed_payload.as_bytes());
    let expected = hex_lower(&mac.finalize().into_bytes());

    signatures.iter().any(|signature| {
        signature.len() == expected.len()
            && signature
                .bytes()
                .zip(expected.bytes())
                .fold(0_u8, |acc, (left, right)| acc | (left ^ right))
                == 0
    })
}

fn hex_lower(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(&mut output, "{byte:02x}").expect("writing to String cannot fail");
    }
    output
}

#[derive(Debug, Deserialize)]
struct StripeList<T> {
    data: Vec<T>,
}

#[derive(Debug, Deserialize)]
struct StripeCustomer {
    id: String,
}

#[derive(Debug, Deserialize)]
pub struct StripeSubscription {
    pub id: String,
    pub status: String,
    pub items: StripeSubscriptionItems,
    #[serde(default)]
    pub current_period_end: i64,
}

impl StripeSubscription {
    pub fn price_ids(&self) -> Vec<&str> {
        self.items
            .data
            .iter()
            .map(|item| item.price.id.as_str())
            .collect()
    }

    pub fn is_active(&self) -> bool {
        self.status == "active" || self.status == "trialing"
    }
}

#[derive(Debug, Deserialize)]
pub struct StripeSubscriptionItems {
    pub data: Vec<StripeSubscriptionItem>,
}

#[derive(Debug, Deserialize)]
pub struct StripeSubscriptionItem {
    pub price: StripePrice,
}

#[derive(Debug, Deserialize)]
pub struct StripePrice {
    pub id: String,
}

#[derive(Debug, Deserialize)]
struct StripeCheckoutSession {
    url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct StripePortalSession {
    url: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rails() -> CommerceRails {
        let config = CommerceRailsConfig::new("whsec_test", "", "price_team", "price_starter");
        // RP-HERMETIC-UNIT (QF-2026-06-02-05): the tests that use `rails()`
        // exercise signature verification and webhook parsing only — they
        // never invoke the HTTP path, so the client here is a sentinel. If
        // a future test needs to hit a stubbed Stripe API, wire a stub
        // client (e.g. backed by `wiremock`) via the existing
        // `CommerceRails::new(client, config)` DI constructor instead.
        #[allow(clippy::disallowed_methods)]
        let client = reqwest::Client::new();
        CommerceRails::new(client, config)
    }

    fn signature_header(payload: &[u8], secret: &str) -> String {
        let timestamp = Utc::now().timestamp();
        let signed_payload = format!("{timestamp}.{}", String::from_utf8_lossy(payload));
        let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(signed_payload.as_bytes());
        let signature = hex_lower(&mac.finalize().into_bytes());
        format!("t={timestamp},v1={signature}")
    }

    #[test]
    fn verifies_stripe_webhook_signature() {
        let payload = br#"{"id":"evt_test","type":"invoice.payment_failed"}"#;
        let header = signature_header(payload, "whsec_test");

        assert!(rails().verify_stripe_webhook_signature(payload, &header));
        assert!(!rails().verify_stripe_webhook_signature(payload, "t=1,v1=bad"));
    }

    #[test]
    fn maps_subscription_webhook_to_commercial_projection() {
        let payload = br#"{
            "id":"evt_sub_updated",
            "type":"customer.subscription.updated",
            "data":{
                "object":{
                    "id":"sub_123",
                    "customer":"cus_123",
                    "status":"active",
                    "current_period_end":12345,
                    "items":{"data":[{"price":{"id":"price_team"}}]}
                }
            }
        }"#;

        let webhook = rails().accept_stripe_webhook(payload).unwrap();

        assert_eq!(webhook.receipt.provider, ProviderName::StripeConnect);
        assert_eq!(webhook.receipt.provider_event_id, "evt_sub_updated");
        assert_eq!(webhook.receipt.status, WebhookReceiptStatus::Accepted);
        assert_eq!(
            webhook.action,
            CommerceWebhookAction::ApplySubscriptionProjection {
                customer_ref: "cus_123".to_string(),
                projection: SubscriptionProjection {
                    plan: BillingPlan::Team,
                    apps: vec!["marquee".to_string()],
                    subscription_status: "active".to_string(),
                    subscription_ref: Some("sub_123".to_string()),
                    current_period_end: Some(12345),
                },
            }
        );
    }

    #[test]
    fn maps_unknown_price_to_free_projection() {
        let payload = br#"{
            "id":"evt_sub_unknown",
            "type":"customer.subscription.updated",
            "data":{
                "object":{
                    "id":"sub_123",
                    "customer":"cus_123",
                    "status":"active",
                    "items":{"data":[{"price":{"id":"price_unknown"}}]}
                }
            }
        }"#;

        let webhook = rails().accept_stripe_webhook(payload).unwrap();

        assert!(matches!(
            webhook.action,
            CommerceWebhookAction::ApplySubscriptionProjection {
                projection: SubscriptionProjection {
                    plan: BillingPlan::Free,
                    ..
                },
                ..
            }
        ));
    }
}
