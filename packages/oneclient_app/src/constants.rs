pub const WINDOW_APP_ID: &str = oneclient_common::brand::WINDOW_APP_ID;
pub const WINDOW_TITLE: &str = oneclient_common::brand::APP_NAME;

/// Self-update stays off until VeloxClient publishes its own signed releases;
/// the endpoint below still points at Polyfrost and would install OneClient.
pub const UPDATER_ENABLED: bool = false;

pub const UPDATER_PUBKEY: &str = "dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IDFGODk3MkMyMjg0MjFDMDUKUldRRkhFSW93bktKSHpkWjNEMXNzaDVINVpCTU8xSnhuK2RnV0dTZ2FkcFJWbG1zUkhGYTNjaUkK";
pub const UPDATER_ENDPOINT: &str =
    "https://github.com/Polyfrost/OneLauncher/releases/latest/download/latest.json";
pub const RELEASES_URL: &str = "https://github.com/Polyfrost/OneLauncher/releases/latest";
