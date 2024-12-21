use ssh2::{OpenFlags, OpenType, Session, Sftp};
use std::{
    io::{Read, Write},
    net::TcpStream,
    path::Path,
};

/// Manages SSH and SFTP connections.
pub struct SSHConnection {
    hostname: String,
    username: String,
    password: String,
    port: u16,
    session: Option<Session>,
    sftp: Option<Sftp>,
}

#[derive(Debug, Clone)]
pub struct ServerStats {
    pub cpu_usage: String,
    pub memory_usage: String,
    pub disk_usage: String,
}
impl SSHConnection {
    pub fn new(hostname: &str, username: &str, password: &str, port: u16) -> Self {
        Self {
            hostname: hostname.to_string(),
            username: username.to_string(),
            password: password.to_string(),
            port,
            session: None,
            sftp: None,
        }
    }

    pub fn connect(&mut self) -> Result<(), String> {
        let addr = format!("{}:{}", self.hostname, self.port);
        let tcp = TcpStream::connect(addr).map_err(|e| format!("Connection error: {}", e))?;
        let mut session = Session::new().map_err(|e| format!("Session creation error: {}", e))?;
        session.set_tcp_stream(tcp);
        session
            .handshake()
            .map_err(|e| format!("Handshake error: {}", e))?;
        session
            .userauth_password(&self.username, &self.password)
            .map_err(|e| format!("Authentication error: {}", e))?;

        if !session.authenticated() {
            return Err("Authentication failed. Check your username and password.".to_string());
        }

        let sftp = session
            .sftp()
            .map_err(|e| format!("SFTP initialization error: {}", e))?;
        self.session = Some(session);
        self.sftp = Some(sftp);

        Ok(())
    }

    pub fn disconnect(&mut self) {
        self.sftp = None;
        self.session = None;
    }

    pub fn delete_file(&self, remote_path: &str) -> Result<(), String> {
        if let Some(sftp) = &self.sftp {
            sftp.unlink(Path::new(remote_path))
                .map_err(|e| format!("Failed to delete file: {}", e))
        } else {
            Err("SFTP subsystem not initialized.".to_string())
        }
    }

    pub fn list_directory(&self, path: &str) -> Result<Vec<(String, bool)>, String> {
        let sftp = self
            .sftp
            .as_ref()
            .ok_or_else(|| "SFTP subsystem not initialized.".to_string())?;

        let entries = sftp
            .readdir(Path::new(path))
            .map_err(|e| format!("Failed to read directory: {}", e))?;

        let mut result = Vec::new();
        for (entry_path, stat) in entries {
            if let Some(name) = entry_path.file_name() {
                let name_str = name.to_string_lossy().to_string();
                result.push((name_str, stat.is_dir()));
            }
        }

        result.sort_by(|a, b| {
            if a.1 && !b.1 {
                std::cmp::Ordering::Less
            } else if !a.1 && b.1 {
                std::cmp::Ordering::Greater
            } else {
                a.0.cmp(&b.0)
            }
        });

        Ok(result)
    }

    pub fn read_file(&self, remote_path: &str) -> Result<String, String> {
        if let Some(sftp) = &self.sftp {
            let mut file = sftp
                .open(Path::new(remote_path))
                .map_err(|e| format!("Failed to open file: {}", e))?;
            let mut content = String::new();
            file.read_to_string(&mut content)
                .map_err(|e| format!("Failed to read file: {}", e))?;
            Ok(content)
        } else {
            Err("SFTP subsystem not initialized.".to_string())
        }
    }

    pub fn write_file(&self, remote_path: &str, content: &str) -> Result<(), String> {
        if let Some(sftp) = &self.sftp {
            let mut file = sftp
                .create(Path::new(remote_path))
                .map_err(|e| format!("Failed to create file: {}", e))?;
            file.write_all(content.as_bytes())
                .map_err(|e| format!("Failed to write file: {}", e))?;
            Ok(())
        } else {
            Err("SFTP subsystem not initialized.".to_string())
        }
    }

    pub fn download_file(&self, remote_path: &str, local_path: &str) -> Result<(), String> {
        let sftp = self
            .sftp
            .as_ref()
            .ok_or_else(|| "SFTP subsystem not initialized.".to_string())?;
        let mut remote_file = sftp
            .open(Path::new(remote_path))
            .map_err(|e| format!("Failed to open remote file: {}", e))?;
        let mut local_file = std::fs::File::create(local_path)
            .map_err(|e| format!("Failed to create local file: {}", e))?;

        let mut buffer = [0; 8192];
        loop {
            let bytes_read = remote_file
                .read(&mut buffer)
                .map_err(|e| format!("Error reading from remote file: {}", e))?;
            if bytes_read == 0 {
                break;
            }
            local_file
                .write_all(&buffer[..bytes_read])
                .map_err(|e| format!("Error writing to local file: {}", e))?;
        }
        Ok(())
    }

