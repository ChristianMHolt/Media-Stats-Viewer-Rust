use crate::types::MediaItem;
use rayon::prelude::*;
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

static RES_REGEX: OnceLock<Regex> = OnceLock::new();
static SOURCE_REGEX: OnceLock<Regex> = OnceLock::new();
static VIDEO_REGEX: OnceLock<Regex> = OnceLock::new();
static AUDIO_REGEX: OnceLock<Regex> = OnceLock::new();
static TAGS_REGEX: OnceLock<Regex> = OnceLock::new();

fn get_res_regex() -> &'static Regex {
    RES_REGEX.get_or_init(|| Regex::new(r"(?i)(\d{3,4}p|4K|2K|8K)").unwrap())
}

fn get_source_regex() -> &'static Regex {
    SOURCE_REGEX.get_or_init(|| Regex::new(r"(?i)(BD[- ]?Encode|WEB[- ]?DL|BluRay|HDTV|DVD|Remux)").unwrap())
}

fn get_video_regex() -> &'static Regex {
    VIDEO_REGEX.get_or_init(|| Regex::new(r"(?i)(H\.?264|x264|H\.?265|x265|HEVC|AV1|SVT[- ]?AV1|VP9)").unwrap())
}

fn get_audio_regex() -> &'static Regex {
    AUDIO_REGEX.get_or_init(|| Regex::new(r"(?i)(AAC|DTS|FLAC|OPUS|AC3|E-AC3|TrueHD|Atmos)").unwrap())
}

fn get_tags_regex() -> &'static Regex {
    TAGS_REGEX.get_or_init(|| Regex::new(r"\[(.*?)\]").unwrap())
}

const VIDEO_EXTENSIONS: &[&str] = &[
    ".mkv", ".mp4", ".avi", ".mov", ".wmv", ".flv", ".webm", ".m4v", ".ts", ".m2ts",
];

fn is_video_file(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        if let Some(ext_str) = ext.to_str() {
            let ext_lower = format!(".{}", ext_str.to_lowercase());
            return VIDEO_EXTENSIONS.contains(&ext_lower.as_str());
        }
    }
    false
}

fn calculate_average_size(folder_path: &Path) -> f64 {
    let mut total_size = 0u64;
    let mut count = 0usize;

    if let Ok(entries) = fs::read_dir(folder_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && is_video_file(&path) {
                if let Ok(metadata) = path.metadata() {
                    total_size += metadata.len();
                    count += 1;
                }
            }
        }
    }

    if count == 0 {
        return 0.0;
    }

    let avg_bytes = total_size as f64 / count as f64;
    avg_bytes / (1024.0 * 1024.0 * 1024.0)
}

fn parse_root_folder(folder_name: &str, path: &Path) -> MediaItem {
    let mut item = MediaItem {
        name: folder_name.to_string(),
        path: path.to_string_lossy().to_string(),
        ..Default::default()
    };

    if let Some(idx) = folder_name.find('[') {
        item.name = folder_name[..idx].trim().to_string();
        let bracket_content = format!("[{}", &folder_name[idx + 1..]);
        
        let tags: Vec<String> = get_tags_regex()
            .captures_iter(&bracket_content)
            .map(|c| c[1].to_string())
            .collect();

        if tags.len() >= 5 {
             item.group = tags[0].clone();
             item.resolution = tags[1].clone();
             item.source = tags[2].clone();
             item.video_codec = tags[3].clone();
             item.audio_codec = tags[4].clone();
        } else {
             if !tags.is_empty() { item.group = tags[0].clone(); }
             if tags.len() > 1 { item.resolution = tags[1].clone(); }
             if tags.len() > 2 { item.source = tags[2].clone(); }
             if tags.len() > 3 { item.video_codec = tags[3].clone(); }
             if tags.len() > 4 { item.audio_codec = tags[4].clone(); }
        }
    }

    item
}

fn parse_season_override(season_folder: &str, parent: &MediaItem, path: &Path) -> MediaItem {
    let mut item = parent.clone();
    item.path = path.to_string_lossy().to_string();
    
    let season_name;
    let tags: Vec<String>;

    if let Some(idx) = season_folder.find('[') {
        season_name = season_folder[..idx].trim().to_string();
        let bracket_content = format!("[{}", &season_folder[idx + 1..]);
        tags = get_tags_regex()
            .captures_iter(&bracket_content)
            .map(|c| c[1].to_string())
            .collect();
    } else {
        season_name = season_folder.trim().to_string();
        tags = Vec::new();
    }
    item.season = Some(season_name);

    if tags.iter().any(|t| t.eq_ignore_ascii_case("airing")) {
        item.is_airing = true;
        item.resolution = "Airing".to_string();
        item.source = "Airing".to_string();
        item.video_codec = "Airing".to_string();
        item.audio_codec = "Airing".to_string();
    } else {
        for tag in tags {
            if get_res_regex().is_match(&tag) {
                item.resolution = tag;
            } else if get_source_regex().is_match(&tag) {
                item.source = tag;
            } else if get_video_regex().is_match(&tag) {
                item.video_codec = tag;
            } else if get_audio_regex().is_match(&tag) {
                item.audio_codec = tag;
            }
        }
    }

    item
}

pub fn scan_library(root_path: &str) -> Vec<MediaItem> {
    let root = Path::new(root_path);
    if !root.exists() || !root.is_dir() {
        return Vec::new();
    }

    // Collect top-level entries first
    let entries: Vec<PathBuf> = match fs::read_dir(root) {
        Ok(read_dir) => read_dir.filter_map(|e| e.ok().map(|e| e.path())).collect(),
        Err(_) => return Vec::new(),
    };

    // Parallel processing
    entries.par_iter().filter(|p| p.is_dir()).flat_map(|path| {
        let folder_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
        let mut parent_item = parse_root_folder(&folder_name, path);
        
        let mut sub_items = Vec::new();
        let mut has_seasons = false;

        if let Ok(sub_entries) = fs::read_dir(path) {
             let sub_dirs: Vec<PathBuf> = sub_entries
                 .filter_map(|e| e.ok().map(|e| e.path()))
                 .filter(|p| p.is_dir())
                 .collect();
             
             // Check for "Season"
             // Sort to ensure order? Not strictly necessary for output as we return list, but good for stability if needed.
             // We'll just filter.
             
             for sub in sub_dirs {
                 let sub_name = sub.file_name().unwrap_or_default().to_string_lossy();
                 if sub_name.to_lowercase().starts_with("season") {
                     has_seasons = true;
                     let mut season_item = parse_season_override(&sub_name, &parent_item, &sub);
                     season_item.avg_size_gb = calculate_average_size(&sub);
                     sub_items.push(season_item);
                 }
             }
        }

        if !has_seasons {
            parent_item.avg_size_gb = calculate_average_size(path);
            vec![parent_item]
        } else {
            sub_items
        }
    }).collect()
}
