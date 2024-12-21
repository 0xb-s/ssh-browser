use crate::{
    localization::{Language, Localizer},
    ssh::SSHConnection,
};
use eframe::egui;
use serde::{Deserialize, Serialize};
use std::{
    path::Path,
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
};

/// The file where connections are stored
const CONNECTIONS_FILE: &str = "saved_connections.json";

/// Represents a saved SSH connection configuration
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SSHConnectionData {
    /// The hostname/IP address of the SSH server
    pub hostname: String,
    /// The username to authenticate with
    pub username: String,
    /// The port number of the SSH server
    pub port: u16,
}

/// Load saved SSH connections from a JSON file
fn load_saved_connections() -> Vec<SSHConnectionData> {
    if Path::new(CONNECTIONS_FILE).exists() {
        let content = std::fs::read_to_string(CONNECTIONS_FILE).unwrap_or_default();
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        Vec::new()
    }
}

/// Save SSH connections to a JSON file
fn save_connections(connections: &Vec<SSHConnectionData>) {
    let content = serde_json::to_string(connections).unwrap();
    std::fs::write(CONNECTIONS_FILE, content).unwrap();
}

/// Represents tasks that can be performed on the SSH connection.
enum Task {
    /// Connect to the SSH server (hostname, username, password, port)
    Connect(String, String, String, u16),
    /// List the directory contents of the given path
    ListDirectory(String),
    /// Create a directory at the specified path
    CreateDirectory(String),
    /// Create an empty file at the specified path
    CreateFile(String),
    /// Download a file from remote to local
    DownloadFile(String, String),
    /// Upload a file from local to remote
    UploadFile(String, String),
    /// Delete a file
    DeleteFile(String),
    /// Rename a file (old_path, new_path)
    RenameFile(String, String),
    /// Read a file from the remote server
    ReadFile(String),
    /// Write file content to the remote server
    WriteFile(String, String),
    /// Disconnect the active connection
    Disconnect,
}

/// Represents the result of executing a Task.
/// The UI thread will receive these results and update the UI state accordingly.
#[allow(clippy::enum_variant_names)]
enum TaskResult {
    /// The result of the connect attempt
    ConnectResult(Result<(), String>),
    /// The result of listing a directory (Vec<(filename, is_dir)> or error)
    ListDirectoryResult(Result<Vec<(String, bool)>, String>),
    /// Generic success message for directory creation
    CreateDirectoryResult(Result<(), String>),
    /// Generic success message for file creation
    CreateFileResult(Result<(), String>),
    /// Generic success message for file download
    DownloadFileResult(Result<(), String>),
    /// Generic success message for file upload
    UploadFileResult(Result<(), String>),
    /// Generic success message for file deletion
    DeleteFileResult(Result<(), String>),
    /// Generic success message for file renaming
    RenameFileResult(Result<(), String>),
    /// The result of reading a file
    ReadFileResult(Result<String, String>),
    /// The result of writing a file
    WriteFileResult(Result<(), String>),
    /// The result of disconnecting
    DisconnectResult,
}

/// BackgroundWorker handles asynchronous tasks to avoid blocking the UI.
/// Communicates with the UI via channels.
struct BackgroundWorker {
    /// Sender to send tasks from the UI thread to the worker thread
    task_sender: Sender<Task>,
    /// Receiver on the UI side to receive the results from the worker thread
    result_receiver: Receiver<TaskResult>,
    /// Holds the active SSH connection if connected
    #[allow(dead_code)]
    connection: Option<SSHConnection>,
}

