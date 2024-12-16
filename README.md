# An SSH file  browser

SSH File Browser is a cross-platform desktop application built using `egui`. It provides a graphical interface to manage files on remote servers via SSH and SFTP. This application allows users to perform various file operations with ease.

**Note**: This project is still in early development and requires further work to enhance features and improve stability. 





## Features

### Connection Management
- **Connect to Remote Server**: Enter hostname, username, password, and port to establish an SSH connection.
- **Saved Connections**: Save frequently used connections for quick access and reuse.
- **Dark/Light Mode**: Toggle between dark and light themes to suit your preferences.

### File Operations
- **File and Directory Listing**: View all files and directories on the remote server.
- **Navigate Paths**: Move between directories using a simple navigation bar or buttons (`Home` and `Up`).
- **Upload Files**: Select a file from your local machine and upload it to the remote server.
- **Download Files**: Download files from the remote server to your local machine.
- **Delete Files**: Remove files directly from the remote server.
- **Modify Files**: Open and edit text files directly within the application and save changes back to the server.



## Contributing


Contributions are welcome! Feel free to open issues or submit pull requests.

## License

This project is licensed under the MIT License.


### Installation
1. Clone this repository:
   ```bash
   git clone https://github.com/0xb-s/ssh-browser
   cd ssh-browser
      ```
2. Build and run the project:

    ```bash
   cargo run --release
 
     ```
