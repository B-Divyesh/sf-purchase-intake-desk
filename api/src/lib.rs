//! Durable, tenant-scoped API. The browser cache is only an offline convenience.
use axum::{
    body::Body,
    extract::{Path as AxumPath, State},
    http::{
        header::{
            AUTHORIZATION, CACHE_CONTROL, CONTENT_SECURITY_POLICY, REFERRER_POLICY,
            WWW_AUTHENTICATE, X_CONTENT_TYPE_OPTIONS,
        },
        HeaderMap, HeaderValue, Request, StatusCode,
    },
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use base64::{engine::general_purpose::STANDARD, Engine};
use chrono::{SecondsFormat, Utc};
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
#[cfg(test)]
use std::collections::HashMap;
use std::{
    env, fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::sync::RwLock;
use tower_governor::{
    governor::GovernorConfigBuilder, key_extractor::SmartIpKeyExtractor, GovernorLayer,
};
use tower_http::{
    services::{ServeDir, ServeFile},
    set_header::SetResponseHeaderLayer,
    trace::TraceLayer,
};
use tracing::warn;
use uuid::Uuid;

const TENANT_ID: &str = "35c6fe40-0ec0-46b6-98c6-213ad4de6650";
const CLIENT_ID: &str = "25c704f4-465a-47af-80ab-2c489466b697";
const SUBDOMAIN: &str = "sociobotcustomers";
#[derive(Clone)]
struct AppState {
    db: Arc<Mutex<Connection>>,
    data_dir: PathBuf,
    auth: AuthVerifier,
    #[cfg(test)]
    test_identities: Arc<HashMap<String, Identity>>,
}
#[derive(Clone)]
struct AuthVerifier {
    client: reqwest::Client,
    cache: Arc<RwLock<Option<CachedIdentityKeys>>>,
}
#[derive(Clone)]
struct CachedIdentityKeys {
    issuer: String,
    jwks: Jwks,
    fetched_at: Instant,
}
#[derive(Serialize)]
struct Health {
    status: &'static str,
    build_sha: &'static str,
}
#[derive(Serialize)]
struct ApiError {
    code: &'static str,
    message: String,
    action: &'static str,
}
#[derive(Deserialize)]
struct Claims {
    oid: String,
    tid: String,
    aud: Value,
    iss: String,
    exp: usize,
    nbf: Option<usize>,
    name: Option<String>,
    email: Option<String>,
}
#[derive(Deserialize)]
struct Discovery {
    issuer: String,
    jwks_uri: String,
}
#[derive(Clone, Deserialize)]
struct Jwks {
    keys: Vec<Jwk>,
}
#[derive(Clone, Deserialize)]
struct Jwk {
    kid: String,
    n: String,
    e: String,
    kty: String,
}
#[derive(Clone)]
struct Identity {
    oid: String,
    name: String,
    email: Option<String>,
}
#[derive(Deserialize)]
struct PurchaseOrder {
    purchase_order_id: String,
    po_number: String,
    supplier: String,
    #[serde(default)]
    payload: Value,
}
#[derive(Deserialize)]
struct Receipt {
    site_id: String,
    purchase_order_id: String,
    receipt_id: String,
    #[serde(default)]
    payload: Value,
}
#[derive(Deserialize)]
struct License {
    license: String,
}
#[derive(Deserialize)]
struct Attachment {
    #[serde(default)]
    id: Option<String>,
    name: String,
    media_type: String,
    data_base64: String,
}
#[derive(Deserialize)]
struct Member {
    oid: String,
    #[serde(default = "receiver_role")]
    role: String,
}
#[derive(Deserialize)]
struct Correction {
    reason: String,
}
fn receiver_role() -> String {
    "receiver".into()
}

pub fn build_sha() -> &'static str {
    option_env!("BUILD_SHA")
        .filter(|v| !v.is_empty())
        .or(option_env!("GIT_SHA").filter(|v| !v.is_empty()))
        .or(option_env!("SOURCE_COMMIT").filter(|v| !v.is_empty()))
        .unwrap_or("dev")
}
fn env_value(name: &str, fallback: &str) -> String {
    env::var(name).unwrap_or_else(|_| fallback.to_owned())
}
fn now() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)
}
fn digest(parts: &[&str]) -> String {
    let mut h = Sha256::new();
    for part in parts {
        h.update(part.as_bytes())
    }
    format!("{:x}", h.finalize())
}
fn digest_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn audit_hash(
    id: &str,
    sequence: i64,
    event_type: &str,
    at: &str,
    actor: &str,
    summary: &str,
    previous_hash: &str,
) -> String {
    let serialized = format!(
        "{{\"id\":{},\"sequence\":{},\"type\":{},\"at\":{},\"actor\":{},\"summary\":{},\"previousHash\":{}}}",
        serde_json::to_string(id).expect("serialize event id"),
        sequence,
        serde_json::to_string(event_type).expect("serialize event type"),
        serde_json::to_string(at).expect("serialize event time"),
        serde_json::to_string(actor).expect("serialize event actor"),
        serde_json::to_string(summary).expect("serialize event summary"),
        serde_json::to_string(previous_hash).expect("serialize previous hash"),
    );
    digest(&[&serialized])
}
fn fail(
    status: StatusCode,
    code: &'static str,
    message: impl Into<String>,
    action: &'static str,
) -> Response {
    let mut response = (
        status,
        Json(ApiError {
            code,
            message: message.into(),
            action,
        }),
    )
        .into_response();
    if status == StatusCode::UNAUTHORIZED {
        response.headers_mut().insert(
            WWW_AUTHENTICATE,
            HeaderValue::from_static("Bearer realm=\"Intake Desk\""),
        );
    }
    response
}

fn create_schema(db: &Connection) -> rusqlite::Result<()> {
    db.execute_batch(
        "PRAGMA foreign_keys=ON;
         CREATE TABLE IF NOT EXISTS tenants(id TEXT PRIMARY KEY,name TEXT NOT NULL,created_at TEXT NOT NULL);
         CREATE TABLE IF NOT EXISTS sites(id TEXT PRIMARY KEY,tenant_id TEXT NOT NULL,name TEXT NOT NULL);
         CREATE TABLE IF NOT EXISTS memberships(tenant_id TEXT NOT NULL,oid TEXT NOT NULL,role TEXT NOT NULL,PRIMARY KEY(tenant_id,oid));
         CREATE TABLE IF NOT EXISTS purchase_orders_scoped(id TEXT NOT NULL,tenant_id TEXT NOT NULL,site_id TEXT NOT NULL,po_number TEXT NOT NULL,supplier TEXT NOT NULL,payload TEXT NOT NULL,created_at TEXT NOT NULL,updated_at TEXT NOT NULL,PRIMARY KEY(tenant_id,site_id,id));
         CREATE TABLE IF NOT EXISTS receipts_scoped(id TEXT NOT NULL,tenant_id TEXT NOT NULL,site_id TEXT NOT NULL,purchase_order_id TEXT NOT NULL,state TEXT NOT NULL,payload TEXT NOT NULL,finalized_at TEXT,created_at TEXT NOT NULL,updated_at TEXT NOT NULL,PRIMARY KEY(tenant_id,site_id,id));
         CREATE TABLE IF NOT EXISTS receipt_events_scoped(id TEXT NOT NULL,tenant_id TEXT NOT NULL,site_id TEXT NOT NULL,receipt_id TEXT NOT NULL,sequence INTEGER NOT NULL,event_type TEXT NOT NULL,payload TEXT NOT NULL,previous_hash TEXT NOT NULL,event_hash TEXT NOT NULL,created_at TEXT NOT NULL,PRIMARY KEY(tenant_id,site_id,receipt_id,sequence),UNIQUE(tenant_id,id));
         CREATE TABLE IF NOT EXISTS attachments_scoped(id TEXT NOT NULL,tenant_id TEXT NOT NULL,site_id TEXT NOT NULL,receipt_id TEXT NOT NULL,name TEXT NOT NULL,object_key TEXT NOT NULL,checksum TEXT NOT NULL,media_type TEXT NOT NULL,bytes INTEGER NOT NULL,retention_until TEXT NOT NULL,created_at TEXT NOT NULL,PRIMARY KEY(tenant_id,site_id,id));
         CREATE TABLE IF NOT EXISTS entitlements_scoped(site_id TEXT NOT NULL,tenant_id TEXT NOT NULL,license_fingerprint TEXT NOT NULL,status TEXT NOT NULL,checked_at TEXT NOT NULL,expires_at TEXT,PRIMARY KEY(tenant_id,site_id));
         CREATE TABLE IF NOT EXISTS idempotency_keys(tenant_id TEXT NOT NULL,oid TEXT NOT NULL,key TEXT NOT NULL,request_hash TEXT NOT NULL,response TEXT NOT NULL,PRIMARY KEY(tenant_id,oid,key));",
    )?;
    // Repair-3 leaves the old globally-keyed tables untouched but copies their data once
    // into tenant-scoped tables. All runtime reads and writes use only the scoped tables.
    let old_exists: bool = db.query_row(
        "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='purchase_orders')",
        [],
        |row| row.get(0),
    )?;
    if old_exists {
        db.execute_batch(
            "INSERT OR IGNORE INTO purchase_orders_scoped(id,tenant_id,site_id,po_number,supplier,payload,created_at,updated_at) SELECT id,tenant_id,site_id,po_number,supplier,payload,created_at,created_at FROM purchase_orders;
             INSERT OR IGNORE INTO receipts_scoped(id,tenant_id,site_id,purchase_order_id,state,payload,finalized_at,created_at,updated_at) SELECT id,tenant_id,site_id,purchase_order_id,state,payload,finalized_at,created_at,created_at FROM receipts;
             INSERT OR IGNORE INTO receipt_events_scoped(id,tenant_id,site_id,receipt_id,sequence,event_type,payload,previous_hash,event_hash,created_at) SELECT e.id,e.tenant_id,r.site_id,e.receipt_id,e.sequence,e.event_type,e.payload,e.previous_hash,e.event_hash,e.created_at FROM receipt_events e JOIN receipts r ON r.id=e.receipt_id AND r.tenant_id=e.tenant_id;
             INSERT OR IGNORE INTO attachments_scoped(id,tenant_id,site_id,receipt_id,name,object_key,checksum,media_type,bytes,retention_until,created_at) SELECT a.id,a.tenant_id,r.site_id,a.receipt_id,'Retained evidence',a.object_key,a.checksum,a.media_type,a.bytes,a.retention_until,a.created_at FROM attachments a JOIN receipts r ON r.id=a.receipt_id AND r.tenant_id=a.tenant_id;
             INSERT OR IGNORE INTO entitlements_scoped(site_id,tenant_id,license_fingerprint,status,checked_at,expires_at) SELECT site_id,tenant_id,license_fingerprint,status,checked_at,expires_at FROM entitlements;",
        )?;
    }
    Ok(())
}

