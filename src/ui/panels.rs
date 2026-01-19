use gtk4::prelude::*;
use gtk4::{Box, Button, Label, Orientation, Grid, Image, Align}; 
use crate::state::AppState;
use std::path::Path;

fn create_icon_widget(icon_def: &Option<String>) -> gtk4::Widget {
    if let Some(icon_str) = icon_def {
        // Case 1: File Path
        if icon_str.contains("/") || icon_str.contains("\\") {
             if Path::new(icon_str).exists() {
                 let img = Image::from_file(icon_str);
                 img.set_pixel_size(32);
                 return img.upcast();
             }
        }
        
        // Case 2: Named Icon from Theme
        // We need to check if the icon exists in the current theme
        if let Some(display) = gtk4::gdk::Display::default() {
            let icon_theme = gtk4::IconTheme::for_display(&display);
            if icon_theme.has_icon(icon_str) {
                let img = Image::from_icon_name(icon_str);
                img.set_pixel_size(32);
                return img.upcast();
            }
        }
        
        // Case 3: Fallback to Nerd Font / Text
        let label = Label::builder()
            .label(icon_str)
            .css_classes(vec!["nerd-icon".to_string()])
            .build();
        return label.upcast();
    }
    
    // Default empty placeholder
    Label::new(None).upcast()
}

fn format_key_for_display(key: &str) -> String {
    match key.to_lowercase().as_str() {
        "slash" => "/".to_string(),
        "comma" => ",".to_string(),
        "period" => ".".to_string(),
        "semicolon" => ";".to_string(),
        "backslash" => "\\".to_string(),
        "bracketleft" => "[".to_string(),
        "bracketright" => "]".to_string(),
        "quote" => "'".to_string(),
        "grave" => "`".to_string(),
        "minus" => "-".to_string(),
        "equal" => "=".to_string(),
        // Keep original if no match
        _ => key.to_string(),
    }
}

// Helper: Build a card widget
// Updated to support icon, distinct name/key display, and click interactions
fn create_card(key: &str, name: &str, icon: &Option<String>, 
               cmd: Option<String>, menu_action: Option<String>, 
               state: &AppState) -> Button {
    let btn = Button::builder()
        .css_classes(vec!["key-card".to_string()])
        // Enforce fixed size for alignment - Increased to 140x140
        .width_request(140)
        .height_request(140)
        .focusable(true) // Allow focus for keyboard navigation/Enter key
        .build();
    
    let container = Box::new(Orientation::Vertical, 6);
    container.set_valign(Align::Center);
    container.set_halign(Align::Center);
    
    // Top: Icon or Name
    let top_widget = create_icon_widget(icon);
    
    let display_key = format_key_for_display(key);
    
    let lbl_key = Label::builder()
        .label(&display_key)
        .css_classes(vec!["key-label".to_string()])
        .build();
        
    let lbl_name = Label::builder()
        .label(name)
        .css_classes(vec!["name-label".to_string()])
        .ellipsize(gtk4::pango::EllipsizeMode::End) 
        .max_width_chars(15)
        .build();

    container.append(&top_widget);
    container.append(&lbl_name);
    container.append(&lbl_key);
    
    btn.set_child(Some(&container));
    
    // Interaction Handlers
    if cmd.is_some() || menu_action.is_some() {
        let state_clone = state.clone();
        let cmd_clone = cmd.clone();
        let menu_clone = menu_action.clone();
        
        btn.connect_clicked(move |_| {
            println!("DEBUG: Button clicked");
            if let Some(c) = &cmd_clone {
                println!("DEBUG: Executing cmd: {}", c);
                let _ = crate::niri::spawn_command(c);
                // Signal UI to close
                state_clone.should_close.store(true, std::sync::atomic::Ordering::Relaxed);
            } else if let Some(act) = &menu_clone {
                 if let Some(menu_name) = act.strip_prefix("menu:") {
                     println!("DEBUG: Switching to menu: {}", menu_name);
                     state_clone.set_menu(menu_name.to_string());
                     // UI refresh handles by timer or we could trigger update
                 }
            }
        });
    }
    
    btn
}

