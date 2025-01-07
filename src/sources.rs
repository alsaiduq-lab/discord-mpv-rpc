#[derive(Debug, Clone, PartialEq)]
pub enum MediaSource {
    YouTube(YouTubeInfo),
    LocalFile(LocalFileInfo),
    Crunchyroll,
    AnimePahe,
    NineAnime,
    Other(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct YouTubeInfo {
    pub title: String,
    pub channel: Option<String>,
    pub uploader: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalFileInfo {
    pub filename: String,
    pub is_anime: bool,
}

impl MediaSource {
    pub fn detect(filename: &str, metadata: &MediaMetadata) -> Self {
        if filename.contains("youtube.com") || filename.contains("youtu.be") {
            MediaSource::YouTube(YouTubeInfo {
                title: metadata.title.clone().unwrap_or_default(),
                channel: metadata.channel.clone(),
                uploader: metadata.uploader.clone(),
            })
        } else if filename.starts_with("http") {
            match filename {
                _ if filename.contains("crunchyroll.com") => MediaSource::Crunchyroll,
                _ if filename.contains("animepahe") => MediaSource::AnimePahe,
                _ if filename.contains("9anime") => MediaSource::NineAnime,
                _ => MediaSource::Other(filename.to_string()),
            }
        } else {
            let is_anime = Self::looks_like_anime(filename);
            MediaSource::LocalFile(LocalFileInfo {
                filename: filename.to_string(),
                is_anime,
            })
        }
    }

    fn looks_like_anime(filename: &str) -> bool {
        let lower = filename.to_lowercase();
        let anime_keywords = [
            "[horriblesubs]",
            "[subsplease]",
            "[erai-raws]",
            "[nyaa]",
            " - episode",
            ".mkv",
        ];
        anime_keywords.iter().any(|&keyword| lower.contains(keyword))
    }

    pub fn get_image_key(&self) -> &'static str {
        match self {
            MediaSource::YouTube(_) => "youtube_large",
            MediaSource::LocalFile(info) if info.is_anime => "anime_large",
            MediaSource::LocalFile(_) => "mpv_large",
            MediaSource::Crunchyroll => "crunchyroll_large",
            MediaSource::AnimePahe | MediaSource::NineAnime => "anime_large",
            MediaSource::Other(_) => "mpv_large",
        }
    }

    pub fn get_small_image_key(&self) -> &'static str {
        match self {
            MediaSource::YouTube(_) => "youtube_small",
            MediaSource::LocalFile(info) if info.is_anime => "anime_small",
            MediaSource::LocalFile(_) => "mpv_small",
            MediaSource::Crunchyroll => "crunchyroll_small",
            MediaSource::AnimePahe | MediaSource::NineAnime => "anime_small",
            MediaSource::Other(_) => "mpv_small",
        }
    }
}

#[derive(Debug, Default)]
pub struct MediaMetadata {
    pub title: Option<String>,
    pub channel: Option<String>,
    pub uploader: Option<String>,
    pub date: Option<String>,
    pub channel_url: Option<String>,
}

impl MediaMetadata {
    pub async fn from_mpv(stream: &mut UnixStream) -> Result<Self> {
        let mut metadata = MediaMetadata::default();
        
        if let Some(title) = mpv_get_property::<String>(stream, "media-title").await? {
            metadata.title = Some(title);
        }

        let youtube_fields = [
            ("youtube-uploader", |m: &mut MediaMetadata, v| m.uploader = Some(v)),
            ("youtube-channel", |m: &mut MediaMetadata, v| m.channel = Some(v)),
            ("youtube-channel-url", |m: &mut MediaMetadata, v| m.channel_url = Some(v)),
            ("youtube-upload-date", |m: &mut MediaMetadata, v| m.date = Some(v)),
        ];

        for (field, setter) in youtube_fields {
            if let Some(value) = mpv_get_property::<String>(stream, field).await? {
                setter(&mut metadata, value);
            }
        }

        Ok(metadata)
    }
}
