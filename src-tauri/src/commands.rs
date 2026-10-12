use serde::Serialize;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_autostart::ManagerExt;

use crate::app_config;
use crate::config::Mode;
use crate::diagnostics::{self, Diagnostics, SpeedResult};
use crate::error::{AppError, Result};
use crate::logs::LogLine;
use crate::profile::{parse_many, to_link, Profile, ProfileView};
use crate::session::{self, State as ConnState, Status};
use crate::settings::{AutoSettings, ReliabilitySettings, RoutingSettings, Subscription};
use crate::state::AppState;
use crate::store::Settings;
use crate::subscriptions::{self, RefreshSummary};
use crate::traffic::{Sample, Traffic};
use crate::{clash, net, notify, reliability, tray};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    profiles: Vec<ProfileView>,
    active_id: Option<String>,
    mode: Mode,
    status: Status,
    logs: Vec<LogLine>,
    traffic: Traffic,
    history: Vec<Sample>,
    settings: Settings,
    launch_at_login: bool,
    auto: AutoSettings,
    subscriptions: Vec<Subscription>,
    routing: RoutingSettings,
    reliability: ReliabilitySettings,
    hotkey: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportSummary {
    added: usize,
    duplicates: usize,
    failed: Vec<String>,
}

#[derive(Serialize)]
pub struct NodeLatency {
    id: String,
    ms: Option<u64>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    name: &'static str,
    brand: &'static str,
    version: &'static str,
    updates_configured: bool,
    feedback_configured: bool,
    protocols: Vec<crate::profile::ProtocolView>,
}

/// Identity and feature switches from `app_config.rs`, so the UI never hardcodes them.
#[tauri::command]
pub fn app_info() -> AppInfo {
    AppInfo {
        name: app_config::APP_NAME,
        brand: app_config::BRAND_NAME,
        version: app_config::APP_VERSION,
        updates_configured: app_config::APP_UPDATE_FEED.is_some() && app_config::APP_UPDATE_PUBKEY.is_some(),
        feedback_configured: app_config::FEEDBACK_ENDPOINT.is_some(),
        protocols: crate::profile::views(),
    }
}

#[tauri::command]
pub fn snapshot(app: AppHandle, state: State<AppState>) -> Snapshot {
    let store = state.store.lock().unwrap();
    let (traffic, history) = state.traffic.lock().unwrap().snapshot();
    Snapshot {
        profiles: store.profiles().iter().map(Profile::view).collect(),
        active_id: store.active_id().map(str::to_string),
        mode: store.mode(),
        status: state.status.lock().unwrap().clone(),
        logs: state.logs.snapshot(),
        traffic,
        history,
        settings: store.settings(),
        launch_at_login: app.autolaunch().is_enabled().unwrap_or(false),
        auto: store.auto(),
        subscriptions: store.subscriptions().to_vec(),
        routing: store.routing(),
        reliability: store.reliability(),
        hotkey: reliability::hotkey_label(),
    }
}

#[tauri::command]
pub fn import_text(state: State<AppState>, text: String) -> Result<ImportSummary> {
    let (profiles, failed) = parse_many(&text);
    if profiles.is_empty() {
        let reason = failed.into_iter().next().unwrap_or_else(|| "no supported server links were found".into());
        return Err(AppError::Invalid(reason));
    }
    let (added, duplicates) = state.store.lock().unwrap().import_unique(profiles)?;
    Ok(ImportSummary { added, duplicates, failed })
}

/// A blank server of one protocol, with that protocol's defaults.
#[tauri::command]
pub fn new_profile(protocol: String) -> Result<Profile> {
    crate::profile::blank(&protocol)
}

#[tauri::command]
pub fn save_profile(state: State<AppState>, mut profile: Profile) -> Result<Profile> {
    profile.normalize();
    profile.validate()?;
    if profile.id.is_empty() {
        profile.id = uuid::Uuid::new_v4().to_string();
    }
    state.store.lock().unwrap().upsert(profile.clone())?;
    Ok(profile)
}

#[tauri::command]
pub fn duplicate_profile(state: State<AppState>, id: String) -> Result<Profile> {
    let mut store = state.store.lock().unwrap();
    let mut copy = store.get(&id).cloned().ok_or(AppError::ProfileNotFound)?;
    copy.id = uuid::Uuid::new_v4().to_string();
    copy.name = format!("{} copy", copy.name);
    copy.subscription_id = None;
    store.add(copy.clone())?;
    Ok(copy)
}

