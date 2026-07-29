//! In-app terminal sessions backed by a native pseudoterminal.

use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use parking_lot::Mutex;
use portable_pty::{native_pty_system, CommandBuilder, MasterPty, PtySize};
use serde::Serialize;
use std::{
    collections::HashMap,
    io::{Read, Write},
    path::PathBuf,
    sync::{Arc, LazyLock},
};
use tauri::{AppHandle, Emitter};
use uuid::Uuid;

struct TerminalSession {
    master: Box<dyn MasterPty + Send>,
    writer: Box<dyn Write + Send>,
    child: Box<dyn portable_pty::Child + Send + Sync>,
}

static TERMINALS: LazyLock<Mutex<HashMap<String, Arc<Mutex<TerminalSession>>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct TerminalOutput {
    terminal_id: String,
    data_base64: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct TerminalExit {
    terminal_id: String,
}

fn shell_command() -> CommandBuilder {
    #[cfg(windows)]
    {
        let shell = std::env::var("COMSPEC").unwrap_or_else(|_| "powershell.exe".into());
        CommandBuilder::new(shell)
    }
    #[cfg(not(windows))]
    {
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".into());
        let mut command = CommandBuilder::new(shell);
        command.arg("-l");
        command
    }
}

#[tauri::command]
pub async fn terminal_start(
    app: AppHandle,
    project_path: Option<String>,
    cols: u16,
    rows: u16,
) -> Result<String, String> {
    let pair = native_pty_system()
        .openpty(PtySize {
            rows: rows.max(2),
            cols: cols.max(2),
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(|error| error.to_string())?;

    let mut command = shell_command();
    if let Some(path) = project_path
        .map(PathBuf::from)
        .filter(|path| path.is_dir())
    {
        command.cwd(path);
    }

    let child = pair
        .slave
        .spawn_command(command)
        .map_err(|error| error.to_string())?;
    drop(pair.slave);
    let mut reader = pair
        .master
        .try_clone_reader()
        .map_err(|error| error.to_string())?;
    let writer = pair
        .master
        .take_writer()
        .map_err(|error| error.to_string())?;

    let terminal_id = Uuid::new_v4().to_string();
    TERMINALS.lock().insert(
        terminal_id.clone(),
        Arc::new(Mutex::new(TerminalSession {
            master: pair.master,
            writer,
            child,
        })),
    );

    let event_id = terminal_id.clone();
    std::thread::spawn(move || {
        let mut buffer = [0_u8; 8192];
        loop {
            match reader.read(&mut buffer) {
                Ok(0) | Err(_) => break,
                Ok(count) => {
                    let _ = app.emit(
                        "terminal://output",
                        TerminalOutput {
                            terminal_id: event_id.clone(),
                            data_base64: BASE64.encode(&buffer[..count]),
                        },
                    );
                }
            }
        }
        TERMINALS.lock().remove(&event_id);
        let _ = app.emit(
            "terminal://exit",
            TerminalExit {
                terminal_id: event_id,
            },
        );
    });

    Ok(terminal_id)
}

#[tauri::command]
pub async fn terminal_write(terminal_id: String, data: String) -> Result<(), String> {
    let session = TERMINALS
        .lock()
        .get(&terminal_id)
        .cloned()
        .ok_or_else(|| "terminal session not found".to_string())?;
    let mut session = session.lock();
    session
        .writer
        .write_all(data.as_bytes())
        .and_then(|_| session.writer.flush())
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn terminal_resize(
    terminal_id: String,
    cols: u16,
    rows: u16,
) -> Result<(), String> {
    let session = TERMINALS
        .lock()
        .get(&terminal_id)
        .cloned()
        .ok_or_else(|| "terminal session not found".to_string())?;
    let result = session
        .lock()
        .master
        .resize(PtySize {
            rows: rows.max(2),
            cols: cols.max(2),
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(|error| error.to_string());
    result
}

#[tauri::command]
pub async fn terminal_stop(terminal_id: String) -> Result<(), String> {
    let Some(session) = TERMINALS.lock().remove(&terminal_id) else {
        return Ok(());
    };
    let mut session = session.lock();
    session.child.kill().map_err(|error| error.to_string())?;
    let _ = session.child.wait();
    Ok(())
}