// ... update callers in build_main_grid, refresh_main_grid ...

pub fn build_main_grid(state: &AppState) -> Grid {
    let grid = Grid::builder()
        .column_spacing(10)
        .row_spacing(10)
        .column_homogeneous(true)
        // .row_homogeneous(true) // Disable to allow spacer row to be thinner
        .halign(Align::Center)
        .valign(Align::Center)
        .margin_top(20)
        .margin_bottom(20)
        .build();
    
    refresh_main_grid(&grid, state);
    
    grid
}

pub fn refresh_main_grid(grid: &Grid, state: &AppState) {
    // Clear all children first
    while let Some(child) = grid.first_child() {
        grid.remove(&child);
    }
    
    // --- ROW 0: Global F-Keys ---
    let globals = &state.config.global;
    
    for (i, num) in (1..=12).enumerate() {
        let key = format!("F{}", num);
        let action = globals.iter().find(|g| g.key.eq_ignore_ascii_case(&key));
        
        let (name, icon, cmd, act) = if let Some(a) = action {
            (a.name.as_str(), &a.icon, a.cmd.clone(), a.action.clone())
        } else {
            ("", &None, None, None)
        };
        
        let card = create_card(&key, name, icon, cmd, act, state);
        if action.is_none() {
             card.set_opacity(0.3);
             card.set_sensitive(false);
             card.set_can_focus(false);
        }
        
        // F1-F12 occupy Col 0 to 11
        grid.attach(&card, i as i32, 0, 1, 1);
    }
    
    // --- ROW 1: Spacer ---
    // Explicitly add a Transparent Spacer to push Row 2 down
    let spacer = Box::new(Orientation::Vertical, 0);
    spacer.set_height_request(40); // 40px gap
    // Span entire width
    grid.attach(&spacer, 0, 1, 12, 1); 

    // --- ROW 2, 3, 4: Context Menu ---
    let menu_name = state.get_current_menu_name();
    let current_menu = state.config.menu.iter().find(|m| m.name == menu_name);
    
    let items = match current_menu {
        Some(m) => &m.items,
        None => &vec![], 
    };
    
    // Keypad Layout
    let rows = vec![
        vec!["Q", "W", "E", "R", "T", "Y", "U", "I", "O", "P"],
        vec!["A", "S", "D", "F", "G", "H", "J", "K", "L", ";"],
        vec!["Z", "X", "C", "V", "B", "N", "M", ",", ".", "/"],
    ];
    
    // Starting at Grid Row 2
    let start_row = 2;
    
    // ... (Loop for context menu items)
    for (r_idx, row_keys) in rows.iter().enumerate() {
        for (idx, key_char) in row_keys.iter().enumerate() {
            // Logic: 5 keys, then 2 spacers, then 5 keys.
            // idx 0..4 (5 keys) -> Col 0..4
            // idx 5..9 (5 keys) -> Col 7..11 (skip 5, 6)
            
            let target_col = if idx >= 5 { idx + 2 } else { idx };
            let target_row = start_row + r_idx;

            let item = items.iter().find(|i| i.key.eq_ignore_ascii_case(key_char));
            let (name, icon, cmd, act) = if let Some(i) = item {
                (i.name.as_str(), &i.icon, i.cmd.clone(), i.action.clone())
            } else {
                ("", &None, None, None)
            };
            
            let btn = create_card(key_char, name, icon, cmd, act, state);
            
            if item.is_none() {
                btn.set_opacity(0.3);
                btn.set_sensitive(false);
                btn.set_can_focus(false);
            }
            
            grid.attach(&btn, target_col as i32, target_row as i32, 1, 1);
        }
    }
    
    // Explicitly focus the first focusable child
    // This allows Arrow Key navigation to work immediately
    let mut child = grid.first_child();
    while let Some(widget) = child {
        if widget.can_focus() && widget.is_sensitive() {
            widget.grab_focus();
            break;
        }
        child = widget.next_sibling();
    }
}
