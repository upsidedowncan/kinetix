use std::fs;
use std::io::BufReader;
use std::path::PathBuf;
use std::process::Command;
use serde::Serialize;
use tauri::AppHandle;
use tauri::Manager;
use font_loader::system_fonts;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn get_system_fonts() -> Vec<String> {
    let mut fonts = system_fonts::query_all();
    fonts.sort();
    fonts.dedup();
    fonts
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// Custom stream protocol handler for low-latency local video playback
#[tauri::command]
async fn stream_video(file_path: String) -> Result<Vec<u8>, String> {
    let path = PathBuf::from(&file_path);
    
    if !path.exists() {
        return Err("File not found".to_string());
    }

    fs::read(&path).map_err(|e| format!("Failed to read file: {}", e))
}

#[derive(Serialize)]
struct ProxyResponse {
    proxy_path: String,
    thumbnail_path: String,
    duration_secs: f64,
}

// Generate a web-friendly proxy video and a thumbnail using FFmpeg CLI
#[tauri::command]
async fn generate_proxy_video(app: AppHandle, file_path: String) -> Result<ProxyResponse, String> {
    let path = PathBuf::from(&file_path);
    if !path.exists() {
        return Err("File not found".to_string());
    }

    let cache_dir = app.path().app_cache_dir().map_err(|e| e.to_string())?;
    let proxy_dir = cache_dir.join("proxies");
    let thumb_dir = cache_dir.join("thumbnails");
    
    if !proxy_dir.exists() { fs::create_dir_all(&proxy_dir).map_err(|e| e.to_string())?; }
    if !thumb_dir.exists() { fs::create_dir_all(&thumb_dir).map_err(|e| e.to_string())?; }

    let file_name = path.file_name().ok_or("Invalid filename")?.to_string_lossy();
    let proxy_path = proxy_dir.join(format!("{}.proxy.webm", file_name));
    let thumb_path = thumb_dir.join(format!("{}.thumb.jpg", file_name));

    // 1. Generate Thumbnail (if not exists)
    if !thumb_path.exists() {
        println!("Backend: Generating thumbnail for {}", file_path);
        let _ = Command::new("ffmpeg")
            .args([
                "-y",
                "-ss", "00:00:01", // Grab frame at 1 second
                "-i", &file_path,
                "-frames:v", "1",
                "-q:v", "4",
                thumb_path.to_str().ok_or("Invalid thumb path")?
            ])
            .status();
    }

    // 2. Generate Proxy (if not exists)
    if !proxy_path.exists() {
        println!("Backend: Starting FFmpeg WebM proxy generation for {}", file_path);
        let status = Command::new("ffmpeg")
            .args([
                "-y",
                "-i", &file_path,
                "-vf", "scale=-2:720",
                "-c:v", "libvpx",
                "-crf", "10",
                "-b:v", "1M",
                "-c:a", "libvorbis",
                proxy_path.to_str().ok_or("Invalid proxy path")?
            ])
            .status()
            .map_err(|e| format!("Failed to execute ffmpeg: {}. Is ffmpeg installed?", e))?;

        if !status.success() {
            return Err("ffmpeg failed to generate proxy video".to_string());
        }
        println!("Backend: Proxy generated successfully: {}", proxy_path.display());
    }

    let meta = get_video_metadata(file_path)?;
    
    Ok(ProxyResponse {
        proxy_path: proxy_path.to_string_lossy().to_string(),
        thumbnail_path: thumb_path.to_string_lossy().to_string(),
        duration_secs: meta.duration_secs,
    })
}

#[derive(Serialize)]
struct VideoMetadata {
    duration_secs: f64,
    width: u32,
    height: u32,
    format: String,
}

// Get video metadata (duration) using pure Rust mp4 crate
#[tauri::command]
fn get_video_metadata(file_path: String) -> Result<VideoMetadata, String> {
    let path = PathBuf::from(&file_path);
    
    if !path.exists() {
        return Err("File not found".to_string());
    }

    // Try to read as MP4
    let file = fs::File::open(&path).map_err(|e| format!("Failed to open file: {}", e))?;
    let size = file.metadata().map_err(|e| format!("Failed to get metadata: {}", e))?.len();
    let reader = BufReader::new(file);
    
    match mp4::Mp4Reader::read_header(reader, size) {
        Ok(mp4) => {
            // Get duration from mp4
            let duration = mp4.duration();
            let duration_secs = duration.as_secs_f64();
            
            println!("MP4 duration: {} seconds", duration_secs);
            
            Ok(VideoMetadata {
                duration_secs,
                width: 0,
                height: 0,
                format: "mp4".to_string(),
            })
        }
        Err(e) => {
            // If not MP4, return 0 duration (frontend will fall back to browser method)
            eprintln!("Failed to parse MP4 (file may be different format): {}", e);
            Ok(VideoMetadata {
                duration_secs: 0.0,
                width: 0,
                height: 0,
                format: "unknown".to_string(),
            })
        }
    }
}

// Read video file as bytes for frontend blob URL creation
#[tauri::command]
async fn read_video_file(file_path: String) -> Result<Vec<u8>, String> {
    println!("Backend: Reading file into memory: {}", file_path);
    let path = PathBuf::from(&file_path);
    if !path.exists() {
        return Err("File not found".to_string());
    }
    
    fs::read(&path).map_err(|e| format!("Failed to read file: {}", e))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![greet, stream_video, get_video_metadata, read_video_file, generate_proxy_video, get_system_fonts])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
