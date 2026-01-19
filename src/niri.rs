use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};
use std::thread;
use anyhow::{Result, Context};
use serde_json::Value;

pub fn listen_events<F>(callback: F) -> Result<()> 
where F: Fn(Value) + Send + 'static 
{
    let mut child = Command::new("niri")
        .args(["msg", "-j", "event-stream"])
        .stdout(Stdio::piped())
        .spawn()
        .context("Failed to spawn niri msg event-stream")?;

    let stdout = child.stdout.take().context("Failed to open stdout")?;
    let reader = BufReader::new(stdout);

    thread::spawn(move || {
        for line in reader.lines() {
            match line {
                Ok(l) => {
                    if let Ok(json) = serde_json::from_str::<Value>(&l) {
                        callback(json);
                    } else {
                        eprintln!("Failed to parse JSON from niri: {}", l);
                    }
                }
                Err(e) => eprintln!("Error reading niri event stream: {}", e),
            }
        }
    });

    Ok(())
}

pub fn get_app_id_for_window(id: u64) -> Result<Option<String>> {
    let output = Command::new("niri")
        .args(["msg", "--json", "windows"])
        .output()
        .context("Failed to execute niri msg windows")?;

    if !output.status.success() {
        return Ok(None);
    }

    let json: Value = serde_json::from_slice(&output.stdout)?;
    
    if let Some(windows) = json.as_array() {
        for w in windows {
            if let Some(w_id) = w.get("id").and_then(|v| v.as_u64()) {
                if w_id == id {
                    return Ok(w.get("app_id").and_then(|v| v.as_str()).map(|s| s.to_string()));
                }
            }
        }
    }
    
    Ok(None)
}

pub fn focus_window(app_id: &str) -> Result<()> {
    Command::new("niri")
        .args(["msg", "action", "focus-window", "--app-id", app_id])
        .spawn()
        .context("Failed to run niri msg action focus-window")?;
    Ok(())
}

pub fn spawn_command(cmd_str: &str) -> Result<()> {
    // Run an arbitrary shell command
    Command::new("sh")
        .arg("-c")
        .arg(cmd_str)
        .spawn()
        .context("Failed to spawn shell command")?;
    Ok(())
}
