use serde::Deserialize;
use std::path::Path;
use std::fs;
use anyhow::{Context as _, Result};

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    #[serde(default)]
    pub settings: Settings,
    
    #[serde(default)]
    pub global: Vec<GlobalAction>,
    
    #[serde(default)]
    pub menu: Vec<Menu>,
    
    #[serde(default)]
    pub context: Vec<Context>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    #[serde(default = "default_width")]
    pub width: i32,
    #[serde(default = "default_height")]
    pub height: i32,
    pub theme: Option<String>,
    pub icon_theme: Option<String>,
}

fn default_width() -> i32 { 800 }
fn default_height() -> i32 { 600 }

impl Default for Settings {
    fn default() -> Self {
        Self {
            width: default_width(),
            height: default_height(),
            theme: None,
            icon_theme: None,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct GlobalAction {
    pub key: String,
    pub name: String,
    // Provide either simple cmd or complex action
    pub cmd: Option<String>,
    pub action: Option<String>,
    pub icon: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Menu {
    pub name: String,
    pub title: String,
    pub items: Vec<MenuItem>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MenuItem {
    pub key: String,
    pub name: String,
    pub cmd: Option<String>,
    pub action: Option<String>,
    pub icon: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Context {
    pub app_id: String,
    pub menu: String,
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_dir = dirs::config_dir().context("Could not determine config directory")?;
        let config_path = config_dir.join("niri-keypad/config.toml");
        
        Self::load_from_path(&config_path)
    }

    pub fn load_from_path(path: &Path) -> Result<Self> {
        if !path.exists() {
            // Return empty default config if not found? Or error?
            // For now, let's return a basic default
            return Ok(Config {
                settings: Settings::default(),
                global: vec![],
                menu: vec![],
                context: vec![],
            });
        }

        let content = fs::read_to_string(path).context("Failed to read config file")?;
        let config: Config = toml::from_str(&content).context("Failed to parse TOML config")?;
        
        Ok(config)
    }
}

// Helper: dirs dependency is needed if we use dirs::config_dir()
// Let's assume we might need to add `dirs` to Cargo.toml or use std::env::var("HOME") manual fallback
// For simplicity in this step, let's add `dirs` to Cargo.toml in the next step or keep it simple.
// Actually, `dirs` is simpler. I'll add it to Cargo.toml.