/// Removes several servers with one save, such as every server a test found unreachable.
#[tauri::command]
pub fn delete_profiles(state: State<AppState>, ids: Vec<String>) -> Result<usize> {
    state.store.lock().unwrap().remove_many(&ids)
}

#[tauri::command]
pub fn delete_profile(state: State<AppState>, id: String) -> Result<()> {
    state.store.lock().unwrap().remove(&id)
}

#[tauri::command]
pub fn set_favorite(state: State<AppState>, id: String, favorite: bool) -> Result<()> {
    state.store.lock().unwrap().edit(&id, |p| p.favorite = favorite)
}

#[tauri::command]
pub fn set_group(state: State<AppState>, id: String, group: Option<String>) -> Result<()> {
    let group = group.map(|g| g.trim().to_string()).filter(|g| !g.is_empty());
    state.store.lock().unwrap().edit(&id, |p| p.group = group)
}

#[tauri::command]
pub fn reorder_profiles(state: State<AppState>, ids: Vec<String>) -> Result<()> {
    state.store.lock().unwrap().reorder(&ids)
}

#[tauri::command]
pub fn profile_link(state: State<AppState>, id: String) -> Result<String> {
    let store = state.store.lock().unwrap();
    store.get(&id).map(to_link).ok_or(AppError::ProfileNotFound)
}

#[tauri::command]
pub fn profile_qr(state: State<AppState>, id: String) -> Result<String> {
    let link = profile_link(state, id)?;
    let code = qrcode::QrCode::new(link.as_bytes()).map_err(|e| AppError::Invalid(e.to_string()))?;
    Ok(code.render::<qrcode::render::svg::Color>().min_dimensions(220, 220).quiet_zone(true).build())
}

#[tauri::command]
pub async fn select_profile(app: AppHandle, id: String) -> Result<()> {
    let state = app.state::<AppState>();
    let auto = {
        let mut store = state.store.lock().unwrap();
        store.select(&id)?;
        store.auto().enabled
    };
    let connected = state.status.lock().unwrap().state == ConnState::Connected;
    if connected && !auto {
        clash::select_profile(&app, &id).await?;
    }
    Ok(())
}

#[tauri::command]
pub fn set_mode(state: State<AppState>, mode: Mode) -> Result<()> {
    state.store.lock().unwrap().set_mode(mode)
}

#[tauri::command]
pub fn set_auto(state: State<AppState>, settings: AutoSettings) -> Result<()> {
    state.store.lock().unwrap().set_auto(settings)
}

#[tauri::command]
pub fn trust_key(state: State<AppState>, id: String, key: String) -> Result<()> {
    state.store.lock().unwrap().trust_key(&id, key)
}

#[tauri::command]
pub fn crash_log(state: State<AppState>) -> String {
    crate::crashlog::read(&state.paths.data_dir())
}

#[tauri::command]
pub fn clear_crash_log(state: State<AppState>) {
    crate::crashlog::clear(&state.paths.data_dir());
}

#[tauri::command]
pub fn clear_logs(state: State<AppState>) {
    state.logs.clear();
}

#[tauri::command]
pub async fn connect(app: AppHandle) -> Result<()> {
    reliability::user_took_over(&app);
    match session::connect(&app).await {
        // The user pressed disconnect while it was starting; that is what they asked for.
        Err(error) if error.is_cancelled() => Ok(()),
        other => other,
    }
}

#[tauri::command]
pub async fn disconnect(app: AppHandle) -> Result<()> {
    reliability::user_took_over(&app);
    session::disconnect(&app).await
}

#[tauri::command]
pub async fn ping_profile(state: State<'_, AppState>, id: String) -> Result<u64> {
    let (server, port) = {
        let store = state.store.lock().unwrap();
        let profile = store.get(&id).ok_or(AppError::ProfileNotFound)?;
        (profile.server.clone(), profile.port)
    };
    net::tcp_latency(&server, port).await
}

