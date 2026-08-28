//! Durable, tenant-scoped API. The browser cache is only an offline convenience.
use axum::{
    body::Body,
    extract::{Path as AxumPath, State},
    http::{
        header::{
            AUTHORIZATION, CACHE_CONTROL, CONTENT_SECURITY_POLICY, REFERRER_POLICY,
            X_CONTENT_TYPE_OPTIONS,
        },
        HeaderMap, HeaderValue, Request, StatusCode,
    },
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, patch, post},
    Json, Router,
};
use base64::{engine::general_purpose::STANDARD, Engine};
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    env, fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};
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
#[derive(Deserialize)]
struct Jwks {
    keys: Vec<Jwk>,
}
#[derive(Deserialize)]
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
    site: String,
    #[serde(default)]
    payload: Value,
}
#[derive(Deserialize)]
struct Receipt {
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
fn now() -> &'static str {
    "2026-08-28T00:00:00Z"
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
fn fail(
    status: StatusCode,
    code: &'static str,
    message: impl Into<String>,
    action: &'static str,
) -> Response {
    (
        status,
        Json(ApiError {
            code,
            message: message.into(),
            action,
        }),
    )
        .into_response()
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
    db.pragma_update(None, "journal_mode", "WAL")
        .expect("enable WAL");
    db.execute_batch("PRAGMA foreign_keys=ON;
 CREATE TABLE IF NOT EXISTS tenants(id TEXT PRIMARY KEY,name TEXT NOT NULL,created_at TEXT NOT NULL);
 CREATE TABLE IF NOT EXISTS sites(id TEXT PRIMARY KEY,tenant_id TEXT NOT NULL,name TEXT NOT NULL);
 CREATE TABLE IF NOT EXISTS memberships(tenant_id TEXT NOT NULL,oid TEXT NOT NULL,role TEXT NOT NULL,PRIMARY KEY(tenant_id,oid));
 CREATE TABLE IF NOT EXISTS purchase_orders(id TEXT PRIMARY KEY,tenant_id TEXT NOT NULL,site_id TEXT NOT NULL,po_number TEXT NOT NULL,supplier TEXT NOT NULL,payload TEXT NOT NULL,created_at TEXT NOT NULL,UNIQUE(tenant_id,site_id,id));
 CREATE TABLE IF NOT EXISTS receipts(id TEXT PRIMARY KEY,tenant_id TEXT NOT NULL,site_id TEXT NOT NULL,purchase_order_id TEXT NOT NULL,state TEXT NOT NULL,payload TEXT NOT NULL,finalized_at TEXT,created_at TEXT NOT NULL);
 CREATE TABLE IF NOT EXISTS receipt_events(id TEXT PRIMARY KEY,tenant_id TEXT NOT NULL,receipt_id TEXT NOT NULL,sequence INTEGER NOT NULL,event_type TEXT NOT NULL,payload TEXT NOT NULL,previous_hash TEXT NOT NULL,event_hash TEXT NOT NULL,created_at TEXT NOT NULL,UNIQUE(receipt_id,sequence));
 CREATE TABLE IF NOT EXISTS attachments(id TEXT PRIMARY KEY,tenant_id TEXT NOT NULL,receipt_id TEXT NOT NULL,object_key TEXT NOT NULL,checksum TEXT NOT NULL,media_type TEXT NOT NULL,bytes INTEGER NOT NULL,retention_until TEXT NOT NULL,created_at TEXT NOT NULL);
 CREATE TABLE IF NOT EXISTS entitlements(site_id TEXT PRIMARY KEY,tenant_id TEXT NOT NULL,license_fingerprint TEXT NOT NULL,status TEXT NOT NULL,checked_at TEXT NOT NULL,expires_at TEXT);
 CREATE TABLE IF NOT EXISTS idempotency_keys(tenant_id TEXT NOT NULL,oid TEXT NOT NULL,key TEXT NOT NULL,request_hash TEXT NOT NULL,response TEXT NOT NULL,PRIMARY KEY(tenant_id,oid,key));").expect("create schema");
    AppState {
        db: Arc::new(Mutex::new(db)),
        data_dir: directory,
    }
}

#[allow(clippy::result_large_err)]
async fn identity(headers: &HeaderMap) -> Result<Identity, Response> {
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
    let tenant = env_value("ENTRA_TENANT_ID", TENANT_ID);
    let client = env_value("ENTRA_CLIENT_ID", CLIENT_ID);
    let subdomain = env_value("ENTRA_TENANT_SUBDOMAIN", SUBDOMAIN);
    let http = reqwest::Client::new();
    let discovery: Discovery = http
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
    let header = decode_header(token).map_err(|_| {
        fail(
            StatusCode::UNAUTHORIZED,
            "invalid_token",
            "The sign-in token is malformed.",
            "sign_in",
        )
    })?;
    let jwks: Jwks = http
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
    let jwk = jwks
        .keys
        .into_iter()
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
    validation.set_issuer(&[discovery.issuer.as_str()]);
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
    let (tenant, _) = provision(db, user).map_err(|_| {
        fail(
            StatusCode::INTERNAL_SERVER_ERROR,
            "storage_error",
            "Could not open the receiving site.",
            "retry",
        )
    })?;
    let found:Option<String>=db.query_row("SELECT s.id FROM sites s JOIN memberships m ON m.tenant_id=s.tenant_id WHERE s.id=?1 AND m.oid=?2",params![site,user.oid],|r|r.get(0)).optional().unwrap_or(None);
    found.map(|s| (tenant, s)).ok_or_else(|| {
        fail(
            StatusCode::FORBIDDEN,
            "site_forbidden",
            "You do not have access to this receiving site.",
            "choose_site",
        )
    })
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
    let user = match identity(&headers).await {
        Ok(v) => v,
        Err(e) => return e,
    };
    let db = state.db.lock().expect("db lock");
    match provision(&db,&user){Ok((tenant,site))=>Json(json!({"oid":user.oid,"name":user.name,"email":user.email,"tenant_id":tenant,"site_id":site,"entitlement":"unverified"})).into_response(),Err(_)=>fail(StatusCode::INTERNAL_SERVER_ERROR,"storage_error","Could not provision your team.","retry")}
}
async fn list_pos(
    State(state): State<AppState>,
    AxumPath(site): AxumPath<String>,
    headers: HeaderMap,
) -> Response {
    let user = match identity(&headers).await {
        Ok(v) => v,
        Err(e) => return e,
    };
    let db = state.db.lock().expect("db lock");
    let (tenant, site) = match allowed_site(&db, &user, &site) {
        Ok(v) => v,
        Err(e) => return e,
    };
    let mut statement=match db.prepare("SELECT payload FROM purchase_orders WHERE tenant_id=?1 AND site_id=?2 ORDER BY created_at DESC"){Ok(v)=>v,Err(_)=>return fail(StatusCode::INTERNAL_SERVER_ERROR,"storage_error","Could not list purchase orders.","retry")};
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
    let user = match identity(&headers).await {
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
    let payload = json!({"purchase_order_id":po.purchase_order_id,"po_number":po.po_number,"supplier":po.supplier,"site":po.site,"payload":po.payload});
    match db.execute("INSERT INTO purchase_orders(id,tenant_id,site_id,po_number,supplier,payload,created_at) VALUES(?1,?2,?3,?4,?5,?6,?7) ON CONFLICT(id) DO UPDATE SET payload=excluded.payload,po_number=excluded.po_number,supplier=excluded.supplier",params![payload["purchase_order_id"].as_str(),tenant,site,payload["po_number"].as_str(),payload["supplier"].as_str(),payload.to_string(),now()]){Ok(_)=>Json(payload).into_response(),Err(_)=>fail(StatusCode::CONFLICT,"purchase_order_conflict","This purchase order could not be saved to this site.","retry")}
}
async fn get_po(
    State(state): State<AppState>,
    AxumPath(id): AxumPath<String>,
    headers: HeaderMap,
) -> Response {
    let user = match identity(&headers).await {
        Ok(v) => v,
        Err(e) => return e,
    };
    let db = state.db.lock().expect("db lock");
    let (tenant, _) = match provision(&db, &user) {
        Ok(v) => v,
        Err(_) => {
            return fail(
                StatusCode::INTERNAL_SERVER_ERROR,
                "storage_error",
                "Could not open storage.",
                "retry",
            )
        }
    };
    let value:Option<String>=db.query_row("SELECT p.payload FROM purchase_orders p JOIN memberships m ON m.tenant_id=p.tenant_id WHERE p.id=?1 AND p.tenant_id=?2 AND m.oid=?3",params![id,tenant,user.oid],|r|r.get(0)).optional().unwrap_or(None);
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
    let user = match identity(&headers).await {
        Ok(v) => v,
        Err(e) => return e,
    };
    let db = state.db.lock().expect("db lock");
    let (tenant, site) = match provision(&db, &user) {
        Ok(v) => v,
        Err(_) => {
            return fail(
                StatusCode::INTERNAL_SERVER_ERROR,
                "storage_error",
                "Could not open storage.",
                "retry",
            )
        }
    };
    let owns: Option<String> = db
        .query_row(
            "SELECT id FROM purchase_orders WHERE id=?1 AND tenant_id=?2 AND site_id=?3",
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
    let payload = json!({"purchase_order_id":input.purchase_order_id,"receipt_id":input.receipt_id,"payload":input.payload});
    let existing: Option<String> = db
        .query_row(
            "SELECT state FROM receipts WHERE id=?1",
            params![payload["receipt_id"].as_str()],
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
    db.execute("INSERT INTO receipts(id,tenant_id,site_id,purchase_order_id,state,payload,created_at) VALUES(?1,?2,?3,?4,'draft',?5,?6) ON CONFLICT(id) DO UPDATE SET payload=excluded.payload",params![payload["receipt_id"].as_str(),tenant,site,payload["purchase_order_id"].as_str(),payload.to_string(),now()]).ok();
    Json(payload).into_response()
}
async fn finalize(
    State(state): State<AppState>,
    AxumPath(id): AxumPath<String>,
    headers: HeaderMap,
) -> Response {
    let user = match identity(&headers).await {
        Ok(v) => v,
        Err(e) => return e,
    };
    let mut db = state.db.lock().expect("db lock");
    let (tenant, _) = match provision(&db, &user) {
        Ok(v) => v,
        Err(_) => {
            return fail(
                StatusCode::INTERNAL_SERVER_ERROR,
                "storage_error",
                "Could not open storage.",
                "retry",
            )
        }
    };
    let payload:Option<String>=db.query_row("SELECT r.payload FROM receipts r JOIN memberships m ON m.tenant_id=r.tenant_id WHERE r.id=?1 AND r.tenant_id=?2 AND m.oid=?3",params![id,tenant,user.oid],|r|r.get(0)).optional().unwrap_or(None);
    let Some(payload) = payload else {
        return fail(
            StatusCode::NOT_FOUND,
            "receipt_not_found",
            "This receipt is not in your site.",
            "open_inbox",
        );
    };
    let last: Option<i64> = db
        .query_row(
            "SELECT MAX(sequence) FROM receipt_events WHERE receipt_id=?1",
            params![id],
            |r| r.get(0),
        )
        .optional()
        .unwrap_or(None);
    let sequence = last.unwrap_or(0) + 1;
    let previous = if sequence == 1 {
        "genesis".to_owned()
    } else {
        db.query_row(
            "SELECT event_hash FROM receipt_events WHERE receipt_id=?1 AND sequence=?2",
            params![id, sequence - 1],
            |r| r.get(0),
        )
        .unwrap_or_else(|_| "genesis".into())
    };
    let event_hash = digest(&[
        id.as_str(),
        &sequence.to_string(),
        "finalized",
        &payload,
        &previous,
    ]);
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
    if tx.execute("UPDATE receipts SET state='finalized',finalized_at=?1 WHERE id=?2 AND state!='finalized'",params![now(),id]).unwrap_or(0)==0{return fail(StatusCode::CONFLICT,"receipt_finalized","This receipt was already finalized.","open_receipt")};
    tx.execute(
        "INSERT INTO receipt_events VALUES(?1,?2,?3,?4,'finalized',?5,?6,?7,?8)",
        params![
            Uuid::new_v4().to_string(),
            tenant,
            id,
            sequence,
            payload,
            previous,
            event_hash,
            now()
        ],
    )
    .ok();
    tx.commit().ok();
    let _ = fs::copy(
        state.data_dir.join("intake-desk.sqlite3"),
        state.data_dir.join("backup-latest.sqlite3"),
    );
    Json(json!({"receipt_id":id,"state":"finalized","event_hash":event_hash,"sequence":sequence}))
        .into_response()
}
async fn add_attachment(
    State(state): State<AppState>,
    AxumPath(receipt_id): AxumPath<String>,
    headers: HeaderMap,
    Json(attachment): Json<Attachment>,
) -> Response {
    let user = match identity(&headers).await {
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
    let db = state.db.lock().expect("db lock");
    let (tenant, _) = match provision(&db, &user) {
        Ok(v) => v,
        Err(_) => {
            return fail(
                StatusCode::INTERNAL_SERVER_ERROR,
                "storage_error",
                "Could not open storage.",
                "retry",
            )
        }
    };
    let allowed:Option<String>=db.query_row("SELECT r.id FROM receipts r JOIN memberships m ON m.tenant_id=r.tenant_id WHERE r.id=?1 AND r.tenant_id=?2 AND m.oid=?3",params![receipt_id,tenant,user.oid],|r|r.get(0)).optional().unwrap_or(None);
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
    db.execute(
        "INSERT INTO attachments VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)",
        params![
            Uuid::new_v4().to_string(),
            tenant,
            receipt_id,
            key,
            checksum,
            attachment.media_type,
            bytes.len() as i64,
            "2033-01-01T00:00:00Z",
            now()
        ],
    )
    .ok();
    Json(json!({"name":attachment.name,"checksum":checksum,"retention_until":"2033-01-01T00:00:00Z"})).into_response()
}
async fn attach_license(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<License>,
) -> Response {
    let user = match identity(&headers).await {
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
    let status = match reqwest::Client::new()
        .get("https://api.sociobot.in/api/v1/products/purchase-intake-desk/verify")
        .query(&[("license", input.license.trim())])
        .send()
        .await
    {
        Ok(response) => match response.json::<Value>().await {
            Ok(value) if value["valid"].as_bool() == Some(true) => "active",
            Ok(_) => "invalid",
            Err(_) => "pending",
        },
        Err(_) => "pending",
    };
    let db = state.db.lock().expect("db lock");
    let (tenant, site) = match provision(&db, &user) {
        Ok(v) => v,
        Err(_) => {
            return fail(
                StatusCode::INTERNAL_SERVER_ERROR,
                "storage_error",
                "Could not open storage.",
                "retry",
            )
        }
    };
    db.execute("INSERT INTO entitlements(site_id,tenant_id,license_fingerprint,status,checked_at) VALUES(?1,?2,?3,?4,?5) ON CONFLICT(site_id) DO UPDATE SET license_fingerprint=excluded.license_fingerprint,status=excluded.status,checked_at=excluded.checked_at",params![site,tenant,digest(&[input.license.trim()]),status,now()]).ok();
    Json(json!({"status":status,"message":if status=="active"{"License is active for this site."}else if status=="invalid"{"This license is not active for this product."}else{"License saved; verification will retry."}})).into_response()
}
async fn add_member(
    State(state): State<AppState>,
    AxumPath(site): AxumPath<String>,
    headers: HeaderMap,
    Json(member): Json<Member>,
) -> Response {
    let user = match identity(&headers).await {
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
    db.execute("INSERT INTO memberships(tenant_id,oid,role) VALUES(?1,?2,?3) ON CONFLICT(tenant_id,oid) DO UPDATE SET role=excluded.role",params![tenant,member.oid,member.role]).ok();
    Json(json!({"status":"added"})).into_response()
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
    let cache = if path == "/sw.js"
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
        .route("/purchase-orders/{id}", get(get_po))
        .route("/receipts", post(save_receipt))
        .route("/receipts/{id}", patch(save_receipt))
        .route("/receipts/{id}/finalize", post(finalize))
        .route("/receipts/{id}/attachments", post(add_attachment))
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
    #[test]
    fn event_hash_is_tamper_evident() {
        assert_ne!(digest(&["a", "1"]), digest(&["a", "2"]));
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
}
