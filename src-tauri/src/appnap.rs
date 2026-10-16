//! Keeps macOS from napping the app while the VPN is connected. App Nap throttles the timers of a
//! menu-bar app whose windows are hidden, which delays the health check and the reconnect logic
//! exactly when they matter. The assertion allows the Mac itself to sleep.

use std::sync::Mutex;

#[cfg(target_os = "macos")]
mod imp {
    use objc2::rc::Retained;
    use objc2::runtime::ProtocolObject;
    use objc2_foundation::{NSActivityOptions, NSObjectProtocol, NSProcessInfo, NSString};

    pub struct Activity(Retained<ProtocolObject<dyn NSObjectProtocol>>);

    // SAFETY: the activity token is only handed back to NSProcessInfo, which is thread-safe, and
    // the holder below is behind a mutex.
    unsafe impl Send for Activity {}

    pub fn begin() -> Option<Activity> {
        let reason = NSString::from_str("Kage VPN is connected");
        Some(Activity(NSProcessInfo::processInfo().beginActivityWithOptions_reason(NSActivityOptions::UserInitiatedAllowingIdleSystemSleep, &reason)))
    }

    pub fn end(activity: Activity) {
        // SAFETY: the token came from `beginActivityWithOptions:reason:` on this same object.
        unsafe { NSProcessInfo::processInfo().endActivity(&activity.0) };
    }
}

#[cfg(not(target_os = "macos"))]
mod imp {
    pub struct Activity;
    pub fn begin() -> Option<Activity> {
        Some(Activity)
    }
    pub fn end(_: Activity) {}
}

static HELD: Mutex<Option<imp::Activity>> = Mutex::new(None);

/// Holds the assertion; does nothing if it is already held.
pub fn hold() {
    let mut held = HELD.lock().unwrap_or_else(|e| e.into_inner());
    if held.is_none() {
        *held = imp::begin();
    }
}

/// Lets go of the assertion; does nothing if it is not held.
pub fn release() {
    let taken = HELD.lock().unwrap_or_else(|e| e.into_inner()).take();
    if let Some(activity) = taken {
        imp::end(activity);
    }
}

#[cfg(test)]
pub fn is_held() -> bool {
    HELD.lock().unwrap_or_else(|e| e.into_inner()).is_some()
}

#[cfg(test)]
#[path = "tests/appnap.rs"]
mod tests;
