//! VeloxClient identity: every user-visible name and system identifier lives here,
//! so rebranding never means hunting through the codebase again.
//!
//! VeloxClient is based on OneClient by Polyfrost (GPL-3.0).

/// Product name shown in windows, tray, notifications and logs.
pub const APP_NAME: &str = "VeloxClient";

/// Folder that groups VeloxClient's data inside the OS data directory.
pub const ORGANIZATION: &str = "VeloxClient";

/// Reverse-DNS qualifier used by legacy per-app folders.
pub const QUALIFIER: &str = "dev";

/// Reverse-DNS bundle identifier (macOS bundle id, Windows pipe names, Linux app id).
pub const BUNDLE_ID: &str = "dev.veloxclient.VeloxClient";

/// Window class / Wayland app id.
pub const WINDOW_APP_ID: &str = "veloxclient";

/// Name of the per-app folder; debug builds never share data with release builds.
#[cfg(not(debug_assertions))]
pub const APPLICATION: &str = "VeloxClient";
#[cfg(debug_assertions)]
pub const APPLICATION: &str = "VeloxClient-dev";

/// Polyfrost's own online services (Poly+, crash reporting, cloud sync, sharing,
/// remote translations). Off: VeloxClient behaves as if their terms were declined,
/// so the network layer blocks every gated Polyfrost host and sentry.io.
pub const POLYFROST_SERVICES: bool = false;

/// Discord Rich Presence. Off until VeloxClient has its own Discord application.
pub const DISCORD_RPC: bool = false;

/// User agent sent to Modrinth, CurseForge and other APIs.
#[must_use]
pub fn user_agent() -> String {
    format!(
        "{APP_NAME}/{} (based on OneClient by Polyfrost)",
        env!("CARGO_PKG_VERSION")
    )
}
