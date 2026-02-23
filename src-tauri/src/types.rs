use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct MediaItem {
    pub name: String,
    pub group: String,
    pub resolution: String,
    pub source: String,
    pub video_codec: String,
    pub audio_codec: String,
    pub season: Option<String>,
    pub path: String,
    pub is_airing: bool,
    pub avg_size_gb: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub last_library_path: Option<String>,
    pub media_statuses: HashMap<String, String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            last_library_path: None,
            media_statuses: HashMap::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum StatusRank {
    Airing = 1,
    Great = 2,
    Good = 3,
    Okay = 4,
    Bad = 5,
    None = 6,
}

impl StatusRank {
    pub fn from_item(item: &MediaItem) -> Self {
        if item.is_airing {
            return StatusRank::Airing;
        }

        let source = item.source.to_lowercase().replace("-", " ");
        let video = item.video_codec.to_lowercase().replace("-", " ");

        if source.contains("web dl") {
            return StatusRank::Bad;
        }

        if source.contains("bd encode") {
            if video.contains("svt av1") {
                return StatusRank::Good;
            } else {
                return StatusRank::Okay;
            }
        }

        if source.contains("bd remux") || source.contains("dvd") {
            if video.contains("h.264") || video.contains("x264") || video.contains("mpeg2") || video.contains("mpeg 2") {
                return StatusRank::Great;
            }
        }

        StatusRank::None
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SortColumn {
    Name,
    Season,
    Group,
    Resolution,
    Source,
    VideoCodec,
    AudioCodec,
    AvgSize,
    Verified,
    Status, // Rank based
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SortOrder {
    Ascending,
    Descending,
}