    pub fn upload_file(&self, local_path: &str, remote_path: &str) -> Result<(), String> {
        let sftp = self
            .sftp
            .as_ref()
            .ok_or_else(|| "SFTP subsystem not initialized.".to_string())?;
        let mut local_file = std::fs::File::open(local_path)
            .map_err(|e| format!("Failed to open local file: {}", e))?;
        let mut remote_file = sftp
            .open_mode(
                Path::new(remote_path),
                OpenFlags::WRITE | OpenFlags::CREATE | OpenFlags::TRUNCATE,
                0o644,
                OpenType::File,
            )
            .map_err(|e| format!("Failed to open remote file: {}", e))?;

        let mut buffer = [0; 8192];
        loop {
            let bytes_read = local_file
                .read(&mut buffer)
                .map_err(|e| format!("Error reading from local file: {}", e))?;
            if bytes_read == 0 {
                break;
            }
            remote_file
                .write_all(&buffer[..bytes_read])
                .map_err(|e| format!("Error writing to remote file: {}", e))?;
        }
        Ok(())
    }

    pub fn rename(&self, old_path: &str, new_path: &str) -> Result<(), String> {
        if let Some(sftp) = &self.sftp {
            let old_path = Path::new(old_path);
            let new_path = Path::new(new_path);

            sftp.rename(old_path, new_path, None)
                .map_err(|e| format!("Failed to rename: {}", e))
        } else {
            Err("SFTP session not initialized.".to_string())
        }
    }

    pub fn create_directory(&self, path: &str) -> Result<(), String> {
        if let Some(sftp) = &self.sftp {
            sftp.mkdir(Path::new(path), 0o755)
                .map_err(|e| format!("Failed to create directory: {}", e))
        } else {
            Err("SFTP subsystem not initialized.".to_string())
        }
    }

    pub fn create_file(&self, path: &str) -> Result<(), String> {
        if let Some(sftp) = &self.sftp {
            let mut file = sftp
                .create(Path::new(path))
                .map_err(|e| format!("Failed to create file: {}", e))?;
            file.write_all(b"")
                .map_err(|e| format!("Failed to initialize file: {}", e))?;
            Ok(())
        } else {
            Err("SFTP subsystem not initialized.".to_string())
        }
    }

    fn run_command(session: &Session, cmd: &str) -> Result<String, String> {
        let mut channel = session
            .channel_session()
            .map_err(|e| format!("Failed to open channel: {}", e))?;
        channel
            .exec(cmd)
            .map_err(|e| format!("Failed to exec command {}: {}", cmd, e))?;

        let mut stdout = String::new();
        channel
            .read_to_string(&mut stdout)
            .map_err(|e| format!("Failed to read command output: {}", e))?;

        channel
            .wait_close()
            .map_err(|e| format!("Failed to close channel: {}", e))?;

        Ok(stdout)
    }

    pub fn fetch_stats(&self) -> Result<ServerStats, String> {
        let session = self
            .session
            .as_ref()
            .ok_or_else(|| "Session not initialized.".to_string())?;

        let cpu_cmd = r#"top -bn1 | grep "Cpu(s)""#;
        let mem_cmd = r#"free -h | grep "Mem:""#;
        let disk_cmd = r#"df -h / | tail -1"#;

        let raw_cpu = Self::run_command(session, cpu_cmd)?;
        let raw_mem = Self::run_command(session, mem_cmd)?;
        let raw_disk = Self::run_command(session, disk_cmd)?;

        Ok(Self::process_stats(&raw_cpu, &raw_mem, &raw_disk))
    }

    fn process_stats(raw_cpu: &str, raw_mem: &str, raw_disk: &str) -> ServerStats {
        let cpu_parts: Vec<&str> = raw_cpu.split_whitespace().collect();
        let cpu_usage = format!(
            "User: {}%, System: {}%, Idle: {}%, Steal: {}%",
            cpu_parts[1], cpu_parts[3], cpu_parts[7], cpu_parts[15]
        );

        let mem_parts: Vec<&str> = raw_mem.split_whitespace().collect();
        let memory_usage = format!(
            "Total: {}, Used: {}, Free: {}, Buffers/Cache: {}",
            mem_parts[1], mem_parts[2], mem_parts[3], mem_parts[5]
        );

        let disk_parts: Vec<&str> = raw_disk.split_whitespace().collect();
        let disk_usage = format!(
            "Filesystem: {}, Total: {}, Used: {}, Available: {}, Usage: {}",
            disk_parts[0], disk_parts[1], disk_parts[2], disk_parts[3], disk_parts[4]
        );

        ServerStats {
            cpu_usage,
            memory_usage,
            disk_usage,
        }
    }
}
