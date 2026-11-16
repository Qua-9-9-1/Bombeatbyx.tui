use crate::local::app::App;
use std::path::PathBuf;
use std::time::{Duration, Instant};

const SERVER_STARTUP_TIMEOUT: Duration = Duration::from_secs(5);
const SERVER_STARTUP_POLL: Duration = Duration::from_millis(50);

/// Places where the server binary may live, most likely first: next to the
/// client executable (release zip), then the cargo target directories.
fn server_binary_candidates() -> Vec<PathBuf> {
    let file_name = format!("server{}", std::env::consts::EXE_SUFFIX);
    let mut candidates = Vec::new();
    if let Some(dir) = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(PathBuf::from))
    {
        candidates.push(dir.join(&file_name));
    }
    for profile_dir in ["target/release", "../target/release", "target/debug", "../target/debug"] {
        candidates.push(PathBuf::from(profile_dir).join(&file_name));
    }
    candidates
}

/// Blocks until the freshly spawned server accepts TCP connections (or gives up).
fn wait_until_listening(port: u16) {
    let started = Instant::now();
    while started.elapsed() < SERVER_STARTUP_TIMEOUT {
        if std::net::TcpStream::connect(("127.0.0.1", port)).is_ok() {
            return;
        }
        std::thread::sleep(SERVER_STARTUP_POLL);
    }
}

impl App {
    pub fn start_local_server(&mut self) -> Result<(), String> {
        if self.server_process.is_some() {
            return Ok(());
        }

        let mut port = 27300;
        for p in 27300..=27310 {
            if std::net::TcpListener::bind(format!("127.0.0.1:{}", p)).is_ok() {
                port = p;
                break;
            }
        }

        let mut spawned = None;
        for path in &server_binary_candidates() {
            if path.exists() {
                if let Ok(child) = std::process::Command::new(path)
                    .arg(port.to_string())
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .spawn()
                {
                    spawned = Some(child);
                    break;
                }
            }
        }

        if spawned.is_none() {
            let shell = if cfg!(target_os = "windows") {
                "cmd"
            } else {
                "sh"
            };
            let run_cmd = format!("cargo run --bin server -- {}", port);
            let args = if cfg!(target_os = "windows") {
                vec!["/C", &run_cmd]
            } else {
                vec!["-c", &run_cmd]
            };

            if let Ok(child) = std::process::Command::new(shell)
                .args(&args)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()
            {
                spawned = Some(child);
            }
        }

        if let Some(child) = spawned {
            self.server_process = Some(child);
            self.network.local_server_port = Some(port);
            wait_until_listening(port);
            Ok(())
        } else {
            Err("Failed to start server binary".to_string())
        }
    }

    pub fn stop_local_server(&mut self) {
        if let Some(mut child) = self.server_process.take() {
            let _ = child.kill();
        }
    }
}
