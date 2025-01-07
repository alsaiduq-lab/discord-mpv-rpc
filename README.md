# MPV Discord Rich Presence

A Discord Rich Presence client for MPV media player that shows your currently playing media with timestamps, progress, and media information.

## Features

- Automatic media detection (Video/Music)
- Real-time progress updates
- Pause/play status
- Cross-platform support (Linux, macOS, Windows)
- Automatic reconnection handling
- Easy integration with MPV
- Customizable Discord assets

## Installation

### Linux
```bash
# Clone the repository
git clone https://github.com/alsaiduq-lab/mpv-discord-rpc
cd mpv-discord-rpc

# Run the installer
chmod +x install_linux.sh
./install_linux.sh
```

### macOS
```bash
# Clone the repository
git clone https://github.com/alsaiduq-lab/mpv-discord-rpc
cd mpv-discord-rpc

# Run the installer
chmod +x install_mac.sh
./install_mac.sh
```

### Windows
1. Clone the repository
2. Right-click `install_windows.ps1` and select "Run with PowerShell"

## Configuration

The configuration file is automatically created at:
- Linux/macOS: `~/.config/discord_mpv_rpc/config.toml`
- Windows: `%APPDATA%\discord_mpv_rpc\config.toml`

```toml
# MPV socket path
socket = "/tmp/mpvsocket"  # Linux/macOS
# socket = "\\\\.\\pipe\\mpvsocket"  # Windows

# Discord Application Client ID
client_id = "1322011605432533082"

# Discord rich presence image keys
large_image = "mpv_large"
small_image = "mpv_small"
```

## Usage

Once installed, the Rich Presence client will automatically start whenever you play media in MPV. You can:
- Press 'D' to toggle the Discord presence on/off
- The presence will automatically update when:
  - Playing/pausing media
  - Changing media files
  - Seeking through media

## Troubleshooting

### Common Issues
1. "Could not connect to MPV"
   - Ensure MPV is running
   - Check if the socket path in config.toml matches your system

2. "Discord connection failed"
   - Make sure Discord is running
   - Verify your client_id is correct

3. No images showing
   - Ensure you're using the correct image keys that match your Discord application's assets

### Debug Mode
Add `-v` flag when running manually for verbose output:
```bash
discord_mpv_rpc -v
```

## Building from Source

Requirements:
- Rust (1.70.0 or higher)
- Cargo
- A C compiler (gcc/clang)

```bash
cargo build --release
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.