impl BackgroundWorker {
    /// Create a new BackgroundWorker and start the worker thread
    fn new() -> Self {
        let (task_sender, task_receiver) = mpsc::channel();
        let (result_sender, result_receiver) = mpsc::channel();

        // Spawn the worker thread
        thread::spawn(move || {
            let mut connection: Option<SSHConnection> = None;
            while let Ok(task) = task_receiver.recv() {
                match task {
                    Task::Connect(hostname, username, password, port) => {
                        let mut conn = SSHConnection::new(&hostname, &username, &password, port);
                        let connect_result = conn.connect();

                        let send_result = match connect_result {
                            Ok(_) => {
                                connection = Some(conn);
                                Ok(())
                            }
                            Err(e) => Err(format!("Failed to connect: {}", e)),
                        };

                        let _ = result_sender.send(TaskResult::ConnectResult(send_result));
                    }

                    Task::ListDirectory(path) => {
                        if let Some(conn) = connection.as_ref() {
                            let result = conn.list_directory(&path);
                            let _ = result_sender.send(TaskResult::ListDirectoryResult(result));
                        } else {
                            let _ = result_sender
                                .send(TaskResult::ListDirectoryResult(Err("Not connected".into())));
                        }
                    }
                    Task::CreateDirectory(path) => {
                        if let Some(conn) = connection.as_ref() {
                            let result = conn
                                .create_directory(&path)
                                .map_err(|e| format!("Failed to create directory: {}", e));
                            let _ = result_sender.send(TaskResult::CreateDirectoryResult(result));
                        } else {
                            let _ = result_sender.send(TaskResult::CreateDirectoryResult(Err(
                                "Not connected".into(),
                            )));
                        }
                    }
                    Task::CreateFile(path) => {
                        if let Some(conn) = connection.as_ref() {
                            let result = conn
                                .create_file(&path)
                                .map_err(|e| format!("Failed to create file: {}", e));
                            let _ = result_sender.send(TaskResult::CreateFileResult(result));
                        } else {
                            let _ = result_sender
                                .send(TaskResult::CreateFileResult(Err("Not connected".into())));
                        }
                    }
                    Task::DownloadFile(remote, local) => {
                        if let Some(conn) = connection.as_ref() {
                            let result = conn
                                .download_file(&remote, &local)
                                .map_err(|e| format!("Failed to download: {}", e));
                            let _ = result_sender.send(TaskResult::DownloadFileResult(result));
                        } else {
                            let _ = result_sender
                                .send(TaskResult::DownloadFileResult(Err("Not connected".into())));
                        }
                    }
                    Task::UploadFile(local, remote) => {
                        if let Some(conn) = connection.as_ref() {
                            let result = conn
                                .upload_file(&local, &remote)
                                .map_err(|e| format!("Failed to upload: {}", e));
                            let _ = result_sender.send(TaskResult::UploadFileResult(result));
                        } else {
                            let _ = result_sender
                                .send(TaskResult::UploadFileResult(Err("Not connected".into())));
                        }
                    }
                    Task::DeleteFile(path) => {
                        if let Some(conn) = connection.as_ref() {
                            let result = conn
                                .delete_file(&path)
                                .map_err(|e| format!("Failed to delete: {}", e));
                            let _ = result_sender.send(TaskResult::DeleteFileResult(result));
                        } else {
                            let _ = result_sender
                                .send(TaskResult::DeleteFileResult(Err("Not connected".into())));
                        }
                    }
                    Task::RenameFile(old, new) => {
                        if let Some(conn) = connection.as_ref() {
                            let result = conn
                                .rename(&old, &new)
                                .map_err(|e| format!("Failed to rename: {}", e));
                            let _ = result_sender.send(TaskResult::RenameFileResult(result));
                        } else {
                            let _ = result_sender
                                .send(TaskResult::RenameFileResult(Err("Not connected".into())));
                        }
                    }
                    Task::ReadFile(path) => {
                        if let Some(conn) = connection.as_ref() {
                            let result = conn
                                .read_file(&path)
                                .map_err(|e| format!("Failed to read file: {}", e));
                            let _ = result_sender.send(TaskResult::ReadFileResult(result));
                        } else {
                            let _ = result_sender
                                .send(TaskResult::ReadFileResult(Err("Not connected".into())));
                        }
                    }
                    Task::WriteFile(path, content) => {
                        if let Some(conn) = connection.as_ref() {
                            let result = conn
                                .write_file(&path, &content)
                                .map_err(|e| format!("Failed to write file: {}", e));
                            let _ = result_sender.send(TaskResult::WriteFileResult(result));
                        } else {
                            let _ = result_sender
                                .send(TaskResult::WriteFileResult(Err("Not connected".into())));
                        }
                    }
                    Task::Disconnect => {
                        if let Some(mut conn) = connection.take() {
                            conn.disconnect();
                        }
                        let _ = result_sender.send(TaskResult::DisconnectResult);
                    }
                }
            }
        });

        Self {
            task_sender,
            result_receiver,
            connection: None,
        }
    }

