use regex::Regex;
use once_cell::sync::Lazy;

static EPISODE_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(?:\[.*?\])?.*?(?:episode|ep|e)\.?\s*(\d+).*?(?:\[.*?\])?").unwrap()
});

static CLEANUP_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        Regex::new(r"(?i)\[(.*?)\]").unwrap(),        // Remove [Square Brackets]
        Regex::new(r"(?i)\((.*?)\)").unwrap(),        // Remove (Parentheses)
        Regex::new(r"(?i)\.mkv$|\.mp4$").unwrap(),    // Remove video extensions
        Regex::new(r"(?i)_|-").unwrap(),              // Replace underscore/dash with space
        Regex::new(r"(?i)episode|ep|\be\b").unwrap(), // Remove episode indicators
    ]
});

#[derive(Debug)]
pub struct AnimeInfo {
    pub title: String,
    pub episode: Option<i32>,
}

impl AnimeInfo {
    pub fn from_filename(filename: &str) -> Self {
        let episode = EPISODE_PATTERN
            .captures(filename)
            .and_then(|cap| cap.get(1))
            .and_then(|m| m.as_str().parse::<i32>().ok());

        let clean_title = clean_title(filename);

        AnimeInfo {
            title: clean_title,
            episode,
        }
    }
}

fn clean_title(filename: &str) -> String {
    let mut cleaned = filename.to_string();

    // Replace special characters with spaces
    cleaned = cleaned.replace(['_', '-', '.'], " ");

    // Apply all cleanup patterns
    for pattern in CLEANUP_PATTERNS.iter() {
        cleaned = pattern.replace_all(&cleaned, " ").to_string();
    }

    // Clean up extra whitespace
    cleaned = cleaned
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    cleaned.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_episode_extraction() {
        let test_cases = vec![
            ("[Group] Anime Title - Episode 01 [1080p]", Some(1)),
            ("Anime.Title.E02.mkv", Some(2)),
            ("Anime Title ep.03", Some(3)),
            ("Anime Title - 04", None),
            ("[Group] Anime Title - E05v2 [1080p]", Some(5)),
        ];

        for (input, expected) in test_cases {
            let info = AnimeInfo::from_filename(input);
            assert_eq!(info.episode, expected, "Failed for input: {}", input);
        }
    }

    #[test]
    fn test_title_cleaning() {
        let test_cases = vec![
            (
                "[Group] My Hero Academia - Episode 01 [1080p]",
                "My Hero Academia",
            ),
            (
                "One.Piece.E1000.mkv",
                "One Piece",
            ),
            (
                "[SubsGroup]_Naruto_Shippuden_-_ep001_(1080p)",
                "Naruto Shippuden",
            ),
        ];

        for (input, expected) in test_cases {
            let info = AnimeInfo::from_filename(input);
            assert_eq!(info.title, expected, "Failed for input: {}", input);
        }
    }
}
