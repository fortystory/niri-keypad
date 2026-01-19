use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, EventControllerKey, Orientation, Box, Align, Grid};
use gtk4::glib;
use gtk4::glib::clone;
use gtk4::glib::Propagation;
use gtk4_layer_shell::{Layer, LayerShell, KeyboardMode};
use std::time::Duration;
use crate::state::AppState;
use crate::ui::panels;

pub fn build_window(app: &Application, state: &AppState) -> ApplicationWindow {
    let width = state.config.settings.width;
    let height = state.config.settings.height;
    
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Niri Keypad")
        .default_width(width)
        .default_height(height)
        .decorated(false)
        .build();

    // Layer Shell Setup
    window.init_layer_shell();
    window.set_layer(Layer::Overlay);
    window.set_keyboard_mode(KeyboardMode::Exclusive);
    window.set_anchor(gtk4_layer_shell::Edge::Left, false);
    window.set_anchor(gtk4_layer_shell::Edge::Right, false);
    window.set_anchor(gtk4_layer_shell::Edge::Top, false);
    window.set_anchor(gtk4_layer_shell::Edge::Bottom, false);
    
    let container = Box::new(Orientation::Vertical, 10);
    container.set_margin_top(20);
    container.set_margin_bottom(20);
    container.set_margin_start(20);
    container.set_margin_end(20);
    container.set_valign(Align::Center);
    container.set_halign(Align::Center);
    
    window.add_css_class("keypad-window");
    
    // Build Main Grid
    let main_grid = panels::build_main_grid(state);
    
    // We can directly append the grid to container
    container.append(&main_grid);
    
    window.set_child(Some(&container));
    
    // Refresh Logic
    let grid_clone = main_grid.clone();
    let state_clone_timer = state.clone();
    let last_menu = std::rc::Rc::new(std::cell::RefCell::new(String::new()));
    
    // Clone window for the timer closure
    let window_for_timer = window.clone();
    
    glib::timeout_add_local(Duration::from_millis(50), move || {
        // Check for close signal
        if state_clone_timer.should_close.swap(false, std::sync::atomic::Ordering::Relaxed) {
            window_for_timer.set_visible(false);
        }
    
        let current_menu = state_clone_timer.get_current_menu_name();
        let mut last = last_menu.borrow_mut();
        if *last != current_menu {
            // Full refresh of the grid
            panels::refresh_main_grid(&grid_clone, &state_clone_timer);
            *last = current_menu;
        }
        glib::ControlFlow::Continue
    });

    // --- Controller 1: Capture Phase (For ESC) ---
    // Handles Escape globally before anything else
    let esc_controller = EventControllerKey::new();
    esc_controller.set_propagation_phase(gtk4::PropagationPhase::Capture);
    
    // We only need a weak ref to window to close it
    esc_controller.connect_key_pressed(clone!(@weak window => @default-return Propagation::Proceed, move |_, key, _, _| {
        let key_name = key.name().map(|s| s.to_string()).unwrap_or_default();
        println!("DEBUG: [Capture] Key: {}", key_name);
        if key_name == "Escape" || key_name == "Esc" {
            println!("DEBUG: ESC detected, closing window");
            window.set_visible(false);
            return Propagation::Stop;
        }
        Propagation::Proceed
    }));
    window.add_controller(esc_controller);

    // --- Controller 2: Bubble Phase (For Shortcuts) ---
    // Handles shortcuts F1-F12, A-Z. 
    // Runs in Bubble phase so buttons can consume Arrow Keys / Enter first.
    let char_controller = EventControllerKey::new();
    // Default phase is Bubble
    let state_clone_key = state.clone();
    
    char_controller.connect_key_pressed(clone!(@weak window => @default-return Propagation::Proceed, move |_, key, _, _| {
        let key_name = key.name().map(|s| s.to_string()).unwrap_or_default();
        
        let mut handled = false;
        let mut should_hide = false;
        
        // Helper to match key names with support for aliases (e.g. slash <-> /)
        let matches_key = |config_key: &str, event_key: &str| -> bool {
            if config_key.eq_ignore_ascii_case(event_key) {
                return true;
            }
            
            // Map common symbol names to their char representation for flexible matching
            let aliases = match event_key.to_lowercase().as_str() {
                "slash" => vec!["/"],
                "comma" => vec![","],
                "period" => vec!["."],
                "semicolon" => vec![";"],
                "backslash" => vec!["\\"],
                "bracketleft" => vec!["["],
                "bracketright" => vec!["]"],
                "quote" => vec!["'"],
                "grave" => vec!["`"],
                "minus" => vec!["-"],
                "equal" => vec!["="],
                _ => vec![],
            };
            
            aliases.contains(&config_key)
        };
        
        if key_name.starts_with('F') {
             let globals = &state_clone_key.config.global;
             if let Some(action) = globals.iter().find(|g| matches_key(&g.key, &key_name)) {
                 handled = true;
                 if let Some(cmd) = &action.cmd {
                     let _ = crate::niri::spawn_command(cmd);
                     should_hide = true;
                 } else if let Some(menu_jump) = &action.action {
                     if let Some(menu_name) = menu_jump.strip_prefix("menu:") {
                         state_clone_key.set_menu(menu_name.to_string());
                     }
                 }
             }
        } else {
             // For regular characters, we only process if they are likely shortcuts (len 1 usually)
             // But key_name can be "a", "B", "space", etc.
             // We want to avoid capturing Modifier keys etc alone, but state logic filters by item match.
             
             let menu_name = state_clone_key.get_current_menu_name();
             if let Some(menu) = state_clone_key.config.menu.iter().find(|m| m.name == menu_name) {
                 if let Some(item) = menu.items.iter().find(|i| matches_key(&i.key, &key_name)) {
                     handled = true;
                     if let Some(cmd) = &item.cmd {
                         let _ = crate::niri::spawn_command(cmd);
                         should_hide = true;
                     } else if let Some(act) = &item.action {
                         if let Some(menu_name) = act.strip_prefix("menu:") {
                             state_clone_key.set_menu(menu_name.to_string());
                         }
                     }
                 }
             }
        }
        
        if should_hide {
            window.set_visible(false);
        }
        
        if handled { Propagation::Stop } else { Propagation::Proceed }
    }));
    
    window.add_controller(char_controller);
    
    window
}
