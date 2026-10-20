//! Feedback reports: built locally, shown to the user exactly as they will be sent, and posted
//! to the backend named in `app_config::FEEDBACK_ENDPOINT`. Nothing is sent until the user asks,
//! and diagnostics are included only when they opt in. Everything that could identify a server
//! is redacted before the preview is shown.

use std::time::Duration;

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::app_config::{self, FEEDBACK_ENDPOINT, FEEDBACK_TOKEN};
use crate::core;
use crate::error::{AppError, Result};
use crate::logs::LogLine;
use crate::profile::Profile;
use crate::state::AppState;

const MAX_MESSAGE: usize = 5000;
const MAX_CONTACT: usize = 200;
const LOG_LINES: usize = 200;
const SEND_TIMEOUT: Duration = Duration::from_secs(15);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FeedbackKind {
    Bug,
    Idea,
    Question,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedbackInput {
    pub kind: FeedbackKind,
    pub message: String,
    pub contact: Option<String>,
    pub include_diagnostics: bool,
}

#[derive(Clone, Serialize)]
pub struct AppRef {
    pub name: String,
    pub version: String,
}

#[derive(Clone, Serialize)]
pub struct CoreRef {
    pub id: String,
    pub version: Option<String>,
}

#[derive(Clone, Serialize)]
pub struct DiagnosticsInfo {
    pub macos: String,
    pub arch: String,
    pub core: CoreRef,
    pub mode: String,
    pub state: String,
    pub logs: Vec<String>,
}

/// The exact JSON body posted to the feedback endpoint.
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Report {
    pub report_id: String,
    pub kind: FeedbackKind,
    pub message: String,
    pub contact: Option<String>,
    pub app: AppRef,
    pub diagnostics: Option<DiagnosticsInfo>,
    pub sent_at: u64,
}

/// What the report is built from, gathered once so building is a pure function.
pub struct Context {
    pub app: AppRef,
    pub diagnostics: DiagnosticsInfo,
    pub profiles: Vec<Profile>,
    pub home: Option<String>,
}

pub fn build_report(ctx: Context, input: FeedbackInput, report_id: String, sent_at: u64) -> Result<Report> {
    let message = input.message.trim().to_string();
    if message.is_empty() {
        return Err(AppError::Invalid("write a message first".into()));
    }
    if message.chars().count() > MAX_MESSAGE {
        return Err(AppError::Invalid(format!("keep the message under {MAX_MESSAGE} characters")));
    }
    let contact = input.contact.map(|c| c.trim().to_string()).filter(|c| !c.is_empty());
    if contact.as_ref().is_some_and(|c| c.chars().count() > MAX_CONTACT) {
        return Err(AppError::Invalid(format!("keep the contact under {MAX_CONTACT} characters")));
    }

    let diagnostics = input.include_diagnostics.then(|| {
        let mut info = ctx.diagnostics;
        info.logs = info.logs.iter().map(|line| redact(line, &ctx.profiles, ctx.home.as_deref())).collect();
        info
    });

    Ok(Report { report_id, kind: input.kind, message, contact, app: ctx.app, diagnostics, sent_at })
}

// ── Redaction ───────────────────────────────────────────────────────────────

/// Replaces anything that could identify the user's servers or machine.
pub fn redact(text: &str, profiles: &[Profile], home: Option<&str>) -> String {
    let mut out = text.to_string();
    for profile in profiles {
        let mut secrets = vec![crate::profile::Secret::new(profile.server.clone(), "[server]")];
        secrets.extend(crate::profile::def(profile.protocol).secrets(profile));
        for secret in secrets {
            if secret.value.trim().len() >= 4 {
                out = replace_ignore_case(&out, secret.value.trim(), secret.label);
            }
        }
    }
    if let Some(home) = home.filter(|h| h.len() > 1) {
        out = out.replace(home, "~");
    }
    mask_ips(&mask_uuids(&out))
}

fn replace_ignore_case(text: &str, needle: &str, with: &str) -> String {
    // ASCII lowercasing keeps byte offsets identical, so matches map back onto `text`.
    let (hay, pat) = (text.to_ascii_lowercase(), needle.to_ascii_lowercase());
    let mut out = String::with_capacity(text.len());
    let mut last = 0;
    while let Some(found) = hay[last..].find(&pat) {
        let start = last + found;
        out.push_str(&text[last..start]);
        out.push_str(with);
        last = start + pat.len();
    }
    out.push_str(&text[last..]);
    out
}

fn is_uuid_at(b: &[u8], i: usize) -> bool {
    const DASHES: [usize; 4] = [8, 13, 18, 23];
    b.len() >= i + 36
        && (0..36).all(|k| if DASHES.contains(&k) { b[i + k] == b'-' } else { b[i + k].is_ascii_hexdigit() })
}

fn mask_uuids(text: &str) -> String {
    let b = text.as_bytes();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    while i < b.len() {
        if is_uuid_at(b, i) {
            out.push_str("[uuid]");
            i += 36;
        } else {
            // copy one whole character, which may be several bytes
            let len = text[i..].chars().next().map_or(1, char::len_utf8);
            out.push_str(&text[i..i + len]);
            i += len;
        }
    }
    out
}

fn is_ipv4(token: &str) -> bool {
    let parts: Vec<&str> = token.split('.').collect();
    parts.len() == 4 && parts.iter().all(|p| (1..=3).contains(&p.len()) && p.parse::<u8>().is_ok())
}

fn is_ipv6(token: &str) -> bool {
    let colons = token.matches(':').count();
    token.len() >= 4 && (token.contains("::") || colons == 7) && token.chars().all(|c| c.is_ascii_hexdigit() || c == ':')
}

/// Masks IPv4 and IPv6 addresses while leaving versions (`1.12.9`) and clock times (`12:30:05`).
fn mask_ips(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut run = String::new();
    let flush = |run: &mut String, out: &mut String| {
        if run.is_empty() {
            return;
        }
        let trimmed = run.trim_end_matches(['.', ':']);
        let tail = &run[trimmed.len()..];
        if is_ipv4(trimmed) || is_ipv6(trimmed) {
            out.push_str("[ip]");
            out.push_str(tail);
        } else {
            out.push_str(run);
        }
        run.clear();
    };
    for c in text.chars() {
        if c.is_ascii_hexdigit() || c == '.' || c == ':' {
            run.push(c);
        } else {
            flush(&mut run, &mut out);
            out.push(c);
        }
    }
    flush(&mut run, &mut out);
    out
}

// ── Sending ─────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SendResult {
    pub id: Option<String>,
}

