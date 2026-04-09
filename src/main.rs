use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::PathBuf;
use std::process::Command;

#[derive(Parser)]
#[command(name = "sshmgr", about = "Minimal SSH session manager")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Add {
        name: String,
        user: String,
        host: String,
        port: u16,
    },
    List,
    Connect {
        name: String,
        #[arg(long)]
        plain: bool,
    },
    Remove {
        name: String,
    },
}

#[derive(Serialize, Deserialize)]
struct Host {
    name: String,
    user: String,
    host: String,
    port: u16,
}

#[derive(Serialize, Deserialize, Default)]
struct HostsFile {
    hosts: Vec<Host>,
}

fn config_path() -> Result<PathBuf, String> {
    let base = dirs::config_dir()
        .ok_or_else(|| String::from("Could not determine config directory"))?;
    Ok(base.join("sshmgr").join("hosts.json"))
}

fn load_hosts(path: &PathBuf) -> Result<HostsFile, String> {
    if !path.exists() {
        return Ok(HostsFile::default());
    }

    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read config file {}: {}", path.display(), e))?;

    serde_json::from_str::<HostsFile>(&content)
        .map_err(|e| format!("Failed to parse config file {}: {}", path.display(), e))
}

fn save_hosts(path: &PathBuf, data: &HostsFile) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            format!(
                "Failed to create config directory {}: {}",
                parent.display(),
                e
            )
        })?;
    }

    let json = serde_json::to_string_pretty(data)
        .map_err(|e| format!("Failed to serialize config data: {}", e))?;

    fs::write(path, json)
        .map_err(|e| format!("Failed to write config file {}: {}", path.display(), e))
}

fn add_host(path: &PathBuf, name: String, user: String, host: String, port: u16) -> Result<(), String> {
    let mut data = load_hosts(path)?;

    if data.hosts.iter().any(|h| h.name == name) {
        return Err(format!("Host '{}' already exists", name));
    }

    data.hosts.push(Host {
        name,
        user,
        host,
        port,
    });

    save_hosts(path, &data)
}

fn list_hosts(path: &PathBuf) -> Result<(), String> {
    let data = load_hosts(path)?;

    if data.hosts.is_empty() {
        println!("No hosts saved.");
        return Ok(());
    }

    for host in data.hosts {
        println!("{} {}@{}:{}", host.name, host.user, host.host, host.port);
    }

    Ok(())
}

fn connect_host_with_mode(path: &PathBuf, name: String, plain: bool) -> Result<(), String> {
    let data = load_hosts(path)?;
    let host = data
        .hosts
        .iter()
        .find(|h| h.name == name)
        .ok_or_else(|| format!("Host '{}' not found", name))?;

    let mut command = build_connect_command(host, plain);
    let program = command.get_program().to_string_lossy().into_owned();

    let status = command
        .status()
        .map_err(|e| format!("Failed to execute {} command: {}", program, e))?;

    if status.success() {
        return Ok(());
    }

    let exit_name = if plain { "ssh" } else { "tmux-backed ssh" };

    match status.code() {
        Some(code) => Err(format!("{} exited with status code {}", exit_name, code)),
        None => Err(format!("{} terminated by signal", exit_name)),
    }
}

fn build_connect_command(host: &Host, plain: bool) -> Command {
    let destination = format!("{}@{}", host.user, host.host);
    let mut command = if plain {
        Command::new("ssh")
    } else {
        Command::new("tmux")
    };
    if plain {
        command.arg("-p").arg(host.port.to_string()).arg(destination);
    } else {
        command
            .arg("new-session")
            .arg("-A")
            .arg("-s")
            .arg(tmux_session_name(&host.name))
            .arg("ssh")
            .arg("-p")
            .arg(host.port.to_string())
            .arg(destination);
    }

    command
}

fn tmux_session_name(name: &str) -> String {
    let sanitized: String = name
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_') {
                ch
            } else {
                '_'
            }
        })
        .collect();

    if sanitized.is_empty() {
        String::from("sshmgr-session")
    } else {
        format!("sshmgr-{}", sanitized)
    }
}

fn remove_host(path: &PathBuf, name: String) -> Result<(), String> {
    let mut data = load_hosts(path)?;
    let before = data.hosts.len();
    data.hosts.retain(|h| h.name != name);

    if data.hosts.len() == before {
        return Err(format!("Host '{}' not found", name));
    }

    save_hosts(path, &data)
}

fn run() -> Result<(), String> {
    let cli = Cli::parse();
    let path = config_path()?;

    match cli.command {
        Commands::Add {
            name,
            user,
            host,
            port,
        } => {
            add_host(&path, name, user, host, port)?;
            println!("Host added.");
            Ok(())
        }
        Commands::List => list_hosts(&path),
        Commands::Connect { name, plain } => connect_host_with_mode(&path, name, plain),
        Commands::Remove { name } => {
            remove_host(&path, name)?;
            println!("Host removed.");
            Ok(())
        }
    }
}

fn main() {
    if let Err(message) = run() {
        let _ = writeln_err(&message);
        std::process::exit(1);
    }
}

fn writeln_err(message: &str) -> io::Result<()> {
    use std::io::Write;
    let mut stderr = io::stderr().lock();
    writeln!(stderr, "Error: {}", message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsStr;

    #[test]
    fn tmux_session_name_sanitizes_invalid_characters() {
        assert_eq!(tmux_session_name("web root/1"), "sshmgr-web_root_1");
    }

    #[test]
    fn tmux_session_name_falls_back_for_empty_names() {
        assert_eq!(tmux_session_name(""), "sshmgr-session");
    }

    #[test]
    fn build_connect_command_uses_tmux_by_default() {
        let host = Host {
            name: String::from("webroot"),
            user: String::from("alice"),
            host: String::from("192.168.1.10"),
            port: 22,
        };

        let command = build_connect_command(&host, false);
        let args: Vec<String> = command
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect();

        assert_eq!(command.get_program(), OsStr::new("tmux"));
        assert_eq!(
            args,
            vec![
                String::from("new-session"),
                String::from("-A"),
                String::from("-s"),
                String::from("sshmgr-webroot"),
                String::from("ssh"),
                String::from("-p"),
                String::from("22"),
                String::from("alice@192.168.1.10"),
            ]
        );
    }

    #[test]
    fn build_connect_command_supports_plain_mode() {
        let host = Host {
            name: String::from("webroot"),
            user: String::from("alice"),
            host: String::from("192.168.1.10"),
            port: 22,
        };

        let command = build_connect_command(&host, true);
        let args: Vec<String> = command
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect();

        assert_eq!(command.get_program(), OsStr::new("ssh"));
        assert_eq!(
            args,
            vec![
                String::from("-p"),
                String::from("22"),
                String::from("alice@192.168.1.10"),
            ]
        );
    }
}
