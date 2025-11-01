#![doc = include_str!("../README.md")]
#![warn(clippy::pedantic, missing_docs, unreachable_pub)]

use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use crossbeam_channel::{unbounded, Receiver, Sender};
use directories::ProjectDirs;
use gpui::App;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use unic_langid::LanguageIdentifier;

/// Wrapper type for storing the persisted dock layout in GPUI globals.
#[derive(Debug, Clone, Default)]
pub struct LayoutState(pub String);

impl gpui::Global for LayoutState {}

/// Default application qualifier used for configuration storage.
static QUALIFIER: &str = "dev.multiplex";
static ORGANIZATION: &str = "gpui-learning";
static APPLICATION: &str = "workspace";

/// Persistent workspace configuration stored as JSON.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct WorkspaceConfig {
    /// Serialized window geometry such as size and position.
    pub window_state: Option<String>,
    /// Dock layout serialized by the component gallery.
    pub layout_state: Option<String>,
    /// Recently opened workspace identifiers.
    pub recent_workspaces: Vec<String>,
}

impl WorkspaceConfig {
    /// Adds a workspace to the MRU list while deduplicating previous entries.
    pub fn push_recent(&mut self, workspace_id: impl Into<String>) {
        let id = workspace_id.into();
        self.recent_workspaces.retain(|existing| existing != &id);
        self.recent_workspaces.insert(0, id);
        self.recent_workspaces.truncate(10);
    }
}

/// Errors raised when reading or writing configuration files.
#[derive(Debug, Error)]
pub enum ConfigError {
    /// Wraps underlying IO errors.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    /// Wraps JSON serialization issues.
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Reader/writer responsible for persisting [`WorkspaceConfig`].
#[derive(Debug, Clone)]
pub struct ConfigStore {
    path: PathBuf,
}

impl Default for ConfigStore {
    fn default() -> Self {
        Self::new(default_config_path())
    }
}

impl ConfigStore {
    /// Creates a store rooted at the given path.
    #[must_use]
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    fn ensure_dir(&self) -> Result<(), ConfigError> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        Ok(())
    }

    /// Loads the workspace configuration or returns a default value.
    pub fn load(&self) -> Result<WorkspaceConfig, ConfigError> {
        if !self.path.exists() {
            return Ok(WorkspaceConfig::default());
        }
        let bytes = fs::read(&self.path)?;
        let config = serde_json::from_slice(&bytes)?;
        Ok(config)
    }

    /// Persists the workspace configuration to disk.
    pub fn save(&self, config: &WorkspaceConfig) -> Result<(), ConfigError> {
        self.ensure_dir()?;
        let buffer = serde_json::to_vec_pretty(config)?;
        fs::write(&self.path, buffer)?;
        Ok(())
    }

    /// Returns the backing file path, primarily used in diagnostics.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }
}

fn default_config_path() -> PathBuf {
    ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION)
        .map(|dirs| dirs.config_dir().join("workspace.json"))
        .unwrap_or_else(|| PathBuf::from("workspace.json"))
}

/// Captures optional platform features that can be toggled at runtime.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
pub struct FeatureFlags {
    /// Enables the embedded webview component in gpui-component if available.
    pub webview: bool,
}

impl FeatureFlags {
    /// Hydrates feature flags from environment variables.
    #[must_use]
    pub fn from_env() -> Self {
        let webview = std::env::var("GPUI_FEATURE_WEBVIEW")
            .map(|value| value == "1" || value.eq_ignore_ascii_case("true"))
            .unwrap_or(false);
        Self { webview }
    }
}

/// Minimal localization registry that can be shared between applications.
#[derive(Debug, Clone, Default)]
pub struct LocalizationRegistry {
    bundles: Arc<Mutex<HashMap<LanguageIdentifier, HashMap<String, String>>>>,
    fallback: LanguageIdentifier,
}

impl LocalizationRegistry {
    /// Creates a registry with the given fallback locale.
    #[must_use]
    pub fn new(fallback: LanguageIdentifier) -> Self {
        Self {
            bundles: Arc::new(Mutex::new(HashMap::new())),
            fallback,
        }
    }

    /// Registers translated messages for the specified locale.
    pub fn register_messages<I, K, V>(&self, locale: LanguageIdentifier, entries: I)
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        let mut guard = self.bundles.lock().expect("localization mutex poisoned");
        let bundle = guard.entry(locale).or_default();
        for (key, value) in entries {
            bundle.insert(key.into(), value.into());
        }
    }

    /// Resolves the message for the given key, falling back to the default
    /// locale.
    pub fn translate(&self, locale: &LanguageIdentifier, key: &str) -> Option<String> {
        let guard = self.bundles.lock().expect("localization mutex poisoned");
        guard
            .get(locale)
            .and_then(|bundle| bundle.get(key))
            .or_else(|| guard.get(&self.fallback).and_then(|bundle| bundle.get(key)))
            .cloned()
    }
}

/// Event bus used to distribute domain commands across subsystems.
#[derive(Debug, Clone, Default)]
pub struct CommandBus<T: Clone + Send + 'static> {
    subscribers: Arc<Mutex<Vec<Sender<T>>>>,
}

impl<T: Clone + Send + 'static> CommandBus<T> {
    /// Constructs an empty command bus.
    #[must_use]
    pub fn new() -> Self {
        Self {
            subscribers: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Adds a new subscriber and returns a receiver that yields commands.
    pub fn subscribe(&self) -> Receiver<T> {
        let (tx, rx) = unbounded();
        self.subscribers
            .lock()
            .expect("command bus mutex poisoned")
            .push(tx);
        rx
    }

    /// Broadcasts a command to all subscribers.
    pub fn publish(&self, command: T) {
        let mut subscribers = self.subscribers.lock().expect("command bus mutex poisoned");
        subscribers.retain(|subscriber| subscriber.send(command.clone()).is_ok());
    }
}

/// Applies persisted configuration to the application at startup.
pub fn bootstrap(app: &mut App, store: &ConfigStore) -> Result<WorkspaceConfig, ConfigError> {
    let config = store.load()?;
    if let Some(layout) = &config.layout_state {
        app.set_global(LayoutState(layout.clone()));
    }
    Ok(config)
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn config_roundtrip() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.json");
        let store = ConfigStore::new(path.clone());
        let mut config = WorkspaceConfig::default();
        config.window_state = Some("800x600".into());
        store.save(&config).unwrap();
        let loaded = store.load().unwrap();
        assert_eq!(loaded.window_state, Some("800x600".into()));
        assert_eq!(store.path(), path.as_path());
    }

    #[test]
    fn localization_lookup() {
        let registry = LocalizationRegistry::new("en-US".parse().unwrap());
        registry.register_messages(
            "en-US".parse().unwrap(),
            [("greeting", "Hello"), ("farewell", "Bye")],
        );
        let fr: LanguageIdentifier = "fr-FR".parse().unwrap();
        assert_eq!(registry.translate(&fr, "greeting"), Some("Hello".into()));
    }

    #[test]
    fn command_bus_publish_subscribe() {
        let bus = CommandBus::<String>::new();
        let receiver = bus.subscribe();
        bus.publish("ping".to_string());
        assert_eq!(receiver.recv().ok(), Some("ping".into()));
    }
}