#[tauri::command]
pub async fn test_all(app: AppHandle) -> Result<Vec<NodeLatency>> {
    const BATCH: usize = 12;
    let state = app.state::<AppState>();
    let targets: Vec<(String, String, u16)> = state
        .store
        .lock()
        .unwrap()
        .profiles()
        .iter()
        .map(|p| (p.id.clone(), p.server.clone(), p.port))
        .collect();
    let connected = state.status.lock().unwrap().state == ConnState::Connected;
    // While connected, servers in the running config are measured through the tunnel. The others
    // (added since connecting, or past the config's limit) are measured directly; asking the engine
    // about a server it does not have would report them all as dead.
    let in_tunnel = if connected { clash::known_tags(&app).await.unwrap_or_default() } else { Default::default() };

    let mut results = Vec::new();
    for batch in targets.chunks(BATCH) {
        let handles: Vec<_> = batch
            .iter()
            .cloned()
            .map(|(id, server, port)| {
                let app = app.clone();
                let through_tunnel = in_tunnel.contains(&crate::config::node_tag(&id));
                tauri::async_runtime::spawn(async move {
                    let ms = if through_tunnel {
                        clash::node_delay(&app, &id).await.ok()
                    } else {
                        net::tcp_latency(&server, port).await.ok()
                    };
                    NodeLatency { id, ms }
                })
            })
            .collect();
        for handle in handles {
            if let Ok(result) = handle.await {
                results.push(result);
            }
        }
    }

    let now = crate::usage::collector::now_secs();
    if let Some(db) = state.usage.db.lock().unwrap().as_ref() {
        for result in &results {
            let _ = db.record_latency(&result.id, now, result.ms);
        }
    }
    Ok(results)
}

#[tauri::command]
pub async fn test_connection() -> Result<u64> {
    net::proxy_delay().await
}

#[tauri::command]
pub async fn exit_ip() -> Result<String> {
    net::exit_ip().await
}

#[tauri::command]
pub fn rename_profile(state: State<AppState>, id: String, name: String) -> Result<()> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err(AppError::Invalid("name cannot be empty".into()));
    }
    state.store.lock().unwrap().edit(&id, |p| p.name = name)
}

#[tauri::command]
pub fn update_settings(app: AppHandle, state: State<AppState>, settings: Settings) -> Result<()> {
    state.store.lock().unwrap().set_settings(settings)?;
    if !settings.show_speed {
        tray::set_speed(&app, None);
    }
    Ok(())
}

#[tauri::command]
pub fn set_launch_at_login(app: AppHandle, enabled: bool) -> Result<()> {
    let autolaunch = app.autolaunch();
    let result = if enabled { autolaunch.enable() } else { autolaunch.disable() };
    result.map_err(|e| AppError::Launch(e.to_string()))
}

#[tauri::command]
pub fn set_routing(app: AppHandle, state: State<AppState>, settings: RoutingSettings) -> Result<()> {
    crate::config::validate_dns(&settings.dns)?;
    let dir = state.paths.rule_sets_dir();
    let (missing, _) = crate::rulesets::plan(&dir, &crate::rulesets::wanted(&settings), crate::rulesets::MAX_AGE);
    state.store.lock().unwrap().set_routing(settings)?;
    // Fetch new lists now, so connecting later does not wait for them and a failure shows up here.
    if !missing.is_empty() {
        tauri::async_runtime::spawn(async move {
            let failed = crate::rulesets::fetch_all(&dir, missing).await;
            if !failed.is_empty() {
                let tags: Vec<String> = failed.into_iter().map(|(item, _)| item.tag).collect();
                notify::send(&app, "Some lists could not be downloaded", &crate::rulesets::skipped_message(&tags));
            }
        });
    }
    Ok(())
}

#[tauri::command]
pub fn set_reliability(app: AppHandle, state: State<AppState>, settings: ReliabilitySettings) -> Result<()> {
    state.store.lock().unwrap().set_reliability(settings)?;
    reliability::apply_hotkey(&app);
    Ok(())
}

#[tauri::command]
pub fn root_service_installed() -> bool {
    crate::runner::daemon::installed()
}

#[tauri::command]
pub async fn install_root_service(state: State<'_, AppState>) -> Result<()> {
    let (spec, settings) = {
        let store = state.store.lock().unwrap();
        (store.active_core(), store.core_settings())
    };
    let (binary, _) = crate::core::resolve(&state.paths, spec, &settings)?;
    crate::runner::daemon::install(&state.paths, &binary).await
}

