use anyhow::{anyhow, bail, Context, Result};
use chrono::Utc;
use clap::Parser;
use discord_rpc_client::Client as DiscordRpcClient;
use serde::Deserialize;
use serde_json::json;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::UnixStream,
    time::sleep,
};

#[derive(Debug, Deserialize, Clone)]
struct Config {
    socket: Option<String>,
    client_id: Option<String>,
    large_image: Option<String>,
    small_image: Option<String>,
}

#[derive(Debug, Parser)]
#[command(version, about = "Discord Rich Presence for MPV")]
struct CliArgs {
    #[arg(short = 'c', long = "config")]
    config_file: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct MpvResponse<T> {
    data: Option<T>,
}

fn format_time(seconds: f64) -> String {
    let total_seconds = seconds as u64;
    let hours = total_seconds / 3600;
    let mins = (total_seconds % 3600) / 60;
    let secs = total_seconds % 60;
    format!("{:02}:{:02}:{:02}", hours, mins, secs)
}

#[derive(Debug)]
struct MediaMetadata {
    title: Option<String>,
    artist: Option<String>,
    media_type: MediaType,
    filename: Option<String>,
}

#[derive(Debug)]
enum MediaType {
    Video,
    Music,
    Unknown,
}

impl MediaMetadata {
    async fn from_mpv(mpv_stream: &mut UnixStream) -> Result<Self> {
        let title = mpv_get_property::<String>(mpv_stream, "media-title").await?;
        let artist = mpv_get_property::<String>(mpv_stream, "metadata/by-key/artist").await?;
        let filename = mpv_get_property::<String>(mpv_stream, "filename").await?;

        let media_type = if let Some(ref fname) = filename {
            if fname.ends_with(".mp3") || 
               fname.ends_with(".flac") || 
               fname.ends_with(".m4a") || 
               fname.ends_with(".opus") {
                MediaType::Music
            } else if fname.ends_with(".mkv") || 
                      fname.ends_with(".mp4") || 
                      fname.ends_with(".webm") ||
                      fname.contains("youtube.com") ||
                      fname.contains("youtu.be") {
                MediaType::Video
            } else {
                MediaType::Unknown
            }
        } else {
            MediaType::Unknown
        };

        Ok(MediaMetadata { title, artist, media_type, filename })
    }
}

async fn mpv_get_property<T>(stream: &mut UnixStream, property: &str) -> Result<Option<T>>
where
    T: for<'de> serde::Deserialize<'de>,
{
    let request = json!({ "command": ["get_property", property] }).to_string();
    stream
        .write_all((request + "\n").as_bytes())
        .await
        .context("Failed to write to MPV socket")?;
    stream.flush().await.context("Failed to flush MPV socket")?;

    let mut reader = BufReader::new(stream);
    let mut response_line = String::new();
    let bytes_read = reader
        .read_line(&mut response_line)
        .await
        .context("Failed to read from MPV")?;

    if bytes_read == 0 {
        bail!("MPV socket closed");
    }

    let parsed: MpvResponse<T> = serde_json::from_str(&response_line)
        .context("Failed to parse MPV response")?;
    Ok(parsed.data)
}

async fn update_presence(
    mpv_stream: &mut UnixStream,
    discord_client: &mut DiscordRpcClient,
    config: &Config,
) -> Result<bool> {
    let metadata = MediaMetadata::from_mpv(mpv_stream).await?;

    let title = metadata.title.unwrap_or_else(|| metadata.filename.unwrap_or_else(|| "Unknown Title".to_string()));
    let duration = mpv_get_property::<f64>(mpv_stream, "duration").await?.unwrap_or(0.0);
    let position = mpv_get_property::<f64>(mpv_stream, "time-pos").await?.unwrap_or(0.0);
    let paused = mpv_get_property::<bool>(mpv_stream, "pause").await?.unwrap_or(false);

    let large_image = config.large_image.as_deref().unwrap_or("default_large");
    let small_image = config.small_image.as_deref().unwrap_or("default_small");
    let status_text = if paused {
        "⏸️ Paused".to_string()
    } else {
        format!("{} / {}", format_time(position), format_time(duration))
    };

    let now = Utc::now().timestamp() as u64;
    let start_time = (now as i64 - position as i64) as u64;
    let end_time = (start_time as i64 + (duration - position) as i64) as u64;

    let details = match metadata.media_type {
        MediaType::Music => {
            if let Some(artist) = metadata.artist {
                format!("{} - {}", title, artist)
            } else {
                title
            }
        },
        _ => title,
    };

    let _ = discord_client.set_activity(|activity| {
        activity
            .state(&status_text)
            .details(&details)
            .assets(|assets| {
                assets
                    .large_image(large_image)
                    .small_image(small_image)
            })
            .timestamps(|timestamps| {
                timestamps
                    .start(start_time)
                    .end(end_time)
            })
    });

    Ok(!paused)
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = CliArgs::parse();

    let config_path = args.config_file.unwrap_or_else(|| {
        dirs::config_dir()
            .expect("Could not find config directory")
            .join("discord_mpv_rpc/config.toml")
    });

    let config: Config = if config_path.exists() {
        let content = fs::read_to_string(&config_path)
            .context("Failed to read config file")?;
        toml::from_str(&content)
            .context("Failed to parse config file")?
    } else {
        println!("Config file not found at: {}", config_path.display());
        println!("Please run the installer");
        bail!("Missing configuration");
    };

    let client_id = config.client_id.as_deref()
        .ok_or_else(|| anyhow!("Discord client_id not set in config file. Please add it to: {}", config_path.display()))?
        .parse::<u64>()
        .context("Invalid client_id - must be a number")?;

    let socket_path = config.socket.as_deref().unwrap_or("/tmp/mpvsocket");
    let mut mpv_stream = UnixStream::connect(socket_path)
        .await
        .with_context(|| format!("Could not connect to MPV at {}. Is MPV running?", socket_path))?;

    let mut discord_client = DiscordRpcClient::new(client_id);
    
    discord_client.start();
    println!("Connected to MPV and Discord successfully!");

    let mut reconnect_attempts = 0;
    let max_reconnects = 3;
    let mut last_update = Instant::now();
    let update_interval = Duration::from_millis(500);

    loop {
        match update_presence(&mut mpv_stream, &mut discord_client, &config).await {
            Ok(active) => {
                reconnect_attempts = 0;
                if !active {
                    sleep(Duration::from_secs(2)).await;
                } else {
                    let elapsed = last_update.elapsed();
                    if elapsed < update_interval {
                        sleep(update_interval - elapsed).await;
                    }
                    last_update = Instant::now();
                }
            }
            Err(e) => {
                eprintln!("Error: {:#}", e);
                reconnect_attempts += 1;

                if reconnect_attempts >= max_reconnects {
                    bail!("Failed after {} attempts", max_reconnects);
                }

                println!("Attempting to reconnect ({}/{})", reconnect_attempts, max_reconnects);
                sleep(Duration::from_secs(1)).await;

                mpv_stream = match UnixStream::connect(socket_path).await {
                    Ok(stream) => stream,
                    Err(e) => {
                        eprintln!("Failed to reconnect to MPV: {}", e);
                        continue;
                    }
                };

                discord_client = DiscordRpcClient::new(client_id);
                discord_client.start();
            }
        }
    }
}
