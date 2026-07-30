use serde::Serialize;
use std::path::{Path, PathBuf};
use tokio::process::Command;

use crate::store::RemoteRuntimeSettings;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteRuntimeProbe {
    pub ok: bool,
    pub version: Option<String>,
    pub error: Option<String>,
}

fn safe_account_part(value: &str) -> bool {
    !value.is_empty()
        && !value.starts_with('-')
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-' | ':'))
}

pub fn validate(settings: &RemoteRuntimeSettings) -> Result<(), String> {
    if !safe_account_part(settings.host.trim()) {
        return Err("REMOTE_HOST_INVALID".into());
    }
    if !safe_account_part(settings.user.trim()) {
        return Err("REMOTE_USER_INVALID".into());
    }
    if settings.port == 0 {
        return Err("REMOTE_PORT_INVALID".into());
    }
    for value in [&settings.pi_path, &settings.cwd, &settings.identity_file] {
        if value.contains(['\n', '\r', '\0']) {
            return Err("REMOTE_VALUE_INVALID".into());
        }
    }
    if settings.pi_path.trim().is_empty() || settings.cwd.trim().is_empty() {
        return Err("REMOTE_VALUE_INVALID".into());
    }
    Ok(())
}

pub fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

fn remote_path(value: &str) -> String {
    if value == "~" {
        return "\"$HOME\"".into();
    }
    if let Some(rest) = value.strip_prefix("~/") {
        return format!("\"$HOME\"/{}", shell_quote(rest));
    }
    shell_quote(value)
}

pub fn ssh_executable() -> Option<PathBuf> {
    which::which("ssh").ok().or_else(|| {
        #[cfg(target_os = "windows")]
        {
            let root = std::env::var_os("WINDIR")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from(r"C:\Windows"));
            let candidate = root.join("System32").join("OpenSSH").join("ssh.exe");
            if candidate.is_file() {
                return Some(candidate);
            }
        }
        None
    })
}

pub fn configure_ssh_command(
    command: &mut Command,
    settings: &RemoteRuntimeSettings,
    remote_command: &str,
) -> Result<(), String> {
    validate(settings)?;
    command.args([
        "-T",
        "-o",
        "BatchMode=yes",
        "-o",
        "StrictHostKeyChecking=yes",
        "-o",
        "ConnectTimeout=12",
    ]);
    command.arg("-p").arg(settings.port.to_string());
    let identity = settings.identity_file.trim();
    if !identity.is_empty() {
        let expanded = crate::cli_probe::expand_user_path(identity);
        if !Path::new(&expanded).is_file() {
            return Err("REMOTE_IDENTITY_NOT_FOUND".into());
        }
        command.arg("-i").arg(expanded);
    }
    command
        .arg(format!("{}@{}", settings.user.trim(), settings.host.trim()))
        .arg("--")
        .arg(remote_command);
    Ok(())
}

pub fn pi_rpc_command(settings: &RemoteRuntimeSettings, extra_args: &[String]) -> String {
    let mut parts = vec![
        format!("cd {}", remote_path(settings.cwd.trim())),
        format!(
            "exec {} --mode rpc --approve",
            shell_quote(settings.pi_path.trim())
        ),
    ];
    if !extra_args.is_empty() {
        parts[1].push(' ');
        parts[1].push_str(
            &extra_args
                .iter()
                .map(|value| shell_quote(value))
                .collect::<Vec<_>>()
                .join(" "),
        );
    }
    parts.join(" && ")
}

#[tauri::command]
pub async fn remote_runtime_test(
    settings: RemoteRuntimeSettings,
) -> Result<RemoteRuntimeProbe, String> {
    validate(&settings)?;
    let ssh = ssh_executable().ok_or("REMOTE_SSH_MISSING")?;
    let check = format!(
        "cd {} && {} --version",
        remote_path(settings.cwd.trim()),
        shell_quote(settings.pi_path.trim())
    );
    let mut command = Command::new(ssh);
    configure_ssh_command(&mut command, &settings, &check)?;
    command.kill_on_drop(true);
    crate::process_util::apply_no_window_tokio(&mut command);
    let output = command
        .output()
        .await
        .map_err(|error| format!("REMOTE_CONNECT_FAILED: {error}"))?;
    if output.status.success() {
        let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
        return Ok(RemoteRuntimeProbe {
            ok: true,
            version: (!version.is_empty()).then_some(version),
            error: None,
        });
    }
    let error = String::from_utf8_lossy(&output.stderr).trim().to_string();
    Ok(RemoteRuntimeProbe {
        ok: false,
        version: None,
        error: Some(if error.is_empty() {
            "REMOTE_CONNECT_FAILED".into()
        } else {
            error
        }),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quotes_posix_values_without_command_injection() {
        assert_eq!(shell_quote("/work/pi app"), "'/work/pi app'");
        assert_eq!(shell_quote("a'b"), "'a'\"'\"'b'");
        assert_eq!(remote_path("~"), "\"$HOME\"");
        assert_eq!(remote_path("~/work tree"), "\"$HOME\"/'work tree'");
    }

    #[test]
    fn rejects_option_and_control_character_injection() {
        let mut settings = RemoteRuntimeSettings::default();
        settings.host = "-oProxyCommand=bad".into();
        settings.user = "dev".into();
        assert!(validate(&settings).is_err());
        settings.host = "server.example".into();
        settings.cwd = "/work\nbad".into();
        assert!(validate(&settings).is_err());
    }
}
