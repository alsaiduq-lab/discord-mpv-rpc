#!/bin/bash
set -e

echo "Installing discord_mpv_rpc..."

if ! command -v cargo &> /dev/null; then
    echo "Error: cargo is not installed. Please install Rust first."
    exit 1
fi

if ! cargo build --release; then
    echo "Build failed. Please check the error messages above."
    exit 1
fi

CONFIG_DIR="$HOME/Library/Application Support/discord_mpv_rpc"
if [[ ! -d "$HOME/Library/Application Support" ]]; then
    echo "Error: Application Support directory not found"
    exit 1
fi

MPV_CONFIG_DIR="$HOME/.config/mpv"
MPV_SCRIPTS_DIR="$HOME/.config/mpv/scripts"

for dir in "$CONFIG_DIR" "$MPV_CONFIG_DIR" "$MPV_SCRIPTS_DIR"; do
    if ! mkdir -p "$dir"; then
        echo "Error: Failed to create directory: $dir"
        exit 1
    fi
done

if ! cp target/release/discord_mpv_rpc /usr/local/bin/; then
    echo "Error: Failed to install binary. Please run with sudo."
    exit 1
fi

if ! cp discord-rpc.lua "$MPV_SCRIPTS_DIR/"; then
    echo "Error: Failed to install MPV script."
    exit 1
fi

CONFIG_FILE="$CONFIG_DIR/config.toml"
if [ ! -f "$CONFIG_FILE" ]; then
    echo "Creating default config..."
    if ! cat > "$CONFIG_FILE" << 'EOL'; then
        echo "Error: Failed to create config file"
        exit 1
    fi

socket = "/tmp/mpvsocket"

# Get this from https://discord.com/developers/applications if you want to have personal assets
client_id = "1322011605432533082"

large_image = "mpv_large"
small_image = "mpv_small"

status_text = "Watching Anime"
EOL
fi

MPV_CONFIG="$MPV_CONFIG_DIR/mpv.conf"
if [ ! -f "$MPV_CONFIG" ] || ! grep -q "input-ipc-server=/tmp/mpvsocket" "$MPV_CONFIG"; then
    echo "Configuring MPV..."
    if ! echo -e "\n# Added by discord_mpv_rpc\ninput-ipc-server=/tmp/mpvsocket" >> "$MPV_CONFIG"; then
        echo "Error: Failed to update MPV config"
        exit 1
    fi
fi

echo -e "\nInstallation complete!"
echo "Discord RPC will start automatically when you play videos in MPV"
echo "Press 'D' in MPV to toggle Discord RPC on/off"
echo -e "\nDon't forget to set your client_id in: $CONFIG_FILE"