fn write_backup(db: &Connection, data_dir: &Path) -> Result<(), String> {
    let pending = data_dir.join("backup-pending.sqlite3");
    let latest = data_dir.join("backup-latest.sqlite3");
    match fs::remove_file(&pending) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.to_string()),
    }
    let mut destination = Connection::open(&pending).map_err(|error| error.to_string())?;
    {
        let backup = rusqlite::backup::Backup::new(db, &mut destination)
            .map_err(|error| error.to_string())?;
        backup
            .run_to_completion(16, Duration::from_millis(20), None)
            .map_err(|error| error.to_string())?;
    }
    destination
        .execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")
        .map_err(|error| error.to_string())?;
    drop(destination);
    fs::rename(&pending, &latest).map_err(|error| error.to_string())
}

fn configure_sqlite(db: &Connection) -> rusqlite::Result<()> {
    db.busy_timeout(Duration::from_secs(30))?;
    let journal_mode: String = db.query_row("PRAGMA journal_mode", [], |row| row.get(0))?;
    if !journal_mode.eq_ignore_ascii_case("wal") {
        db.pragma_update(None, "journal_mode", "WAL")?;
    }
    Ok(())
}

fn open_state() -> AppState {
    let wanted = PathBuf::from(env::var("DATA_DIR").unwrap_or_else(|_| "/data".into()));
    let directory = if fs::create_dir_all(wanted.join("objects")).is_ok() {
        wanted
    } else {
        warn!("DATA_DIR unavailable; using ./data");
        let fallback = PathBuf::from("data");
        fs::create_dir_all(fallback.join("objects")).expect("create data directory");
        fallback
    };
    let db = Connection::open(directory.join("intake-desk.sqlite3")).expect("open database");
    configure_sqlite(&db).expect("configure database");
    create_schema(&db).expect("create schema");
    AppState {
        db: Arc::new(Mutex::new(db)),
        data_dir: directory,
        auth: AuthVerifier {
            client: reqwest::Client::new(),
            cache: Arc::new(RwLock::new(None)),
        },
        #[cfg(test)]
        test_identities: Arc::new(HashMap::new()),
    }
}

#[allow(clippy::result_large_err)]
async fn identity(state: &AppState, headers: &HeaderMap) -> Result<Identity, Response> {
    let token = headers
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(|| {
            fail(
                StatusCode::UNAUTHORIZED,
                "unauthorized",
                "Sign in to access this site.",
                "sign_in",
            )
        })?;
    #[cfg(test)]
    if let Some(identity) = state.test_identities.get(token) {
        return Ok(identity.clone());
    }
    let tenant = env_value("ENTRA_TENANT_ID", TENANT_ID);
    let client = env_value("ENTRA_CLIENT_ID", CLIENT_ID);
    let subdomain = env_value("ENTRA_TENANT_SUBDOMAIN", SUBDOMAIN);
    let cached = state.auth.cache.read().await.clone();
    let keys = if let Some(cached) =
        cached.filter(|value| value.fetched_at.elapsed() < Duration::from_secs(3600))
    {
        cached
    } else {
        let discovery: Discovery = state
            .auth
            .client
            .get(format!(
                "https://{subdomain}.ciamlogin.com/{tenant}/v2.0/.well-known/openid-configuration"
            ))
            .send()
            .await
            .map_err(|_| {
                fail(
                    StatusCode::UNAUTHORIZED,
                    "identity_unavailable",
                    "Identity verification is temporarily unavailable.",
                    "retry",
                )
            })?
            .json()
            .await
            .map_err(|_| {
                fail(
                    StatusCode::UNAUTHORIZED,
                    "identity_unavailable",
                    "Identity verification is temporarily unavailable.",
                    "retry",
                )
            })?;
        let jwks: Jwks = state
            .auth
            .client
            .get(discovery.jwks_uri)
            .send()
            .await
            .map_err(|_| {
                fail(
                    StatusCode::UNAUTHORIZED,
                    "identity_unavailable",
                    "Identity verification is temporarily unavailable.",
                    "retry",
                )
            })?
            .json()
            .await
            .map_err(|_| {
                fail(
                    StatusCode::UNAUTHORIZED,
                    "identity_unavailable",
                    "Identity verification is temporarily unavailable.",
                    "retry",
                )
            })?;
        let cached = CachedIdentityKeys {
            issuer: discovery.issuer,
            jwks,
            fetched_at: Instant::now(),
        };
        *state.auth.cache.write().await = Some(cached.clone());
        cached
    };
    let header = decode_header(token).map_err(|_| {
        fail(
            StatusCode::UNAUTHORIZED,
            "invalid_token",
            "The sign-in token is malformed.",
            "sign_in",
        )
    })?;
    let jwk = keys
        .jwks
        .keys
        .iter()
        .find(|k| Some(&k.kid) == header.kid.as_ref() && k.kty == "RSA")
        .ok_or_else(|| {
            fail(
                StatusCode::UNAUTHORIZED,
                "invalid_token",
                "The sign-in key is not recognized.",
                "sign_in",
            )
        })?;
    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_audience(&[client.as_str()]);
    validation.set_issuer(&[keys.issuer.as_str()]);
    let decoded = decode::<Claims>(
        token,
        &DecodingKey::from_rsa_components(&jwk.n, &jwk.e).map_err(|_| {
            fail(
                StatusCode::UNAUTHORIZED,
                "invalid_token",
                "The sign-in key is invalid.",
                "sign_in",
            )
        })?,
        &validation,
    )
    .map_err(|_| {
        fail(
            StatusCode::UNAUTHORIZED,
            "invalid_token",
            "Your sign-in has expired. Sign in again.",
            "sign_in",
        )
    })?;
    if decoded.claims.tid != tenant {
        return Err(fail(
            StatusCode::UNAUTHORIZED,
            "invalid_token",
            "This token belongs to another tenant.",
            "sign_in",
        ));
    };
    let _ = (
        &decoded.claims.aud,
        &decoded.claims.iss,
        decoded.claims.exp,
        decoded.claims.nbf,
    );
    Ok(Identity {
        oid: decoded.claims.oid,
        name: decoded
            .claims
            .name
            .unwrap_or_else(|| "Receiving team member".into()),
        email: decoded.claims.email,
    })
}
fn provision(db: &Connection, user: &Identity) -> rusqlite::Result<(String, String)> {
    let existing = db
        .query_row(
            "SELECT m.tenant_id,s.id FROM memberships m JOIN sites s ON s.tenant_id=m.tenant_id WHERE m.oid=?1 ORDER BY CASE m.role WHEN 'owner' THEN 0 ELSE 1 END,s.id LIMIT 1",
            params![user.oid],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
        )
        .optional()?;
    if let Some(existing) = existing {
        return Ok(existing);
    }
    let tenant = format!("tenant-{}", user.oid);
    let site = format!("site-{}", user.oid);
    db.execute(
        "INSERT OR IGNORE INTO tenants VALUES(?1,?2,?3)",
        params![tenant, format!("{}'s team", user.name), now()],
    )?;
    db.execute(
        "INSERT OR IGNORE INTO sites VALUES(?1,?2,'Receiving site')",
        params![site, tenant],
    )?;
    db.execute(
        "INSERT OR IGNORE INTO memberships VALUES(?1,?2,'owner')",
        params![tenant, user.oid],
    )?;
    Ok((tenant, site))
}
#[allow(clippy::result_large_err)]
fn allowed_site(
    db: &Connection,
    user: &Identity,
    site: &str,
) -> Result<(String, String), Response> {
    provision(db, user).map_err(|_| {
        fail(
            StatusCode::INTERNAL_SERVER_ERROR,
            "storage_error",
            "Could not open the receiving site.",
            "retry",
        )
    })?;
    let found:Option<(String,String)>=db.query_row("SELECT s.tenant_id,s.id FROM sites s JOIN memberships m ON m.tenant_id=s.tenant_id WHERE s.id=?1 AND m.oid=?2",params![site,user.oid],|r|Ok((r.get(0)?,r.get(1)?))).optional().unwrap_or(None);
    found.ok_or_else(|| {
        fail(
            StatusCode::FORBIDDEN,
            "site_forbidden",
            "You do not have access to this receiving site.",
            "choose_site",
        )
    })
}

