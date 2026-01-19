use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub current_app_id: Arc<Mutex<Option<String>>>,
    pub active_menu: Arc<Mutex<Option<String>>>, 
    pub should_close: Arc<AtomicBool>, // Signal to close UI
}

impl AppState {
    pub fn new(config: Config) -> Self {
        Self {
            config: Arc::new(config),
            current_app_id: Arc::new(Mutex::new(None)),
            active_menu: Arc::new(Mutex::new(None)),
            should_close: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn set_app_id(&self, app_id: Option<String>) {
        if let Ok(mut id) = self.current_app_id.lock() {
            *id = app_id;
            // Also reset active menu to default context whenever app changes
            // logic can be refined later
            if let Ok(mut menu) = self.active_menu.lock() {
                *menu = None; // None means "auto-resolve based on context"
            }
        }
    }

    pub fn set_menu(&self, menu: String) {
        if let Ok(mut m) = self.active_menu.lock() {
            *m = Some(menu);
        }
    }
    
    pub fn get_current_menu_name(&self) -> String {
        // Priority: 
        // 1. Manually active menu
        // 2. Resolve from Context based on App ID
        // 3. "default" or empty
        
        // 1
        if let Ok(menu) = self.active_menu.lock() {
            if let Some(name) = menu.as_ref() {
                return name.clone();
            }
        }
        
        // 2
        if let Ok(app_id_guard) = self.current_app_id.lock() {
            if let Some(app_id) = app_id_guard.as_ref() {
                // Find matching context
                for ctx in &self.config.context {
                    if &ctx.app_id == app_id {
                        return ctx.menu.clone();
                    }
                }
            }
        }
        
        // 3. Fallback
        for ctx in &self.config.context {
            if ctx.app_id == "default" {
                return ctx.menu.clone();
            }
        }
        
        "root".to_string()
    }
}
