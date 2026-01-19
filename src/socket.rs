use std::os::unix::net::{UnixListener, UnixStream};
use std::io::Write;
use std::thread;
use std::fs;
use std::sync::mpsc::{channel, Sender, Receiver};
use gtk4::prelude::*;
use gtk4::glib;

use crate::state::AppState;

pub fn start_command_server(window: gtk4::ApplicationWindow, state: AppState) {
    let (sender, receiver) = channel::<String>();
    
    // UI Thread Poller
    glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
        // Drain all messages
        while let Ok(cmd) = receiver.try_recv() {
            // Signal received
            if let Some(rest) = cmd.strip_prefix("OPEN ") {
                 let menu_name = rest.trim();
                 if !menu_name.is_empty() {
                     println!("DEBUG: Socket received menu signal: {}", menu_name);
                     state.set_menu(menu_name.to_string());
                 }
            }
            
            window.set_visible(true);
            window.present();
        }
        glib::ControlFlow::Continue
    });

    thread::spawn(move || {
        let socket_path = "/tmp/niri-keypad.sock";
        let _ = fs::remove_file(socket_path);
        
        let listener = match UnixListener::bind(socket_path) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("Failed to bind command socket: {}", e);
                return;
            }
        };
        
        for stream in listener.incoming() {
             match stream {
                 Ok(mut s) => {
                    // Read data logic - for now let's read the whole thing (simplistic)
                    // Or since start_client just writes one command...
                    use std::io::Read;
                    let mut buf = String::new();
                    if let Ok(_) = s.read_to_string(&mut buf) {
                         let _ = sender.send(buf);
                    }
                 }
                 Err(e) => eprintln!("Connection failed: {}", e),
             }
        }
    });
}

pub fn send_open_signal(menu: Option<String>) -> anyhow::Result<()> {
    let socket_path = "/tmp/niri-keypad.sock";
    let mut stream = UnixStream::connect(socket_path)?;
    let cmd = if let Some(m) = menu {
        format!("OPEN {}", m)
    } else {
        "OPEN".to_string()
    };
    stream.write_all(cmd.as_bytes())?;
    Ok(())
}