    /// Send a task to the worker thread
    fn send_task(&self, task: Task) {
        let _ = self.task_sender.send(task);
    }
}

/// Represents the UI state
pub struct UIState {
    /// The SSH hostname
    pub hostname: String,
    /// The SSH username
    pub username: String,
    /// The SSH password
    pub password: String,
    /// The SSH port
    pub port: u16,
    /// Whether currently connected or not
    pub connected: bool,
    /// The current remote directory path
    pub current_path: String,
    /// List of files in the current directory
    pub files: Vec<(String, bool)>,
    /// Any error or status message to display
    pub error_message: Option<String>,
    /// Whether dark mode is enabled
    pub dark_mode: bool,
    /// A list of saved connections
    pub saved_connections: Vec<SSHConnectionData>,
    /// If we are editing a file, store its remote path
    pub editing_file: Option<String>,
    /// The content of the file currently being edited
    pub file_content: String,
    /// If we are renaming a file, store its name
    pub renaming_file: Option<String>,
    /// The new name for the file/directory being renamed
    pub new_name: String,
    /// The name for new directories
    pub new_directory_name: String,
    /// The name for new files
    pub new_file_name: String,
    /// The background worker to run tasks asynchronously
    worker: Arc<Mutex<BackgroundWorker>>,
    /// Shows if an operation is in progress to provide feedback to the user
    pub operation_in_progress: bool,

    /// The current chosen language
    pub language: Language,
    /// The localizer that holds translations
    pub localizer: Localizer,
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
            renaming_file: None,
            new_name: String::new(),
            new_directory_name: String::new(),
            new_file_name: String::new(),
            worker: Arc::new(Mutex::new(BackgroundWorker::new())),
            operation_in_progress: false,
            language: Language::English,

            localizer: Localizer::new(),
        }
    }
}