fn entitlement_status(db: &Connection, tenant: &str, site: &str) -> String {
    let entitlement: Option<(String, Option<String>)> = db
        .query_row(
            "SELECT status,expires_at FROM entitlements_scoped WHERE tenant_id=?1 AND site_id=?2",
            params![tenant, site],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .unwrap_or(None);
    match entitlement {
        Some((status, Some(expires_at))) if status == "active" => {
            match chrono::DateTime::parse_from_rfc3339(&expires_at) {
                Ok(expiry) if expiry <= Utc::now() => "expired".into(),
                _ => status,
            }
        }
        Some((status, _)) => status,
        None => "unavailable".into(),
    }
}

#[allow(clippy::result_large_err)]
fn require_active_entitlement(db: &Connection, tenant: &str, site: &str) -> Result<(), Response> {
    if entitlement_status(db, tenant, site) == "active" {
        Ok(())
    } else {
        Err(fail(
            StatusCode::PAYMENT_REQUIRED,
            "site_read_only",
            "This Dock site is read-only because checkout is not active.",
            "export_records",
        ))
    }
}

async fn health() -> Json<Health> {
    Json(Health {
        status: "ok",
        build_sha: build_sha(),
    })
}
async fn ready(State(state): State<AppState>) -> Response {
    if state.data_dir.join("intake-desk.sqlite3").exists() {
        Json(json!({"status":"ready","build_sha":build_sha()})).into_response()
    } else {
        fail(
            StatusCode::SERVICE_UNAVAILABLE,
            "not_ready",
            "Storage is not ready.",
            "retry",
        )
    }
}
async fn me(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let user = match identity(&state, &headers).await {
        Ok(v) => v,
        Err(e) => return e,
    };
    let db = state.db.lock().expect("db lock");
    match provision(&db, &user) {
        Ok((tenant, site)) => {
            let entitlement = entitlement_status(&db, &tenant, &site);
            Json(json!({"oid":user.oid,"name":user.name,"email":user.email,"tenant_id":tenant,"site_id":site,"entitlement":entitlement})).into_response()
        }
        Err(_) => fail(
            StatusCode::INTERNAL_SERVER_ERROR,
            "storage_error",
            "Could not provision your team.",
            "retry",
        ),
    }
}
async fn list_pos(
    State(state): State<AppState>,
    AxumPath(site): AxumPath<String>,
    headers: HeaderMap,
) -> Response {
    let user = match identity(&state, &headers).await {
        Ok(v) => v,
        Err(e) => return e,
    };
    let db = state.db.lock().expect("db lock");
    let (tenant, site) = match allowed_site(&db, &user, &site) {
        Ok(v) => v,
        Err(e) => return e,
    };
    let mut statement=match db.prepare("SELECT payload FROM purchase_orders_scoped WHERE tenant_id=?1 AND site_id=?2 ORDER BY updated_at DESC"){Ok(v)=>v,Err(_)=>return fail(StatusCode::INTERNAL_SERVER_ERROR,"storage_error","Could not list purchase orders.","retry")};
    let items = statement
        .query_map(params![tenant, site], |r| r.get::<_, String>(0))
        .ok()
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .filter_map(|v| serde_json::from_str::<Value>(&v).ok())
        .collect::<Vec<_>>();
    Json(json!({"purchase_orders":items})).into_response()
}
async fn create_po(
    State(state): State<AppState>,
    AxumPath(site): AxumPath<String>,
    headers: HeaderMap,
    Json(po): Json<PurchaseOrder>,
) -> Response {
    let user = match identity(&state, &headers).await {
        Ok(v) => v,
        Err(e) => return e,
    };
    if po.purchase_order_id.trim().is_empty() || po.po_number.trim().is_empty() {
        return fail(
            StatusCode::BAD_REQUEST,
            "invalid_purchase_order",
            "A purchase order ID and number are required.",
            "fix_fields",
        );
    };
    let db = state.db.lock().expect("db lock");
    let (tenant, site) = match allowed_site(&db, &user, &site) {
        Ok(v) => v,
        Err(e) => return e,
    };
    if let Err(response) = require_active_entitlement(&db, &tenant, &site) {
        return response;
    }
    let created_at = now();
    match db.execute("INSERT INTO purchase_orders_scoped(id,tenant_id,site_id,po_number,supplier,payload,created_at,updated_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?7) ON CONFLICT(tenant_id,site_id,id) DO UPDATE SET payload=excluded.payload,po_number=excluded.po_number,supplier=excluded.supplier,updated_at=excluded.updated_at",params![po.purchase_order_id,tenant,site,po.po_number,po.supplier,po.payload.to_string(),created_at]){Ok(_)=>Json(po.payload).into_response(),Err(_)=>fail(StatusCode::CONFLICT,"purchase_order_conflict","This purchase order could not be saved to this site.","retry")}
}
async fn get_po(
    State(state): State<AppState>,
    AxumPath((site, id)): AxumPath<(String, String)>,
    headers: HeaderMap,
) -> Response {
    let user = match identity(&state, &headers).await {
        Ok(v) => v,
        Err(e) => return e,
    };
    let db = state.db.lock().expect("db lock");
    let (tenant, site) = match allowed_site(&db, &user, &site) {
        Ok(v) => v,
        Err(response) => return response,
    };
    let value:Option<String>=db.query_row("SELECT payload FROM purchase_orders_scoped WHERE id=?1 AND tenant_id=?2 AND site_id=?3",params![id,tenant,site],|r|r.get(0)).optional().unwrap_or(None);
    value
        .and_then(|v| serde_json::from_str::<Value>(&v).ok())
        .map(|v| Json(v).into_response())
        .unwrap_or_else(|| {
            fail(
                StatusCode::NOT_FOUND,
                "purchase_order_not_found",
                "This purchase order is not in your site.",
                "open_inbox",
            )
        })
}
async fn save_receipt(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<Receipt>,
) -> Response {
    let user = match identity(&state, &headers).await {
        Ok(v) => v,
        Err(e) => return e,
    };
    let db = state.db.lock().expect("db lock");
    let (tenant, site) = match allowed_site(&db, &user, &input.site_id) {
        Ok(v) => v,
        Err(response) => return response,
    };
    if let Err(response) = require_active_entitlement(&db, &tenant, &site) {
        return response;
    }
    let owns: Option<String> = db
        .query_row(
            "SELECT id FROM purchase_orders_scoped WHERE id=?1 AND tenant_id=?2 AND site_id=?3",
            params![input.purchase_order_id, tenant, site],
            |r| r.get(0),
        )
        .optional()
        .unwrap_or(None);
    if owns.is_none() {
        return fail(
            StatusCode::FORBIDDEN,
            "purchase_order_forbidden",
            "That purchase order is not in your site.",
            "open_inbox",
        );
    };
    let existing: Option<String> = db
        .query_row(
            "SELECT state FROM receipts_scoped WHERE tenant_id=?1 AND site_id=?2 AND id=?3",
            params![tenant, site, input.receipt_id],
            |r| r.get(0),
        )
        .optional()
        .unwrap_or(None);
    if existing.as_deref() == Some("finalized") {
        return fail(
            StatusCode::CONFLICT,
            "receipt_finalized",
            "Finalized receipts cannot be changed. Add a correction event.",
            "create_correction",
        );
    };
    let saved_at = now();
    match db.execute("INSERT INTO receipts_scoped(id,tenant_id,site_id,purchase_order_id,state,payload,created_at,updated_at) VALUES(?1,?2,?3,?4,'draft',?5,?6,?6) ON CONFLICT(tenant_id,site_id,id) DO UPDATE SET payload=excluded.payload,updated_at=excluded.updated_at",params![input.receipt_id,tenant,site,input.purchase_order_id,input.payload.to_string(),saved_at]) {
        Ok(_) => Json(input.payload).into_response(),
        Err(_) => fail(StatusCode::CONFLICT,"receipt_conflict","This receipt could not be saved to this site.","retry"),
    }
}
async fn finalize(
    State(state): State<AppState>,
    AxumPath((site, id)): AxumPath<(String, String)>,
    headers: HeaderMap,
) -> Response {
    let user = match identity(&state, &headers).await {
        Ok(v) => v,
        Err(e) => return e,
    };
    let mut db = state.db.lock().expect("db lock");
    let (tenant, site) = match allowed_site(&db, &user, &site) {
        Ok(v) => v,
        Err(response) => return response,
    };
    if let Err(response) = require_active_entitlement(&db, &tenant, &site) {
        return response;
    }
    let record: Option<(String, String)> = db
        .query_row(
            "SELECT payload,state FROM receipts_scoped WHERE id=?1 AND tenant_id=?2 AND site_id=?3",
            params![id, tenant, site],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()
        .unwrap_or(None);
    let Some((payload, receipt_state)) = record else {
        return fail(
            StatusCode::NOT_FOUND,
            "receipt_not_found",
            "This receipt is not in your site.",
            "open_inbox",
        );
    };
    if receipt_state == "finalized" {
        if write_backup(&db, &state.data_dir).is_err() {
            return fail(
                StatusCode::INTERNAL_SERVER_ERROR,
                "backup_error",
                "The finalized receipt is safe, but its backup still could not be completed.",
                "retry",
            );
        }
        let event: Option<(String,i64,String)> = db.query_row("SELECT event_hash,sequence,created_at FROM receipt_events_scoped WHERE tenant_id=?1 AND site_id=?2 AND receipt_id=?3 AND event_type='finalized' ORDER BY sequence LIMIT 1",params![tenant,site,id],|row|Ok((row.get(0)?,row.get(1)?,row.get(2)?))).optional().unwrap_or(None);
        return match event {
            Some((event_hash,sequence,created_at)) => Json(json!({"receipt_id":id,"state":"finalized","event_hash":event_hash,"sequence":sequence,"created_at":created_at,"idempotent":true})).into_response(),
            None => fail(StatusCode::INTERNAL_SERVER_ERROR,"audit_missing","The receipt is finalized, but its audit event is missing.","contact_support"),
        };
    }
    let last: Option<i64> = db
        .query_row(
            "SELECT COALESCE(MAX(sequence),0) FROM receipt_events_scoped WHERE tenant_id=?1 AND site_id=?2 AND receipt_id=?3",
            params![tenant,site,id],
            |r| r.get(0),
        )
        .optional()
        .unwrap_or(None);
    let sequence = last.unwrap_or(0) + 1;
    let previous = if sequence == 1 {
        "GENESIS".to_owned()
    } else {
        db.query_row(
            "SELECT event_hash FROM receipt_events_scoped WHERE tenant_id=?1 AND site_id=?2 AND receipt_id=?3 AND sequence=?4",
            params![tenant,site,id, sequence - 1],
            |r| r.get(0),
        )
        .unwrap_or_else(|_| "GENESIS".into())
    };
    let event_time = now();
    let receipt_value = serde_json::from_str::<Value>(&payload).unwrap_or(Value::Null);
    let actor = receipt_value["receivedBy"]
        .as_str()
        .unwrap_or("Receiving team member");
    let summary = receipt_value["events"]
        .as_array()
        .and_then(|events| events.last())
        .and_then(|event| event["summary"].as_str())
        .unwrap_or("Receipt finalized.");
    let event_id = format!("{id}:event-{sequence}");
    let event_payload = json!({"actor":actor,"summary":summary});
    let event_hash = audit_hash(
        &event_id,
        sequence,
        "finalized",
        &event_time,
        actor,
        summary,
        &previous,
    );
    let tx = match db.transaction() {
        Ok(v) => v,
        Err(_) => {
            return fail(
                StatusCode::INTERNAL_SERVER_ERROR,
                "storage_error",
                "Could not finalize the receipt.",
                "retry",
            )
        }
    };
    let updated = match tx.execute("UPDATE receipts_scoped SET state='finalized',finalized_at=?1,updated_at=?1 WHERE tenant_id=?2 AND site_id=?3 AND id=?4 AND state!='finalized'",params![event_time,tenant,site,id]) {
        Ok(value) => value,
        Err(_) => return fail(StatusCode::INTERNAL_SERVER_ERROR,"storage_error","Could not finalize the receipt.","retry"),
    };
    if updated == 0 {
        return fail(
            StatusCode::CONFLICT,
            "receipt_finalized",
            "This receipt was already finalized.",
            "open_receipt",
        );
    };
    if tx.execute(
        "INSERT INTO receipt_events_scoped(id,tenant_id,site_id,receipt_id,sequence,event_type,payload,previous_hash,event_hash,created_at) VALUES(?1,?2,?3,?4,?5,'finalized',?6,?7,?8,?9)",
        params![
            event_id,
            tenant,
            site,
            id,
            sequence,
            event_payload.to_string(),
            previous,
            event_hash,
            event_time
        ],
    ).is_err() {
        return fail(StatusCode::INTERNAL_SERVER_ERROR,"storage_error","Could not write the receipt audit event.","retry");
    }
    if tx.commit().is_err() {
        return fail(
            StatusCode::INTERNAL_SERVER_ERROR,
            "storage_error",
            "Could not commit the finalized receipt.",
            "retry",
        );
    }
    if write_backup(&db, &state.data_dir).is_err() {
        return fail(
            StatusCode::INTERNAL_SERVER_ERROR,
            "backup_error",
            "The receipt was finalized, but its backup could not be completed.",
            "retry",
        );
    }
    Json(json!({"receipt_id":id,"state":"finalized","event_hash":event_hash,"sequence":sequence,"created_at":event_time}))
        .into_response()
}
async fn add_attachment(
    State(state): State<AppState>,
    AxumPath((site, receipt_id)): AxumPath<(String, String)>,
    headers: HeaderMap,
    Json(attachment): Json<Attachment>,
) -> Response {
    let user = match identity(&state, &headers).await {
        Ok(v) => v,
        Err(e) => return e,
    };
    if !["image/jpeg", "image/png", "image/webp", "application/pdf"]
        .contains(&attachment.media_type.as_str())
    {
        return fail(
            StatusCode::BAD_REQUEST,
            "invalid_attachment",
            "Only JPEG, PNG, WebP, and PDF evidence is accepted.",
            "choose_file",
        );
    };
    let bytes = match STANDARD.decode(attachment.data_base64) {
        Ok(v) if v.len() <= 5 * 1024 * 1024 => v,
        _ => {
            return fail(
                StatusCode::BAD_REQUEST,
                "invalid_attachment",
                "The evidence file is invalid or exceeds 5 MB.",
                "choose_file",
            )
        }
    };
    let signature_matches = match attachment.media_type.as_str() {
        "image/png" => bytes.starts_with(&[0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]),
        "image/jpeg" => bytes.starts_with(&[0xff, 0xd8, 0xff]),
        "image/webp" => bytes.starts_with(b"RIFF") && bytes.get(8..12) == Some(b"WEBP"),
        "application/pdf" => bytes.starts_with(b"%PDF-"),
        _ => false,
    };
    if !signature_matches {
        return fail(
            StatusCode::BAD_REQUEST,
            "invalid_attachment",
            "The evidence file content does not match its file type.",
            "choose_file",
        );
    }
    let db = state.db.lock().expect("db lock");
    let (tenant, site) = match allowed_site(&db, &user, &site) {
        Ok(v) => v,
        Err(response) => return response,
    };
    if let Err(response) = require_active_entitlement(&db, &tenant, &site) {
        return response;
    }
    let allowed: Option<String> = db
        .query_row(
            "SELECT id FROM receipts_scoped WHERE id=?1 AND tenant_id=?2 AND site_id=?3",
            params![receipt_id, tenant, site],
            |r| r.get(0),
        )
        .optional()
        .unwrap_or(None);
    if allowed.is_none() {
        return fail(
            StatusCode::FORBIDDEN,
            "receipt_forbidden",
            "That receipt is not in your site.",
            "open_inbox",
        );
    };
    let checksum = digest_bytes(&bytes);
    let key = format!("objects/{checksum}");
    if fs::write(state.data_dir.join(&key), &bytes).is_err() {
        return fail(
            StatusCode::INTERNAL_SERVER_ERROR,
            "storage_error",
            "Could not retain the evidence file.",
            "retry",
        );
    };
    let attachment_id = attachment
        .id
        .filter(|value| !value.trim().is_empty() && value.len() <= 128)
        .unwrap_or_else(|| Uuid::new_v4().to_string());
    let retention_until =
        (Utc::now() + chrono::Duration::days(365 * 7)).to_rfc3339_opts(SecondsFormat::Secs, true);
    let created_at = now();
    if db.execute(
        "INSERT INTO attachments_scoped(id,tenant_id,site_id,receipt_id,name,object_key,checksum,media_type,bytes,retention_until,created_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11) ON CONFLICT(tenant_id,site_id,id) DO NOTHING",
        params![
            attachment_id,
            tenant,
            site,
            receipt_id,
            attachment.name,
            key,
            checksum,
            attachment.media_type,
            bytes.len() as i64,
            retention_until,
            created_at
        ],
    ).is_err() {
        return fail(StatusCode::INTERNAL_SERVER_ERROR,"storage_error","The evidence file was retained, but its receipt link could not be saved.","retry");
    }
    Json(json!({"id":attachment_id,"name":attachment.name,"checksum":checksum,"retention_until":retention_until,"created_at":created_at})).into_response()
}
async fn attach_license(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<License>,
) -> Response {
    let user = match identity(&state, &headers).await {
        Ok(v) => v,
        Err(e) => return e,
    };
    if input.license.trim().is_empty() {
        return fail(
            StatusCode::BAD_REQUEST,
            "invalid_license",
            "Paste a license token first.",
            "paste_license",
        );
    };
    let _ = (state, user, input);
    fail(
        StatusCode::SERVICE_UNAVAILABLE,
        "checkout_unavailable",
        "Dock checkout and license attachment are unavailable until the Sociobot product mapping is complete.",
        "use_local_workspace",
    )
}
async fn add_member(
    State(state): State<AppState>,
    AxumPath(site): AxumPath<String>,
    headers: HeaderMap,
    Json(member): Json<Member>,
) -> Response {
    let user = match identity(&state, &headers).await {
        Ok(v) => v,
        Err(e) => return e,
    };
    if member.oid.trim().is_empty()
        || !["owner", "manager", "receiver", "viewer"].contains(&member.role.as_str())
    {
        return fail(
            StatusCode::BAD_REQUEST,
            "invalid_member",
            "Provide a member object ID and supported role.",
            "fix_fields",
        );
    };
    let db = state.db.lock().expect("db lock");
    let (tenant, _) = match allowed_site(&db, &user, &site) {
        Ok(v) => v,
        Err(e) => return e,
    };
    if let Err(response) = require_active_entitlement(&db, &tenant, &site) {
        return response;
    }
    let role: Option<String> = db
        .query_row(
            "SELECT role FROM memberships WHERE tenant_id=?1 AND oid=?2",
            params![tenant, user.oid],
            |r| r.get(0),
        )
        .optional()
        .unwrap_or(None);
    if !matches!(role.as_deref(), Some("owner") | Some("manager")) {
        return fail(
            StatusCode::FORBIDDEN,
            "member_forbidden",
            "Only an owner or manager can add staff.",
            "ask_owner",
        );
    };
    if db.execute("INSERT INTO memberships(tenant_id,oid,role) VALUES(?1,?2,?3) ON CONFLICT(tenant_id,oid) DO UPDATE SET role=excluded.role",params![tenant,member.oid,member.role]).is_err() {
        return fail(
            StatusCode::INTERNAL_SERVER_ERROR,
            "storage_error",
            "The staff member could not be saved.",
            "retry",
        );
    }
    Json(json!({"status":"added"})).into_response()
}

async fn get_receipt(
    State(state): State<AppState>,
    AxumPath((site, id)): AxumPath<(String, String)>,
    headers: HeaderMap,
) -> Response {
    let user = match identity(&state, &headers).await {
        Ok(value) => value,
        Err(response) => return response,
    };
    let db = state.db.lock().expect("db lock");
    let (tenant, site) = match allowed_site(&db, &user, &site) {
        Ok(value) => value,
        Err(response) => return response,
    };
    let record: Option<(String, String, Option<String>)> = db
        .query_row(
            "SELECT payload,state,finalized_at FROM receipts_scoped WHERE tenant_id=?1 AND site_id=?2 AND id=?3",
            params![tenant, site, id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()
        .unwrap_or(None);
    let Some((payload, receipt_state, finalized_at)) = record else {
        return fail(
            StatusCode::NOT_FOUND,
            "receipt_not_found",
            "This receipt is not in your site.",
            "open_inbox",
        );
    };
    let mut events = Vec::new();
    if let Ok(mut statement) = db.prepare("SELECT id,sequence,event_type,payload,previous_hash,event_hash,created_at FROM receipt_events_scoped WHERE tenant_id=?1 AND site_id=?2 AND receipt_id=?3 ORDER BY sequence") {
        if let Ok(rows) = statement.query_map(params![tenant,site,id], |row| {
            Ok(json!({"id":row.get::<_,String>(0)?,"sequence":row.get::<_,i64>(1)?,"type":row.get::<_,String>(2)?,"payload":serde_json::from_str::<Value>(&row.get::<_,String>(3)?).unwrap_or(Value::Null),"previous_hash":row.get::<_,String>(4)?,"hash":row.get::<_,String>(5)?,"created_at":row.get::<_,String>(6)?}))
        }) {
            events.extend(rows.filter_map(Result::ok));
        }
    }
    let mut attachments = Vec::new();
    if let Ok(mut statement) = db.prepare("SELECT id,name,checksum,media_type,bytes,retention_until,created_at FROM attachments_scoped WHERE tenant_id=?1 AND site_id=?2 AND receipt_id=?3 ORDER BY created_at") {
        if let Ok(rows) = statement.query_map(params![tenant,site,id], |row| {
            let attachment_id: String = row.get(0)?;
            Ok(json!({"id":attachment_id,"name":row.get::<_,String>(1)?,"checksum":row.get::<_,String>(2)?,"media_type":row.get::<_,String>(3)?,"bytes":row.get::<_,i64>(4)?,"retention_until":row.get::<_,String>(5)?,"created_at":row.get::<_,String>(6)?,"download_url":format!("/api/v1/sites/{site}/receipts/{id}/attachments/{attachment_id}")}))
        }) {
            attachments.extend(rows.filter_map(Result::ok));
        }
    }
    Json(json!({
        "receipt_id": id,
        "state": receipt_state,
        "finalized_at": finalized_at,
        "payload": serde_json::from_str::<Value>(&payload).unwrap_or(Value::Null),
        "events": events,
        "attachments": attachments,
    }))
    .into_response()
}

async fn add_correction(
    State(state): State<AppState>,
    AxumPath((site, id)): AxumPath<(String, String)>,
    headers: HeaderMap,
    Json(input): Json<Correction>,
) -> Response {
    let reason = input.reason.trim();
    if reason.is_empty() || reason.len() > 1000 {
        return fail(
            StatusCode::BAD_REQUEST,
            "invalid_correction",
            "Add a correction reason of 1 to 1,000 characters.",
            "fix_fields",
        );
    }
    let user = match identity(&state, &headers).await {
        Ok(value) => value,
        Err(response) => return response,
    };
    let db = state.db.lock().expect("db lock");
    let (tenant, site) = match allowed_site(&db, &user, &site) {
        Ok(value) => value,
        Err(response) => return response,
    };
    if let Err(response) = require_active_entitlement(&db, &tenant, &site) {
        return response;
    }
    let receipt_record: Option<(String, String)> = db
        .query_row(
            "SELECT state,payload FROM receipts_scoped WHERE tenant_id=?1 AND site_id=?2 AND id=?3",
            params![tenant, site, id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .unwrap_or(None);
    if receipt_record.as_ref().map(|record| record.0.as_str()) != Some("finalized") {
        return fail(
            StatusCode::CONFLICT,
            "receipt_not_finalized",
            "Finalize this receipt before recording a correction.",
            "finalize_receipt",
        );
    }
    let (sequence, previous): (i64, String) = db.query_row("SELECT COALESCE(MAX(sequence),0)+1,COALESCE((SELECT event_hash FROM receipt_events_scoped WHERE tenant_id=?1 AND site_id=?2 AND receipt_id=?3 ORDER BY sequence DESC LIMIT 1),'GENESIS') FROM receipt_events_scoped WHERE tenant_id=?1 AND site_id=?2 AND receipt_id=?3",params![tenant,site,id],|row|Ok((row.get(0)?,row.get(1)?))).unwrap_or((1,"GENESIS".into()));
    let created_at = now();
    let receipt_payload = receipt_record
        .and_then(|record| serde_json::from_str::<Value>(&record.1).ok())
        .unwrap_or(Value::Null);
    let actor = receipt_payload["receivedBy"].as_str().unwrap_or(&user.name);
    let summary = format!("Correction recorded: {reason}");
    let event_id = format!("{id}:event-{sequence}");
    let payload = json!({"reason":reason,"actor":actor,"summary":summary});
    let event_hash = audit_hash(
        &event_id,
        sequence,
        "corrected",
        &created_at,
        actor,
        &summary,
        &previous,
    );
    match db.execute("INSERT INTO receipt_events_scoped(id,tenant_id,site_id,receipt_id,sequence,event_type,payload,previous_hash,event_hash,created_at) VALUES(?1,?2,?3,?4,?5,'corrected',?6,?7,?8,?9)",params![event_id,tenant,site,id,sequence,payload.to_string(),previous,event_hash,created_at]) {
        Ok(_) => Json(json!({"receipt_id":id,"sequence":sequence,"type":"corrected","reason":reason,"event_hash":event_hash,"created_at":created_at})).into_response(),
        Err(_) => fail(StatusCode::INTERNAL_SERVER_ERROR,"storage_error","The correction could not be added.","retry"),
    }
}

async fn get_attachment(
    State(state): State<AppState>,
    AxumPath((site, receipt_id, attachment_id)): AxumPath<(String, String, String)>,
    headers: HeaderMap,
) -> Response {
    let user = match identity(&state, &headers).await {
        Ok(value) => value,
        Err(response) => return response,
    };
    let db = state.db.lock().expect("db lock");
    let (tenant, site) = match allowed_site(&db, &user, &site) {
        Ok(value) => value,
        Err(response) => return response,
    };
    let record: Option<(String,String,String)> = db.query_row("SELECT object_key,media_type,name FROM attachments_scoped WHERE tenant_id=?1 AND site_id=?2 AND receipt_id=?3 AND id=?4",params![tenant,site,receipt_id,attachment_id],|row|Ok((row.get(0)?,row.get(1)?,row.get(2)?))).optional().unwrap_or(None);
    let Some((object_key, media_type, name)) = record else {
        return fail(
            StatusCode::NOT_FOUND,
            "attachment_not_found",
            "This evidence file is not in your site.",
            "open_receipt",
        );
    };
    let bytes = match fs::read(state.data_dir.join(object_key)) {
        Ok(value) => value,
        Err(_) => {
            return fail(
                StatusCode::NOT_FOUND,
                "attachment_missing",
                "The retained evidence file could not be found.",
                "contact_support",
            )
        }
    };
    let mut response = Response::new(Body::from(bytes));
    if let Ok(value) = HeaderValue::from_str(&media_type) {
        response
            .headers_mut()
            .insert(axum::http::header::CONTENT_TYPE, value);
    }
    if let Ok(value) = HeaderValue::from_str(&format!(
        "attachment; filename=\"{}\"",
        name.replace(['\r', '\n', '\"'], "_")
    )) {
        response
            .headers_mut()
            .insert(axum::http::header::CONTENT_DISPOSITION, value);
    }
    response
}

fn known(path: &str) -> bool {
    path == "/"
        || path == "/health"
        || path == "/ready"
        || matches!(
            path,
            "/demo" | "/start" | "/app" | "/privacy" | "/terms" | "/auth/callback"
        )
        || path.starts_with("/api/")
        || path.starts_with("/demo/")
        || path.starts_with("/app/")
        || path.starts_with("/assets/")
        || path.starts_with("/fonts/")
        || matches!(
            path,
            "/index.html"
                | "/404.html"
                | "/favicon.svg"
                | "/apple-touch-icon.png"
                | "/apple-touch-icon.svg"
                | "/og-image.svg"
                | "/manifest.webmanifest"
                | "/robots.txt"
                | "/sitemap.xml"
                | "/staticwebapp.config.json"
                | "/sw.js"
        )
}
async fn policy(request: Request<Body>, next: Next) -> Response {
    let path = request.uri().path().to_owned();
    let mut response = next.run(request).await;
    if response.status() == StatusCode::OK && !known(&path) {
        *response.status_mut() = StatusCode::NOT_FOUND
    };
    let cache = if path.starts_with("/api/")
        || path == "/sw.js"
        || path == "/index.html"
        || path == "/404.html"
        || !path.contains('.')
    {
        "no-cache, no-store, must-revalidate"
    } else if path.starts_with("/assets/") || path.starts_with("/fonts/") {
        "public, max-age=31536000, immutable"
    } else {
        "public, max-age=3600"
    };
    response
        .headers_mut()
        .insert(CACHE_CONTROL, HeaderValue::from_static(cache));
    response
}
pub fn app(static_dir: impl AsRef<Path>) -> Router {
    let state = open_state();
    let directory = PathBuf::from(static_dir.as_ref());
    let files =
        ServeDir::new(directory.clone()).fallback(ServeFile::new(directory.join("index.html")));
    let limit = GovernorConfigBuilder::default()
        .per_millisecond(50)
        .burst_size(40)
        .key_extractor(SmartIpKeyExtractor)
        .use_headers()
        .finish()
        .expect("rate config");
    let api_limit = GovernorConfigBuilder::default()
        .per_millisecond(50)
        .burst_size(40)
        .key_extractor(SmartIpKeyExtractor)
        .use_headers()
        .finish()
        .expect("api rate config");
    let api = Router::new()
        .route("/me", get(me))
        .route(
            "/sites/{site}/purchase-orders",
            get(list_pos).post(create_po),
        )
        .route("/sites/{site}/members", post(add_member))
        .route("/sites/{site}/purchase-orders/{id}", get(get_po))
        .route("/receipts", post(save_receipt))
        .route(
            "/sites/{site}/receipts/{id}",
            get(get_receipt).patch(save_receipt),
        )
        .route("/sites/{site}/receipts/{id}/finalize", post(finalize))
        .route(
            "/sites/{site}/receipts/{id}/corrections",
            post(add_correction),
        )
        .route(
            "/sites/{site}/receipts/{id}/attachments",
            post(add_attachment),
        )
        .route(
            "/sites/{site}/receipts/{receipt}/attachments/{attachment}",
            get(get_attachment),
        )
        .route("/billing/attach", post(attach_license))
        .with_state(state.clone())
        .layer(GovernorLayer::new(api_limit));
    let static_routes = Router::new()
        .fallback_service(files)
        .layer(GovernorLayer::new(limit))
        .with_state(state.clone());
    Router::new().route("/health",get(health)).route("/ready",get(ready)).nest("/api/v1",api).merge(static_routes).with_state(state).layer(SetResponseHeaderLayer::if_not_present(X_CONTENT_TYPE_OPTIONS,HeaderValue::from_static("nosniff"))).layer(SetResponseHeaderLayer::if_not_present(REFERRER_POLICY,HeaderValue::from_static("strict-origin-when-cross-origin"))).layer(SetResponseHeaderLayer::if_not_present(CONTENT_SECURITY_POLICY,HeaderValue::from_static("default-src 'self'; base-uri 'self'; connect-src 'self' https://sociobotcustomers.ciamlogin.com https://api.sociobot.in; font-src 'self'; frame-ancestors 'none'; img-src 'self' data: blob:; object-src 'none'; script-src 'self'; style-src 'self'"))).layer(SetResponseHeaderLayer::if_not_present(axum::http::HeaderName::from_static("strict-transport-security"),HeaderValue::from_static("max-age=31536000; includeSubDomains"))).layer(SetResponseHeaderLayer::if_not_present(axum::http::HeaderName::from_static("permissions-policy"),HeaderValue::from_static("camera=(self), microphone=(), geolocation=()"))).layer(middleware::from_fn(policy)).layer(TraceLayer::new_for_http())
}
#[cfg(test)]
mod tests {
    use super::*;
    use http_body_util::BodyExt;

    fn test_state(label: &str, identities: &[(&str, &str)]) -> AppState {
        let directory = env::temp_dir().join(format!(
            "intake-desk-{label}-{}-{}",
            std::process::id(),
            Uuid::new_v4()
        ));
        test_state_at(directory, identities)
    }

    fn test_state_at(directory: PathBuf, identities: &[(&str, &str)]) -> AppState {
        fs::create_dir_all(directory.join("objects")).expect("test data directory");
        let db = Connection::open(directory.join("intake-desk.sqlite3")).expect("test database");
        configure_sqlite(&db).expect("test WAL");
        create_schema(&db).expect("test schema");
        let test_identities = identities
            .iter()
            .map(|(token, oid)| {
                (
                    (*token).to_string(),
                    Identity {
                        oid: (*oid).to_string(),
                        name: format!("{oid} receiver"),
                        email: None,
                    },
                )
            })
            .collect();
        AppState {
            db: Arc::new(Mutex::new(db)),
            data_dir: directory,
            auth: AuthVerifier {
                client: reqwest::Client::new(),
                cache: Arc::new(RwLock::new(None)),
            },
            test_identities: Arc::new(test_identities),
        }
    }

    fn auth(token: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {token}")).expect("test token"),
        );
        headers
    }

    fn activate(state: &AppState, user: &Identity) -> (String, String) {
        let db = state.db.lock().expect("db lock");
        let (tenant, site) = provision(&db, user).expect("site provisioned");
        db.execute(
            "INSERT INTO entitlements_scoped(site_id,tenant_id,license_fingerprint,status,checked_at) VALUES(?1,?2,'test-fingerprint','active',?3)",
            params![site,tenant,now()],
        )
        .expect("active test entitlement");
        (tenant, site)
    }

    async fn json_body(response: Response) -> Value {
        let body = response
            .into_body()
            .collect()
            .await
            .expect("response body")
            .to_bytes();
        serde_json::from_slice(&body).expect("JSON response")
    }
    #[test]
    fn event_hash_is_tamper_evident() {
        assert_ne!(digest(&["a", "1"]), digest(&["a", "2"]));
    }

    #[test]
    fn existing_wal_database_reopens_while_an_old_connection_is_writing() {
        let directory = env::temp_dir().join(format!(
            "intake-desk-wal-reopen-{}-{}",
            std::process::id(),
            Uuid::new_v4()
        ));
        fs::create_dir_all(&directory).expect("test directory");
        let path = directory.join("intake-desk.sqlite3");
        let first = Connection::open(&path).expect("first database connection");
        configure_sqlite(&first).expect("initial WAL configuration");
        first
            .execute_batch("CREATE TABLE held(value TEXT); BEGIN IMMEDIATE; INSERT INTO held VALUES('old revision');")
            .expect("held write transaction");

        let replacement = Connection::open(&path).expect("replacement database connection");
        configure_sqlite(&replacement).expect("reuse existing WAL mode without a write lock");

        first.execute_batch("ROLLBACK").expect("release test lock");
        fs::remove_dir_all(directory).expect("remove test database");
    }

    #[test]
    fn tenant_membership_cannot_cross_to_another_site() {
        let db = Connection::open_in_memory().expect("memory database");
        db.execute_batch("CREATE TABLE tenants(id TEXT PRIMARY KEY,name TEXT,created_at TEXT); CREATE TABLE sites(id TEXT PRIMARY KEY,tenant_id TEXT,name TEXT); CREATE TABLE memberships(tenant_id TEXT,oid TEXT,role TEXT,PRIMARY KEY(tenant_id,oid));").expect("schema");
        let alex = Identity {
            oid: "alex".into(),
            name: "Alex".into(),
            email: None,
        };
        let blair = Identity {
            oid: "blair".into(),
            name: "Blair".into(),
            email: None,
        };
        let (_, alex_site) = provision(&db, &alex).expect("alex site");
        let (_, blair_site) = provision(&db, &blair).expect("blair site");
        assert!(allowed_site(&db, &alex, &alex_site).is_ok());
        assert!(allowed_site(&db, &alex, &blair_site).is_err());
    }

    /// @claim:server-tenant-isolation
    #[tokio::test]
    async fn claim_server_tenant_isolation_same_ids_never_overwrite() {
        let state = test_state(
            "tenant-isolation",
            &[("token-a", "tenant-a"), ("token-b", "tenant-b")],
        );
        let alice = state.test_identities["token-a"].clone();
        let blair = state.test_identities["token-b"].clone();
        let (_, alice_site) = activate(&state, &alice);
        let (_, blair_site) = activate(&state, &blair);

        for (token, site, supplier) in [
            ("token-a", alice_site.as_str(), "Alice Bearings"),
            ("token-b", blair_site.as_str(), "Blair Bolts"),
        ] {
            let response = create_po(
                State(state.clone()),
                AxumPath(site.to_string()),
                auth(token),
                Json(PurchaseOrder {
                    purchase_order_id: "po-1001".into(),
                    po_number: "PO-1001".into(),
                    supplier: supplier.into(),
                    payload: json!({"schemaVersion":1,"purchaseOrderId":"po-1001","supplier":supplier}),
                }),
            )
            .await;
            assert_eq!(response.status(), StatusCode::OK);
            let response = save_receipt(
                State(state.clone()),
                auth(token),
                Json(Receipt {
                    site_id: site.to_string(),
                    purchase_order_id: "po-1001".into(),
                    receipt_id: "receipt-1001".into(),
                    payload: json!({"receiptId":"receipt-1001","supplier":supplier}),
                }),
            )
            .await;
            assert_eq!(response.status(), StatusCode::OK);
        }

        let alice_po = json_body(
            get_po(
                State(state.clone()),
                AxumPath((alice_site.clone(), "po-1001".into())),
                auth("token-a"),
            )
            .await,
        )
        .await;
        let blair_po = json_body(
            get_po(
                State(state.clone()),
                AxumPath((blair_site.clone(), "po-1001".into())),
                auth("token-b"),
            )
            .await,
        )
        .await;
        assert_eq!(alice_po["supplier"], "Alice Bearings");
        assert_eq!(blair_po["supplier"], "Blair Bolts");

        let alice_receipt = json_body(
            get_receipt(
                State(state.clone()),
                AxumPath((alice_site.clone(), "receipt-1001".into())),
                auth("token-a"),
            )
            .await,
        )
        .await;
        let blair_receipt = json_body(
            get_receipt(
                State(state.clone()),
                AxumPath((blair_site.clone(), "receipt-1001".into())),
                auth("token-b"),
            )
            .await,
        )
        .await;
        assert_eq!(alice_receipt["payload"]["supplier"], "Alice Bearings");
        assert_eq!(blair_receipt["payload"]["supplier"], "Blair Bolts");

        let forbidden = get_po(
            State(state.clone()),
            AxumPath((blair_site, "po-1001".into())),
            auth("token-a"),
        )
        .await;
        assert_eq!(forbidden.status(), StatusCode::FORBIDDEN);
        fs::remove_dir_all(&state.data_dir).expect("remove test data");
    }

    /// @claim:entitlement-read-only
    #[tokio::test]
    async fn claim_entitlement_makes_writes_read_only_but_keeps_reads() {
        let state = test_state("entitlement", &[("token", "reader")]);
        let user = state.test_identities["token"].clone();
        let (_, site) = {
            let db = state.db.lock().expect("db lock");
            provision(&db, &user).expect("site")
        };
        let denied = create_po(
            State(state.clone()),
            AxumPath(site.clone()),
            auth("token"),
            Json(PurchaseOrder {
                purchase_order_id: "po-read-only".into(),
                po_number: "PO-READ-ONLY".into(),
                supplier: "No write".into(),
                payload: json!({"purchaseOrderId":"po-read-only"}),
            }),
        )
        .await;
        assert_eq!(denied.status(), StatusCode::PAYMENT_REQUIRED);
        let denied_body = json_body(denied).await;
        assert_eq!(denied_body["code"], "site_read_only");

        let receipt_denied = save_receipt(
            State(state.clone()),
            auth("token"),
            Json(Receipt {
                site_id: site.clone(),
                purchase_order_id: "po-read-only".into(),
                receipt_id: "receipt-read-only".into(),
                payload: json!({}),
            }),
        )
        .await;
        assert_eq!(receipt_denied.status(), StatusCode::PAYMENT_REQUIRED);
        let finalize_denied = finalize(
            State(state.clone()),
            AxumPath((site.clone(), "receipt-read-only".into())),
            auth("token"),
        )
        .await;
        assert_eq!(finalize_denied.status(), StatusCode::PAYMENT_REQUIRED);
        let correction_denied = add_correction(
            State(state.clone()),
            AxumPath((site.clone(), "receipt-read-only".into())),
            auth("token"),
            Json(Correction {
                reason: "Correct the count".into(),
            }),
        )
        .await;
        assert_eq!(correction_denied.status(), StatusCode::PAYMENT_REQUIRED);
        let attachment_denied = add_attachment(
            State(state.clone()),
            AxumPath((site.clone(), "receipt-read-only".into())),
            auth("token"),
            Json(Attachment {
                id: Some("proof".into()),
                name: "proof.png".into(),
                media_type: "image/png".into(),
                data_base64: STANDARD.encode([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]),
            }),
        )
        .await;
        assert_eq!(attachment_denied.status(), StatusCode::PAYMENT_REQUIRED);
        let member_denied = add_member(
            State(state.clone()),
            AxumPath(site.clone()),
            auth("token"),
            Json(Member {
                oid: "another-user".into(),
                role: "receiver".into(),
            }),
        )
        .await;
        assert_eq!(member_denied.status(), StatusCode::PAYMENT_REQUIRED);

        let readable = list_pos(State(state.clone()), AxumPath(site.clone()), auth("token")).await;
        assert_eq!(readable.status(), StatusCode::OK);
        assert_eq!(json_body(readable).await["purchase_orders"], json!([]));
        {
            let db = state.db.lock().expect("db lock");
            let (tenant, site) = provision(&db, &user).expect("site");
            db.execute("INSERT INTO entitlements_scoped(site_id,tenant_id,license_fingerprint,status,checked_at,expires_at) VALUES(?1,?2,'expired-test','active',?3,'2020-01-01T00:00:00Z')",params![site,tenant,now()]).expect("expired entitlement");
            assert_eq!(entitlement_status(&db, &tenant, &site), "expired");
            db.execute("UPDATE entitlements_scoped SET status='revoked',expires_at=NULL WHERE tenant_id=?1 AND site_id=?2",params![tenant,site]).expect("revoked entitlement");
            assert_eq!(entitlement_status(&db, &tenant, &site), "revoked");
        }
        fs::remove_dir_all(&state.data_dir).expect("remove test data");
    }

    /// @claim:server-audit-retention
    #[tokio::test]
    async fn claim_server_audit_uses_request_time_and_persists_after_reopen() {
        let state = test_state("audit-time", &[("token", "auditor")]);
        let user = state.test_identities["token"].clone();
        let (_, site) = activate(&state, &user);
        let po = create_po(State(state.clone()),AxumPath(site.clone()),auth("token"),Json(PurchaseOrder{purchase_order_id:"po-audit".into(),po_number:"PO-AUDIT".into(),supplier:"Clock Supply".into(),payload:json!({"schemaVersion":1,"purchaseOrderId":"po-audit","supplier":"Clock Supply"})})).await;
        assert_eq!(po.status(), StatusCode::OK);
        let receipt = save_receipt(
            State(state.clone()),
            auth("token"),
            Json(Receipt {
                site_id: site.clone(),
                purchase_order_id: "po-audit".into(),
                receipt_id: "receipt-audit".into(),
                payload: json!({"receiptId":"receipt-audit","supplier":"Clock Supply"}),
            }),
        )
        .await;
        assert_eq!(receipt.status(), StatusCode::OK);
        let attachment = add_attachment(
            State(state.clone()),
            AxumPath((site.clone(), "receipt-audit".into())),
            auth("token"),
            Json(Attachment {
                id: Some("dock-evidence".into()),
                name: "dock.png".into(),
                media_type: "image/png".into(),
                data_base64: STANDARD.encode([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]),
            }),
        )
        .await;
        assert_eq!(attachment.status(), StatusCode::OK);
        let attachment_json = json_body(attachment).await;
        let finalized = finalize(
            State(state.clone()),
            AxumPath((site.clone(), "receipt-audit".into())),
            auth("token"),
        )
        .await;
        assert_eq!(finalized.status(), StatusCode::OK);
        let finalized_json = json_body(finalized).await;
        let created_at = finalized_json["created_at"]
            .as_str()
            .expect("audit timestamp");
        let parsed = chrono::DateTime::parse_from_rfc3339(created_at).expect("RFC3339 timestamp");
        assert!(
            (Utc::now() - parsed.with_timezone(&Utc))
                .num_seconds()
                .abs()
                < 10
        );
        assert_ne!(created_at, "2026-08-28T00:00:00Z");
        assert!(state.data_dir.join("backup-latest.sqlite3").exists());
        let retried = finalize(
            State(state.clone()),
            AxumPath((site.clone(), "receipt-audit".into())),
            auth("token"),
        )
        .await;
        assert_eq!(retried.status(), StatusCode::OK);
        assert_eq!(json_body(retried).await["idempotent"], true);

        let correction = add_correction(
            State(state.clone()),
            AxumPath((site.clone(), "receipt-audit".into())),
            auth("token"),
            Json(Correction {
                reason: "Supplier confirmed the corrected count.".into(),
            }),
        )
        .await;
        assert_eq!(correction.status(), StatusCode::OK);
        let correction_json = json_body(correction).await;
        assert!(correction_json["created_at"]
            .as_str()
            .is_some_and(|value| value != "2026-08-28T00:00:00Z"));
        let loaded = json_body(
            get_receipt(
                State(state.clone()),
                AxumPath((site.clone(), "receipt-audit".into())),
                auth("token"),
            )
            .await,
        )
        .await;
        assert_eq!(loaded["events"].as_array().map(Vec::len), Some(2));
        assert_eq!(loaded["attachments"].as_array().map(Vec::len), Some(1));
        let attachment_id = attachment_json["id"].as_str().expect("attachment id");
        let download = get_attachment(
            State(state.clone()),
            AxumPath((site.clone(), "receipt-audit".into(), attachment_id.into())),
            auth("token"),
        )
        .await;
        assert_eq!(download.status(), StatusCode::OK);
        let downloaded = download
            .into_body()
            .collect()
            .await
            .expect("attachment body")
            .to_bytes();
        assert_eq!(
            downloaded.as_ref(),
            [0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]
        );

        let reopened =
            Connection::open(state.data_dir.join("intake-desk.sqlite3")).expect("reopen database");
        let persisted: String = reopened
            .query_row(
                "SELECT payload FROM receipts_scoped WHERE id='receipt-audit'",
                [],
                |row| row.get(0),
            )
            .expect("persisted receipt");
        assert!(persisted.contains("Clock Supply"));
        let persisted_time: String = reopened
            .query_row(
                "SELECT created_at FROM receipt_events_scoped WHERE receipt_id='receipt-audit'",
                [],
                |row| row.get(0),
            )
            .expect("persisted event");
        assert_eq!(persisted_time, created_at);
        drop(reopened);
        fs::remove_dir_all(&state.data_dir).expect("remove test data");
    }

    /// @claim:server-record-reload
    #[tokio::test]
    async fn claim_server_records_reload_after_process_reopen() {
        let state = test_state(
            "server-reload",
            &[
                ("owner-token", "reload-owner"),
                ("member-token", "reload-member"),
            ],
        );
        let directory = state.data_dir.clone();
        let user = state.test_identities["owner-token"].clone();
        let (_, site) = activate(&state, &user);
        let saved=create_po(State(state.clone()),AxumPath(site.clone()),auth("owner-token"),Json(PurchaseOrder{purchase_order_id:"po-reload".into(),po_number:"PO-RELOAD".into(),supplier:"Persistent Supply".into(),payload:json!({"schemaVersion":1,"source":"csv","purchaseOrderId":"po-reload","supplier":"Persistent Supply"})})).await;
        assert_eq!(saved.status(), StatusCode::OK);
        let member = add_member(
            State(state.clone()),
            AxumPath(site.clone()),
            auth("owner-token"),
            Json(Member {
                oid: "reload-member".into(),
                role: "receiver".into(),
            }),
        )
        .await;
        assert_eq!(member.status(), StatusCode::OK);
        drop(state);

        let reopened = test_state_at(
            directory.clone(),
            &[
                ("owner-token", "reload-owner"),
                ("member-token", "reload-member"),
            ],
        );
        let profile = json_body(me(State(reopened.clone()), auth("member-token")).await).await;
        assert_eq!(profile["site_id"], site);
        let response = list_pos(
            State(reopened.clone()),
            AxumPath(site),
            auth("member-token"),
        )
        .await;
        assert_eq!(response.status(), StatusCode::OK);
        let records = json_body(response).await;
        assert_eq!(
            records["purchase_orders"][0]["supplier"],
            "Persistent Supply"
        );
        drop(reopened);
        fs::remove_dir_all(directory).expect("remove test data");
    }
}
