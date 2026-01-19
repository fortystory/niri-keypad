mod config;
mod niri;
mod state;
mod ui;
mod socket;

use clap::{Parser, Subcommand};
use gtk4::prelude::*;
use gtk4::{Application};
use state::AppState;

#[derive(Parser)]
#[command(name = "niri-keypad")]
#[command(version = "0.1.0")]
#[command(about = "A global helper for Niri")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the daemon server
    Server,
    /// Open the keypad window (client command)
    Open {
        /// Optional: Force open a specific menu
        #[arg(long)]
        menu: Option<String>,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Server) => run_server(),
        Some(Commands::Open { menu }) => run_client(menu),
        None => run_client(None), 
    }
}

fn run_server() -> anyhow::Result<()> {
    // initialize GTK
    gtk4::init()?;

    let config = match config::Config::load() {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Warning: Failed to load config: {}", e);
            return Err(e);
        }
    };
    
    let state = AppState::new(config);

    // State for Niri Listener
    let state_for_niri = state.clone();
    
    // Start Niri Event Listener
    if let Err(e) = niri::listen_events(move |json| {
        // println!("DEBUG: RAW EVENT: {:?}", json); // Too noisy? 
        
        if let Some(obj) = json.as_object() {
            // Print top-level keys to help debugging
            // println!("DEBUG: Event Type: {:?}", obj.keys());
            
            // Try both potential keys for focus change
            let focus_data = json.get("WindowFocused").or_else(|| json.get("WindowFocusChanged"));

            if let Some(focus) = focus_data {
                 // Check if it's the new nested structure or direct structure
                 // Niri 0.1.x might be different. Let's look for known fields.
                 
                 // Sometimes focus event is just ID, and we need to query window? 
                 // Or it contains full info. 
                 // Let's print what we got.
                 println!("DEBUG: Focus Event Data: {:?}", focus);

                 let mut app_id = focus.get("app_id").and_then(|v| v.as_str()).map(|s| s.to_string());
                 
                 // If app_id missing but we have ID, query it
                 if app_id.is_none() {
                     if let Some(id) = focus.get("id").and_then(|v| v.as_u64()) {
                         println!("DEBUG: App ID missing in event, querying for window ID: {}", id);
                         if let Ok(Some(found_id)) = niri::get_app_id_for_window(id) {
                             app_id = Some(found_id);
                         }
                     } else {
                         // ID is Null or missing. This likely means focus went to the overlay (keypad)
                         // or desktop. We should IGNORE this to preserve the last known app context.
                         println!("DEBUG: Focus event with Null/Missing ID. Ignoring to preserve context.");
                         return; 
                     }
                 }
                 
                 // If app_id is effectively empty, treat as None
                 let app_id = if let Some(ref s) = app_id {
                     if s.is_empty() { None } else { Some(s.clone()) }
                 } else {
                     None
                 };
                 
                 println!("DEBUG: Detected App ID: {:?}", app_id);
                 state_for_niri.set_app_id(app_id);
                 
                 // Signal UI to close on focus change
                 state_for_niri.should_close.store(true, std::sync::atomic::Ordering::Relaxed);
            }
        }
    }) {
        eprintln!("Failed to start niri listener: {}", e);
    }

    let app = Application::builder()
        .application_id("org.niri.keypad")
        .build();

    // State for UI
    let state_for_ui = state.clone();
    
    // Channel to control window visibility from other threads (e.g. Niri event listener)
    // We can reuse the socket channel approach or add a new one?
    // Actually, `niri::listen_events` is independent.
    // Let's just make the window hidden by default.
    
    app.connect_activate(move |app| {
        load_css();
        
        // Apply Icon Theme if configured
        if let Some(display) = gtk4::gdk::Display::default() {
            if let Some(theme_name) = &state_for_ui.config.settings.icon_theme {
                println!("DEBUG: Setting icon theme to: {}", theme_name);
                let icon_theme = gtk4::IconTheme::for_display(&display);
                icon_theme.set_theme_name(Some(theme_name));
            }
        }

        let window = ui::window::build_window(app, &state_for_ui);
        // Do NOT call window.present() initially, capturing it as hidden daemon.
        window.set_visible(false); 
        
        // Start command socket listener
        let state_for_socket = state_for_ui.clone();
        crate::socket::start_command_server(window.clone(), state_for_socket);
    });
    
    app.run_with_args(&Vec::<String>::new());
    Ok(())
}

fn load_css() {
    // Load CSS
    let provider = gtk4::CssProvider::new();
    provider.load_from_data(
        "
        .name-label {
            font-size: 11px;
            opacity: 0.8;
        }
        .key-label {
            font-size: 20px;
            font-weight: bold;
        }
        .key-card {
            padding: 4px;
        }
        "
    );

    // Apply to default display
    if let Some(display) = gtk4::gdk::Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}

fn run_client(menu: Option<String>) -> anyhow::Result<()> {
    crate::socket::send_open_signal(menu)?;
    Ok(())
}