/// Render the UI and handle events
pub fn render_ui(ui: &mut egui::Ui, state: &mut UIState, _connection: &mut Option<SSHConnection>) {
    let ctx = ui.ctx();
    apply_theme(ctx, state.dark_mode);

    poll_worker(state);

    ui.horizontal(|ui| {
        ui.label(state.localizer.t(state.language, "theme_label"));

        if ui
            .button(if state.dark_mode {
                state.localizer.t(state.language, "switch_light_mode")
            } else {
                state.localizer.t(state.language, "switch_dark_mode")
            })
            .clicked()
        {
            state.dark_mode = !state.dark_mode;
        }

        ui.label("Language:");
        egui::ComboBox::from_label("")
            .selected_text(format!("{:?}", state.language))
            .show_ui(ui, |ui| {
                if ui.button("English").clicked() {
                    state.language = Language::English;
                }
                if ui.button("Arabic").clicked() {
                    state.language = Language::Arabic;
                }
                if ui.button("French").clicked() {
                    state.language = Language::French;
                }
                if ui.button("Chinese").clicked() {
                    state.language = Language::Chinese;
                }
            });
    });

    if state.operation_in_progress {
        ui.label(state.localizer.t(state.language, "operation_in_progress"));
    }

    if !state.connected {
        ui.heading(state.localizer.t(state.language, "connect_to_ssh"));

        ui.horizontal(|ui| {
            ui.label(state.localizer.t(state.language, "saved_connections"));
            if !state.saved_connections.is_empty() {
                egui::ComboBox::from_label(
                    state
                        .localizer
                        .t(state.language, "select_connection_combo_label"),
                )
                .selected_text(state.localizer.t(state.language, "choose_a_connection"))
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
                ui.label(state.localizer.t(state.language, "no_saved_connections"));
            }
        });

        ui.horizontal(|ui| {
            ui.label(state.localizer.t(state.language, "hostname_label"));
            ui.text_edit_singleline(&mut state.hostname);
        });

        ui.horizontal(|ui| {
            ui.label(state.localizer.t(state.language, "username_label"));
            ui.text_edit_singleline(&mut state.username);
        });

        ui.horizontal(|ui| {
            ui.label(state.localizer.t(state.language, "password_label"));
            ui.add(egui::TextEdit::singleline(&mut state.password).password(true));
        });

        ui.horizontal(|ui| {
            ui.label(state.localizer.t(state.language, "port_label"));
            ui.add(egui::DragValue::new(&mut state.port).range(1..=65535));
        });

        if ui
            .button(state.localizer.t(state.language, "save_current_connection"))
            .clicked()
        {
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

        if ui
            .button(state.localizer.t(state.language, "connect_button"))
            .clicked()
        {
            state.operation_in_progress = true;
            let worker = state.worker.clone();
            let hostname = state.hostname.clone();
            let username = state.username.clone();
            let password = state.password.clone();
            let port = state.port;
            worker
                .lock()
                .unwrap()
                .send_task(Task::Connect(hostname, username, password, port));
        }

        if let Some(error) = &state.error_message {
            ui.colored_label(egui::Color32::RED, error);
        }
    } else {
        ui.heading(state.localizer.t(state.language, "ssh_file_manager"));

        ui.horizontal(|ui| {
            ui.label(state.localizer.t(state.language, "current_path_label"));
            if ui
                .text_edit_singleline(&mut state.current_path)
                .lost_focus()
                && ui.input(|state| state.key_pressed(egui::Key::Enter))
            {
                state.operation_in_progress = true;
                let worker = state.worker.clone();
                let path = state.current_path.clone();
                worker.lock().unwrap().send_task(Task::ListDirectory(path));
            }
        });

        ui.horizontal(|ui| {
            ui.label(state.localizer.t(state.language, "create_directory_label"));
            ui.text_edit_singleline(&mut state.new_directory_name);
            if ui
                .button(state.localizer.t(state.language, "create_label"))
                .clicked()
            {
                if !state.new_directory_name.is_empty() {
                    let full_path = format!("{}/{}", state.current_path, state.new_directory_name);
                    state.operation_in_progress = true;
                    state.new_directory_name.clear();
                    let worker = state.worker.clone();
                    worker
                        .lock()
                        .unwrap()
                        .send_task(Task::CreateDirectory(full_path));
                } else {
                    state.error_message = Some(
                        state
                            .localizer
                            .t(state.language, "directory_name_empty_error")
                            .to_string(),
                    );
                }
            }
        });

        ui.horizontal(|ui| {
            ui.label(state.localizer.t(state.language, "create_file_label"));
            ui.text_edit_singleline(&mut state.new_file_name);
            if ui
                .button(state.localizer.t(state.language, "create_label"))
                .clicked()
            {
                if !state.new_file_name.is_empty() {
                    let full_path = format!("{}/{}", state.current_path, state.new_file_name);
                    state.operation_in_progress = true;
                    state.new_file_name.clear();
                    let worker = state.worker.clone();
                    worker
                        .lock()
                        .unwrap()
                        .send_task(Task::CreateFile(full_path));
                } else {
                    state.error_message = Some(
                        state
                            .localizer
                            .t(state.language, "file_name_empty_error")
                            .to_string(),
                    );
                }
            }
        });

        ui.horizontal(|ui| {
            if ui
                .button(state.localizer.t(state.language, "up_button"))
                .clicked()
            {
                if let Some(pos) = state.current_path.rfind('/') {
                    state.current_path.truncate(pos);
                    if state.current_path.is_empty() {
                        state.current_path = "/".to_string();
                    }
                    state.operation_in_progress = true;
                    let worker = state.worker.clone();
                    let path = state.current_path.clone();
                    worker.lock().unwrap().send_task(Task::ListDirectory(path));
                }
            }
            if ui
                .button(state.localizer.t(state.language, "home_button"))
                .clicked()
            {
                state.current_path = "/".to_string();
                state.operation_in_progress = true;
                let worker = state.worker.clone();
                let path = state.current_path.clone();
                worker.lock().unwrap().send_task(Task::ListDirectory(path));
            }
            if ui
                .button(state.localizer.t(state.language, "disconnect_button"))
                .clicked()
            {
                state.operation_in_progress = true;
                let worker = state.worker.clone();
                worker.lock().unwrap().send_task(Task::Disconnect);
            }
        });

        egui::ScrollArea::vertical().show(ui, |ui| {
            for (name, is_dir) in state.files.clone() {
                ui.horizontal(|ui| {
                    if let Some(renaming_file) = &state.renaming_file {
                        if renaming_file == &name {
                            ui.text_edit_singleline(&mut state.new_name);
                            if ui
                                .button(state.localizer.t(state.language, "save_button"))
                                .clicked()
                            {
                                let old_path = format!("{}/{}", state.current_path, name);
                                let new_path = format!("{}/{}", state.current_path, state.new_name);
                                state.operation_in_progress = true;
                                state.renaming_file = None;
                                state.new_name.clear();
                                let worker = state.worker.clone();
                                worker
                                    .lock()
                                    .unwrap()
                                    .send_task(Task::RenameFile(old_path, new_path));
                            }
                            if ui
                                .button(state.localizer.t(state.language, "cancel_button"))
                                .clicked()
                            {
                                state.renaming_file = None;
                                state.new_name.clear();
                            }
                        }
                    } else {
                        if is_dir {
                            if ui.button(format!("📁 {}", name)).clicked() {
                                state.current_path = format!(
                                    "{}/{}",
                                    state.current_path.trim_end_matches('/'),
                                    name
                                );
                                state.operation_in_progress = true;
                                let worker = state.worker.clone();
                                let path = state.current_path.clone();
                                worker.lock().unwrap().send_task(Task::ListDirectory(path));
                            }
                        } else {
                            ui.label(format!("📄 {}", name));
                        }

                        if !is_dir
                            && ui
                                .button(state.localizer.t(state.language, "download_button"))
                                .clicked()
                        {
                            if let Some(local_path) = rfd::FileDialog::new()
                                .set_file_name(name.clone())
                                .save_file()
                            {
                                let remote_path = format!("{}/{}", state.current_path, name);
                                let worker = state.worker.clone();
                                state.operation_in_progress = true;
                                worker.lock().unwrap().send_task(Task::DownloadFile(
                                    remote_path,
                                    local_path.to_str().unwrap().to_string(),
                                ));
                            }
                        }

                        if ui
                            .button(state.localizer.t(state.language, "delete_button"))
                            .clicked()
                        {
                            let remote_path = format!("{}/{}", state.current_path, name);
                            let worker = state.worker.clone();
                            state.operation_in_progress = true;
                            worker
                                .lock()
                                .unwrap()
                                .send_task(Task::DeleteFile(remote_path));
                        }

                        if !is_dir
                            && ui
                                .button(state.localizer.t(state.language, "modify_button"))
                                .clicked()
                        {
                            let remote_path = format!("{}/{}", state.current_path, name);
                            let worker = state.worker.clone();
                            state.operation_in_progress = true;
                            worker
                                .lock()
                                .unwrap()
                                .send_task(Task::ReadFile(remote_path));
                        }

                        if ui
                            .button(state.localizer.t(state.language, "rename_button"))
                            .clicked()
                        {
                            state.renaming_file = Some(name.clone());
                            state.new_name = name.clone();
                        }
                    }
                });
            }
        });

        if let Some(editing_file) = &state.editing_file {
            let editing_file_clone = editing_file.clone();
            egui::Window::new(state.localizer.t(state.language, "edit_file_window"))
                .resizable(true)
                .collapsible(false)
                .show(ui.ctx(), |ui| {
                    ui.label(format!(
                        "{} {}",
                        state.localizer.t(state.language, "editing_label"),
                        editing_file_clone
                    ));
                    ui.text_edit_multiline(&mut state.file_content);

                    ui.horizontal(|ui| {
                        if ui
                            .button(state.localizer.t(state.language, "save_button"))
                            .clicked()
                        {
                            let worker = state.worker.clone();
                            state.operation_in_progress = true;
                            let path = editing_file_clone.clone();
                            let content = state.file_content.clone();
                            worker
                                .lock()
                                .unwrap()
                                .send_task(Task::WriteFile(path, content));
                        }
                        if ui
                            .button(state.localizer.t(state.language, "cancel_button"))
                            .clicked()
                        {
                            state.editing_file = None;
                        }
                    });
                });
        }

        if ui
            .button(state.localizer.t(state.language, "upload_file_button"))
            .clicked()
        {
            if let Some(local_path) = rfd::FileDialog::new().pick_file() {
                let remote_path = format!(
                    "{}/{}",
                    state.current_path,
                    local_path.file_name().unwrap().to_str().unwrap()
                );
                let worker = state.worker.clone();
                state.operation_in_progress = true;
                worker.lock().unwrap().send_task(Task::UploadFile(
                    local_path.to_str().unwrap().to_string(),
                    remote_path,
                ));
            }
        }

        if let Some(error) = &state.error_message {
            ui.colored_label(egui::Color32::RED, error);
        }
    }
}

/// Apply the chosen theme (dark or light mode)
fn apply_theme(ctx: &egui::Context, dark_mode: bool) {
    let mut style = (*ctx.style()).clone();
    if dark_mode {
        style.visuals = egui::Visuals::dark();
    } else {
        style.visuals = egui::Visuals::light();
    }
    ctx.set_style(style);
}

/// Poll the background worker for results and update the UI state accordingly
fn poll_worker(state: &mut UIState) {
    let worker = state.worker.clone();
    let worker = worker.lock().unwrap();
    while let Ok(result) = worker.result_receiver.try_recv() {
        state.operation_in_progress = false;
        match result {
            TaskResult::ConnectResult(res) => {
                match res {
                    Ok(_) => {
                        state.connected = true;
                        state.current_path = "/".to_string();
                        // Once connected, immediately list the directory
                        state.operation_in_progress = true;
                        let path = state.current_path.clone();
                        worker.send_task(Task::ListDirectory(path));
                    }
                    Err(e) => {
                        state.error_message = Some(e);
                        state.connected = false;
                    }
                }
            }
            TaskResult::ListDirectoryResult(res) => match res {
                Ok(files) => {
                    state.files = files;
                    state.error_message = None;
                }
                Err(e) => {
                    state.error_message = Some(e);
                }
            },
            TaskResult::CreateDirectoryResult(res) => match res {
                Ok(_) => {
                    state.error_message = Some("Directory created successfully.".to_string());
                    state.operation_in_progress = true;
                    let path = state.current_path.clone();
                    worker.send_task(Task::ListDirectory(path));
                }
                Err(e) => {
                    state.error_message = Some(e);
                }
            },
            TaskResult::CreateFileResult(res) => match res {
                Ok(_) => {
                    state.error_message = Some("File created successfully.".to_string());
                    state.operation_in_progress = true;
                    let path = state.current_path.clone();
                    worker.send_task(Task::ListDirectory(path));
                }
                Err(e) => {
                    state.error_message = Some(e);
                }
            },
            TaskResult::DownloadFileResult(res) => match res {
                Ok(_) => state.error_message = Some("Download successful".to_string()),
                Err(e) => state.error_message = Some(e),
            },
            TaskResult::UploadFileResult(res) => match res {
                Ok(_) => {
                    state.error_message = Some("Upload successful".to_string());
                    state.operation_in_progress = true;
                    let path = state.current_path.clone();
                    worker.send_task(Task::ListDirectory(path));
                }
                Err(e) => state.error_message = Some(e),
            },
            TaskResult::DeleteFileResult(res) => match res {
                Ok(_) => {
                    state.error_message = Some("File deleted successfully.".to_string());
                    state.operation_in_progress = true;
                    let path = state.current_path.clone();
                    worker.send_task(Task::ListDirectory(path));
                }
                Err(e) => state.error_message = Some(e),
            },
            TaskResult::RenameFileResult(res) => match res {
                Ok(_) => {
                    state.error_message = Some("File renamed successfully.".to_string());
                    state.operation_in_progress = true;
                    let path = state.current_path.clone();
                    worker.send_task(Task::ListDirectory(path));
                }
                Err(e) => state.error_message = Some(e),
            },
            TaskResult::ReadFileResult(res) => match res {
                Ok(content) => {
                    state.file_content = content;
                    state.error_message = Some("File content loaded.".to_string());
                }
                Err(e) => {
                    state.error_message = Some(e);
                }
            },
            TaskResult::WriteFileResult(res) => match res {
                Ok(_) => {
                    state.error_message = Some("File saved successfully.".to_string());
                    state.editing_file = None;
                }
                Err(e) => {
                    state.error_message = Some(e);
                }
            },
            TaskResult::DisconnectResult => {
                state.connected = false;
                state.files.clear();
                state.current_path = "/".to_string();
                state.error_message = Some("Disconnected".to_string());
            }
        }
    }
}
