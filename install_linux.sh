#!/bin/bash
set -e

echo "Installing discord_mpv_rpc..."

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
    echo "Error: cargo is not installed. Please install Rust first."
    exit 1
fi

cargo build --release || {
    echo "Build failed. Please check the error messages above."
    exit 1
}

CONFIG_DIR="$HOME/.config/discord_mpv_rpc"
MPV_CONFIG_DIR="$HOME/.config/mpv"
MPV_SCRIPTS_DIR="$HOME/.config/mpv/scripts"

mkdir -p "$CONFIG_DIR"
mkdir -p "$MPV_CONFIG_DIR"
mkdir -p "$MPV_SCRIPTS_DIR"

echo "Installing binary..."
sudo cp target/release/discord_mpv_rpc /usr/local/bin/ || {
    echo "Failed to install binary. Please check permissions."
    exit 1
}

echo "Installing MPV script..."
cp discord-rpc.lua "$MPV_SCRIPTS_DIR/" || {
    echo "Failed to install MPV script."
    exit 1
}

CONFIG_FILE="$CONFIG_DIR/config.toml"
if [ ! -f "$CONFIG_FILE" ]; then
    echo "Creating default config..."
    cat > "$CONFIG_FILE" << EOL

socket = "/tmp/mpvsocket"

# Get this from https://discord.com/developers/applications if you want to have personal assets
client_id = "1322011605432533082"

large_image = "mpv_large"
small_image = "mpv_small"

# Optional: Custom status text
status_text = "Watching Anime"
EOL
fi

# Configure MPV
MPV_CONFIG="$MPV_CONFIG_DIR/mpv.conf"
if [ ! -f "$MPV_CONFIG" ] || ! grep -q "input-ipc-server=/tmp/mpvsocket" "$MPV_CONFIG"; then
    echo "Configuring MPV..."
    echo -e "\n# Added by discord_mpv_rpc\ninput-ipc-server=/tmp/mpvsocket" >> "$MPV_CONFIG"
fi

echo -e "\nInstallation complete!"
echo "Discord RPC will start automatically when you play videos in MPV"
echo "Press 'D' in MPV to toggle Discord RPC on/off"
echo -e "\nDon't forget to set your client_id in: $CONFIG_FILE"

