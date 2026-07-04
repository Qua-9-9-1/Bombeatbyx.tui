use crate::local::app::App;

impl App {
    pub fn start_local_server(&mut self) -> Result<(), String> {
        if self.server_process.is_some() {
            return Ok(());
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
            let args = if cfg!(target_os = "windows") {
                vec!["/C", "cargo run --bin server"]
            } else {
                vec!["-c", "cargo run --bin server"]
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
