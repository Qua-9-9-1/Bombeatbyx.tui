use crate::local::app::App;

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

        let paths = [
            "target/debug/server.exe",
            "../target/debug/server.exe",
            "target/release/server.exe",
            "../target/release/server.exe",
        ];

        let mut spawned = None;
        for path in &paths {
            if std::path::Path::new(path).exists() {
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
            std::thread::sleep(std::time::Duration::from_millis(150));
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