#[tauri::command]
pub async fn uninstall_root_service() -> Result<()> {
    crate::runner::daemon::uninstall().await
}

#[tauri::command]
pub async fn restore_network() -> Result<()> {
    reliability::restore_network().await
}

#[tauri::command]
pub async fn add_subscription(
    app: AppHandle,
    name: String,
    url: String,
    user_agent: Option<String>,
    send_device_id: Option<bool>,
) -> Result<RefreshSummary> {
    subscriptions::validate_url(&url)?;
    let name = name.trim();
    let subscription = Subscription {
        id: uuid::Uuid::new_v4().to_string(),
        name: if name.is_empty() { "Subscription".into() } else { name.to_string() },
        url: url.trim().to_string(),
        user_agent: subscriptions::clean_user_agent(user_agent.as_deref())?,
        send_device_id: send_device_id.unwrap_or(false),
        ..Subscription::default()
    };
    let id = subscription.id.clone();
    let state = app.state::<AppState>();
    state.store.lock().unwrap().add_subscription(subscription)?;
    match subscriptions::refresh(&app, &id).await {
        Ok(summary) => Ok(summary),
        Err(error) => {
            state.store.lock().unwrap().remove_subscription(&id)?;
            Err(error)
        }
    }
}

/// Changes a subscription's name, link or request options. The caller refreshes it afterwards.
#[tauri::command]
pub fn edit_subscription(
    state: State<AppState>,
    id: String,
    name: String,
    url: String,
    user_agent: Option<String>,
    send_device_id: bool,
) -> Result<()> {
    subscriptions::validate_url(&url)?;
    let user_agent = subscriptions::clean_user_agent(user_agent.as_deref())?;
    let mut store = state.store.lock().unwrap();
    let mut subscription = store
        .subscriptions()
        .iter()
        .find(|s| s.id == id)
        .cloned()
        .ok_or_else(|| AppError::Invalid("subscription not found".into()))?;
    let name = name.trim();
    if !name.is_empty() {
        subscription.name = name.to_string();
    }
    if subscription.url != url.trim() {
        subscription.url = url.trim().to_string();
        // A new link is a fresh start: no back-off from the old one's failures.
        subscription.failures = 0;
        subscription.last_error = None;
    }
    subscription.user_agent = user_agent;
    subscription.send_device_id = send_device_id;
    store.update_subscription(subscription)
}

#[tauri::command]
pub async fn installed_apps() -> Result<Vec<crate::apps::InstalledApp>> {
    crate::apps::list().await
}

#[tauri::command]
pub async fn refresh_subscription(app: AppHandle, id: String) -> Result<RefreshSummary> {
    subscriptions::refresh(&app, &id).await
}

#[tauri::command]
pub fn remove_subscription(state: State<AppState>, id: String) -> Result<()> {
    state.store.lock().unwrap().remove_subscription(&id)
}

#[tauri::command]
pub async fn run_diagnostics(state: State<'_, AppState>) -> Result<Diagnostics> {
    let connected = state.status.lock().unwrap().state == ConnState::Connected;
    Ok(diagnostics::run(connected).await)
}

#[tauri::command]
pub async fn run_speed_test(app: AppHandle) -> Result<SpeedResult> {
    diagnostics::speed_test(&app).await
}

#[tauri::command]
pub fn cancel_speed_test() {
    diagnostics::cancel_speed_test();
}

#[tauri::command]
pub async fn connections_list(app: AppHandle) -> Result<Vec<clash::Connection>> {
    clash::connections(&app).await
}

#[tauri::command]
pub async fn close_connection(app: AppHandle, id: String) -> Result<()> {
    clash::close_connection(&app, &id).await
}

#[tauri::command]
pub async fn close_all_connections(app: AppHandle) -> Result<()> {
    clash::close_all(&app).await
}

#[tauri::command]
pub fn open_main_window(app: AppHandle) {
    tray::show_main_window(&app);
}

#[tauri::command]
pub fn hide_tray(app: AppHandle) {
    tray::hide_popover(&app);
}

#[tauri::command]
pub fn notify_os(app: AppHandle, title: String, body: String) {
    notify::send(&app, &title, &body);
}

#[tauri::command]
pub fn quit_app(app: AppHandle) {
    tray::quit_app(&app);
}
