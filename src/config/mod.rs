//! Application configuration.

use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::i18n::Locale;
use crate::ui::theme::ThemeMode;

pub const DEFAULT_PORT: u16 = 8080;
pub const DEFAULT_BIND: &str = "0.0.0.0";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub name: String,
    pub password: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub port: u16,
    pub bind: String,
    pub expert_mode: bool,
    pub auto_copy_url_on_add: bool,
    pub send_server_header: bool,
    pub open_in_browser_use_localhost: bool,
    pub show_bandwidth_graph: bool,
    #[serde(default)]
    pub locale: Locale,
    #[serde(default)]
    pub theme_mode: ThemeMode,
    /// Soft cap; `None` means unlimited.
    pub max_connections: Option<usize>,
    /// Allow browser/multipart uploads into real folders.
    #[serde(default = "default_true")]
    pub allow_upload: bool,
    /// Max single upload size in MiB (0 = unlimited).
    #[serde(default = "default_upload_max")]
    pub upload_max_mb: u64,
    /// Require HTTP basic auth for uploads when accounts exist.
    #[serde(default)]
    pub protect_uploads: bool,
    #[serde(default)]
    pub accounts: Vec<Account>,
}

fn default_upload_max() -> u64 {
    512
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            port: DEFAULT_PORT,
            bind: DEFAULT_BIND.to_string(),
            expert_mode: false,
            auto_copy_url_on_add: true,
            send_server_header: true,
            open_in_browser_use_localhost: true,
            show_bandwidth_graph: true,
            locale: Locale::System,
            theme_mode: ThemeMode::System,
            max_connections: None,
            allow_upload: true,
            upload_max_mb: default_upload_max(),
            protect_uploads: false,
            accounts: Vec::new(),
        }
    }
}

impl AppConfig {
    pub fn config_path() -> PathBuf {
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.join("hfs-rs.json")))
            .unwrap_or_else(|| PathBuf::from("hfs-rs.json"))
    }

    pub fn load_or_default() -> Self {
        let path = Self::config_path();
        Self::load_from(&path).unwrap_or_default()
    }

    pub fn load_from(path: &Path) -> Option<Self> {
        let raw = fs::read_to_string(path).ok()?;
        serde_json::from_str(&raw).ok()
    }

    pub fn save(&self) -> anyhow::Result<()> {
        self.save_to(&Self::config_path())
    }

    pub fn save_to(&self, path: &Path) -> anyhow::Result<()> {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let raw = serde_json::to_string_pretty(self)?;
        fs::write(path, raw)?;
        Ok(())
    }

    pub fn listen_addr(&self) -> String {
        format!("{}:{}", self.bind, self.port)
    }

    pub fn public_base_url(&self) -> String {
        let host = if self.open_in_browser_use_localhost || self.bind == "0.0.0.0" {
            "127.0.0.1"
        } else {
            self.bind.as_str()
        };
        if self.port == 80 {
            format!("http://{host}/")
        } else {
            format!("http://{host}:{}/", self.port)
        }
    }

    pub fn upload_max_bytes(&self) -> Option<u64> {
        if self.upload_max_mb == 0 {
            None
        } else {
            Some(self.upload_max_mb.saturating_mul(1024 * 1024))
        }
    }

    pub fn find_account(&self, user: &str, pass: &str) -> bool {
        self.accounts
            .iter()
            .any(|a| a.enabled && a.name == user && a.password == pass)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn roundtrip_json() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("cfg.json");
        let mut cfg = AppConfig {
            port: 9090,
            expert_mode: true,
            locale: Locale::En,
            allow_upload: false,
            ..Default::default()
        };
        cfg.accounts.push(Account {
            name: "admin".into(),
            password: "secret".into(),
            enabled: true,
        });
        cfg.save_to(&path).unwrap();
        let loaded = AppConfig::load_from(&path).unwrap();
        assert_eq!(loaded.port, 9090);
        assert!(loaded.expert_mode);
        assert_eq!(loaded.locale, Locale::En);
        assert!(!loaded.allow_upload);
        assert_eq!(loaded.accounts.len(), 1);
    }
}
