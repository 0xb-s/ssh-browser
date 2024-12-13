use crate::ssh::SSHConnection;
use crate::ssh::SSHConnectionS;
use eframe::egui;
use egui::Ui;
use std::fs::File;
use std::io::Read;
use std::path::Path;
pub struct UIState {
    pub hostname: String,
    pub username: String,
    pub password: String,
    pub port: u16,
    pub connected: bool,
    pub current_path: String,
    pub files: Vec<(String, bool)>,
    pub error_message: Option<String>,
    pub dark_mode: bool,
    pub saved_connections: Vec<SSHConnectionData>,
    pub editing_file: Option<String>,
    pub file_content: String,
}

impl Default for UIState {
    fn default() -> Self {
        Self {
            hostname: String::new(),
            username: String::new(),
            password: String::new(),
            port: 22,
            connected: false,
            current_path: "/".to_string(),
            files: Vec::new(),
            error_message: None,
            dark_mode: true,
            saved_connections: load_saved_connections(),
            editing_file: None, 
            file_content: String::new(),
        }
    }
}
#[derive(PartialEq, Eq)]

enum FileOrType {
    Type,
    File,
}

pub fn render_ui(ui: &mut egui::Ui, state: &mut UIState, connection: &mut Option<SSHConnection>) {
    // Apply the selected theme
    let ctx = ui.ctx();
    apply_theme(ctx, state.dark_mode);

    // Theme Toggle
    ui.horizontal(|ui| {
        ui.label("Theme:");
        if ui
            .button(if state.dark_mode {
                "Switch to Light Mode"
            } else {
                "Switch to Dark Mode"
            })
            .clicked()
        {
            state.dark_mode = !state.dark_mode;
        }
    });

    if !state.connected {
     
        ui.heading("Connect to SSH Server");

        ui.horizontal(|ui| {
            ui.label("Saved Connections:");
            if !state.saved_connections.is_empty() {
                egui::ComboBox::from_label("Select")
                    .selected_text("Choose a connection")
                    .show_ui(ui, |ui| {
                        for saved_conn in &state.saved_connections {
                            if ui
                                .button(format!(
                                    "{}@{}:{}",
                                    saved_conn.username, saved_conn.hostname, saved_conn.port
                                ))
                                .clicked()
                            {
                                state.hostname = saved_conn.hostname.clone();
                                state.username = saved_conn.username.clone();
                                state.port = saved_conn.port;
                            }
                        }
                    });
            } else {
                ui.label("No saved connections.");
            }
        });

        ui.horizontal(|ui| {
            ui.label("Hostname:");
            ui.text_edit_singleline(&mut state.hostname);
        });
        ui.horizontal(|ui| {
            ui.label("Username:");
            ui.text_edit_singleline(&mut state.username);
        });
        ui.horizontal(|ui| {
            ui.label("Password:");
            ui.text_edit_singleline(&mut state.password);
        });
        ui.horizontal(|ui| {
            ui.label("Port:");
            ui.add(egui::DragValue::new(&mut state.port).clamp_range(1..=65535));
        });

        if ui.button("Save Current Connection").clicked() {
            let new_conn = SSHConnectionData {
                hostname: state.hostname.clone(),
                username: state.username.clone(),
                port: state.port,
            };
            if !state.saved_connections.contains(&new_conn) {
                state.saved_connections.push(new_conn);
                save_connections(&state.saved_connections);
            }
        }

        // Connect Button
        if ui.button("Connect").clicked() {
            let mut ssh_conn = SSHConnection::new(
                &state.hostname,
                &state.username,
                &state.password,
                state.port,
            );
            match ssh_conn.connect() {
                Ok(()) => {
                    state.connected = true;
                    state.current_path = "/".to_string();
                    match ssh_conn.list_directory(&state.current_path) {
                        Ok(files) => state.files = files,
                        Err(e) => state.error_message = Some(e),
                    }
                    *connection = Some(ssh_conn);
                }
                Err(e) => state.error_message = Some(format!("Failed to connect: {}", e)),
            }
        }

        if let Some(error) = &state.error_message {
            ui.colored_label(egui::Color32::RED, error);
        }
    } else {
        // File Manager UI
        ui.heading("SSH File Manager");

 
        ui.horizontal(|ui| {
            ui.label("Current Path:");
            if ui
                .text_edit_singleline(&mut state.current_path)
                .lost_focus()
                && ui.input(|state| state.key_pressed(egui::Key::Enter))
            {
                if let Some(conn) = connection {
                    match conn.list_directory(&state.current_path) {
                        Ok(files) => state.files = files,
                        Err(e) => state.error_message = Some(e),
                    }
                }
            }
        });

    
        ui.horizontal(|ui| {
            if ui.button("Up").clicked() {
                if let Some(pos) = state.current_path.rfind('/') {
                    state.current_path.truncate(pos);
                    if state.current_path.is_empty() {
                        state.current_path = "/".to_string();
                    }
                    if let Some(conn) = connection {
                        match conn.list_directory(&state.current_path) {
                            Ok(files) => state.files = files,
                            Err(e) => state.error_message = Some(e),
                        }
                    }
                }
            }
            if ui.button("Home").clicked() {
                state.current_path = "/".to_string();
                if let Some(conn) = connection {
                    match conn.list_directory(&state.current_path) {
                        Ok(files) => state.files = files,
                        Err(e) => state.error_message = Some(e),
                    }
                }
            }
            if ui.button("Disconnect").clicked() {
                state.connected = false;
                if let Some(mut conn) = connection.take() {
                    conn.disconnect();
                }
                state.files.clear();
                state.current_path = "/".to_string();
            }
        });

     
        egui::ScrollArea::vertical().show(ui, |ui| {
            for (name, is_dir) in state.files.clone() {
                ui.horizontal(|ui| {
                    if is_dir {
                        if ui.button(format!("[DIR] {}", name)).clicked() {
                            state.current_path =
                                format!("{}/{}", state.current_path.trim_end_matches('/'), name);
                            if let Some(conn) = connection {
                                match conn.list_directory(&state.current_path) {
                                    Ok(files) => state.files = files,
                                    Err(e) => state.error_message = Some(e),
                                }
                            }
                        }
                    } else {
                        ui.label(name.clone());
                        if ui.button("Download").clicked() {
                            if let Some(conn) = connection {
                                if let Some(local_path) = rfd::FileDialog::new()
                                    .set_file_name(name.clone())
                                    .save_file()
                                {
                                    match conn.download_file(
                                        &format!("{}/{}", state.current_path, name),
                                        local_path.to_str().unwrap(),
                                    ) {
                                        Ok(_) => {
                                            state.error_message =
                                                Some("Download successful".to_string())
                                        }
                                        Err(e) => {
                                            state.error_message =
                                                Some(format!("Failed to download: {}", e))
                                        }
                                    };
                                }
                            }
                        }
                        if ui.button("Delete").clicked() {
                            if let Some(conn) = connection {
                                let remote_path = format!("{}/{}", state.current_path, name);
                                match conn.delete_file(&remote_path) {
                                    Ok(_) => {
                                        state.error_message =
                                            Some("File deleted successfully.".to_string());
                                        // Refresh the directory
                                        match conn.list_directory(&state.current_path) {
                                            Ok(files) => state.files = files,
                                            Err(e) => state.error_message = Some(e),
                                        }
                                    }
                                    Err(e) => {
                                        state.error_message =
                                            Some(format!("Failed to delete: {}", e))
                                    }
                                }
                            }
                        }

                        // Add the Modify button here
                        if ui.button("Modify").clicked() {
                            if let Some(conn) = connection {
                                let remote_path = format!("{}/{}", state.current_path, name);
                                match conn.read_file(&remote_path) {
                                    Ok(content) => {
                                        state.editing_file = Some(remote_path);
                                        state.file_content = content;
                                    }
                                    Err(e) => {
                                        state.error_message =
                                            Some(format!("Failed to read file: {}", e))
                                    }
                                }
                            }
                        }
                    }
                });
            }
        });
        // Editor 
        if let Some(editing_file) = &state.editing_file {
            let editing_file_clone = editing_file.clone();
            egui::Window::new("Edit File")
                .resizable(true)
                .collapsible(false)
                .show(ui.ctx(), |ui| {
                    ui.label(format!("Editing: {}", editing_file_clone));
                    ui.text_edit_multiline(&mut state.file_content);

                    ui.horizontal(|ui| {
                        if ui.button("Save").clicked() {
                            if let Some(conn) = connection {
                                match conn.write_file(&editing_file_clone, &state.file_content) {
                                    Ok(_) => {
                                        state.error_message =
                                            Some("File saved successfully.".to_string());
                                        state.editing_file = None; 
                                    }
                                    Err(e) => {
                                        state.error_message =
                                            Some(format!("Failed to save file: {}", e))
                                    }
                                }
                            }
                        }
                        if ui.button("Cancel").clicked() {
                            state.editing_file = None; // Cancel the edit
                        }
                    });
                });
        }

        // Upload File Button
        if ui.button("Upload File").clicked() {
            if let Some(conn) = connection {
                if let Some(local_path) = rfd::FileDialog::new().pick_file() {
                    let remote_path = format!(
                        "{}/{}",
                        state.current_path,
                        local_path.file_name().unwrap().to_str().unwrap()
                    );
                    match conn.upload_file(local_path.to_str().unwrap(), &remote_path) {
                        Ok(_) => state.error_message = Some("Upload successful".to_string()),
                        Err(e) => state.error_message = Some(format!("Failed to upload: {}", e)),
                    };
                };
            }
        }

        if let Some(error) = &state.error_message {
            ui.colored_label(egui::Color32::RED, error);
        }
    }
}


fn apply_theme(ctx: &egui::Context, dark_mode: bool) {
    let mut style = (*ctx.style()).clone();

    if dark_mode {
        style.visuals = egui::Visuals::dark();
    } else {
        style.visuals = egui::Visuals::light();
    }

    ctx.set_style(style);
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SSHConnectionData {
    pub hostname: String,
    pub username: String,
    pub port: u16,
}
use serde::Deserialize;
use serde::Serialize;

const CONNECTIONS_FILE: &str = "saved_connections.json";

fn load_saved_connections() -> Vec<SSHConnectionData> {
    if Path::new(CONNECTIONS_FILE).exists() {
        let content = std::fs::read_to_string(CONNECTIONS_FILE).unwrap_or_default();
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        Vec::new()
    }
}

fn save_connections(connections: &Vec<SSHConnectionData>) {
    let content = serde_json::to_string(connections).unwrap();
    std::fs::write(CONNECTIONS_FILE, content).unwrap();
}
