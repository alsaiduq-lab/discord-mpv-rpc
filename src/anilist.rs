use anyhow::Result;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct AniListResponse {
    pub data: AniListData,
}

#[derive(Debug, Deserialize)]
pub struct AniListData {
    pub Media: Option<Media>,
}

#[derive(Debug, Deserialize)]
pub struct Media {
    pub id: i32,
    pub title: MediaTitle,
    #[serde(rename = "type")]
    pub media_type: String,
    pub episodes: Option<i32>,
    pub duration: Option<i32>,
    pub coverImage: CoverImage,
}

#[derive(Debug, Deserialize)]
pub struct MediaTitle {
    pub romaji: Option<String>,
    pub english: Option<String>,
    pub native: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CoverImage {
    pub large: String,
}

pub async fn search_anime(title: &str) -> Result<Option<Media>> {
    let query = r#"
    query ($search: String) {
        Media(search: $search, type: ANIME) {
            id
            title {
                romaji
                english
                native
            }
            type
            episodes
            duration
            coverImage {
                large
            }
        }
    }
    "#;

    let mut variables = HashMap::new();
    variables.insert("search", title);

    let client = reqwest::Client::new();
    let res = client
        .post("https://graphql.anilist.co")
        .json(&serde_json::json!({
            "query": query,
            "variables": variables
        }))
        .send()
        .await?
        .json::<AniListResponse>()
        .await?;

    Ok(res.data.Media)
}

pub fn get_preferred_title(media: &Media) -> String {
    media.title.english
        .as_ref()
        .or(media.title.romaji.as_ref())
        .or(media.title.native.as_ref())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "Unknown Anime".to_string())
}