#[derive(Deserialize, Default)]
struct Accepted {
    id: Option<serde_json::Value>,
}

pub async fn send_report(endpoint: &str, token: Option<&str>, report: &Report) -> Result<SendResult> {
    let client = reqwest::Client::builder().user_agent(app_config::user_agent()).timeout(SEND_TIMEOUT).build()?;
    let mut request = client.post(endpoint).json(report);
    if let Some(token) = token {
        request = request.bearer_auth(token);
    }
    let response = request.send().await.map_err(|e| {
        if e.is_timeout() {
            AppError::Launch("the feedback server took too long to answer".into())
        } else {
            AppError::Launch("could not reach the feedback server, check your connection".into())
        }
    })?;
    let status = response.status();
    if !status.is_success() {
        return Err(AppError::Launch(format!("the feedback server rejected the report ({status})")));
    }
    let accepted: Accepted = response.json().await.unwrap_or_default();
    let id = accepted.id.map(|v| v.as_str().map(str::to_string).unwrap_or_else(|| v.to_string()));
    Ok(SendResult { id })
}

// ── Commands ────────────────────────────────────────────────────────────────

fn text_of<T: Serialize>(value: &T) -> String {
    serde_json::to_value(value).ok().and_then(|v| v.as_str().map(str::to_string)).unwrap_or_default()
}

fn gather(state: &AppState) -> Context {
    let (spec, settings, profiles, mode) = {
        let store = state.store.lock().unwrap();
        (store.active_core(), store.core_settings(), store.profiles().to_vec(), store.mode())
    };
    let core_version = core::resolve(&state.paths, spec, &settings)
        .ok()
        .and_then(|(path, source)| core::inspect(spec, &path, source).ok())
        .map(|info| info.version);
    let status = state.status.lock().unwrap().clone();
    let logs = state
        .logs
        .snapshot()
        .iter()
        .rev()
        .take(LOG_LINES)
        .rev()
        .map(|LogLine { level, text, .. }| format!("[{}] {}", text_of(level).to_uppercase(), text))
        .collect();

    Context {
        app: AppRef { name: app_config::APP_NAME.into(), version: app_config::APP_VERSION.into() },
        diagnostics: DiagnosticsInfo {
            macos: crate::device::macos_version(),
            arch: std::env::consts::ARCH.into(),
            core: CoreRef { id: spec.id.into(), version: core_version },
            mode: text_of(&mode),
            state: text_of(&status.state),
            logs,
        },
        profiles,
        home: std::env::var("HOME").ok(),
    }
}

fn now_secs() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

fn report_for(state: &AppState, input: FeedbackInput) -> Result<Report> {
    build_report(gather(state), input, uuid::Uuid::new_v4().to_string(), now_secs())
}

/// The exact JSON that would be sent, for the preview in the form.
#[tauri::command]
pub fn feedback_preview(state: State<AppState>, input: FeedbackInput) -> Result<String> {
    Ok(serde_json::to_string_pretty(&report_for(&state, input)?)?)
}

#[tauri::command]
pub async fn feedback_send(state: State<'_, AppState>, input: FeedbackInput) -> Result<SendResult> {
    let endpoint = FEEDBACK_ENDPOINT.ok_or_else(|| AppError::Invalid("feedback is not set up in this build".into()))?;
    let report = report_for(&state, input)?;
    send_report(endpoint, FEEDBACK_TOKEN, &report).await
}

#[cfg(test)]
#[path = "tests/feedback.rs"]
mod tests;
