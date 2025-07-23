
mod db;
mod http;
mod ml;

pub mod config {

    use std::{env, io::{self, ErrorKind}, path::Path};

    use axum::http::{HeaderValue, Method};
    use chrono::{Duration, Timelike, Utc};
    use tokio::time::{sleep_until, Instant};
    use tower_http::cors::CorsLayer;

    use crate::{db::db_conn::drop_all_uploads};

    pub async fn create_upload_dir() {
        let dir_upload = env::var("SERVER_DATA").expect("UPLOADS DIR env var not found");
        
        let dir = Path::new(&dir_upload).join("uploads");

        let _upload_dir = match tokio::fs::create_dir_all(&dir).await {
            Ok(_) => Ok(()),
            Err(e) if e.kind() == ErrorKind::AlreadyExists => Ok(()),
            Err(e) => Err(Box::new(e)),
        }
        .expect("Should create directory");
    }

    pub fn create_cors_layers() -> CorsLayer {
        CorsLayer::new()
            .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
            .allow_methods([Method::GET, Method::POST])
    }

    pub fn create_server_data_dirs(base: &str) -> std::io::Result<()> {
        let folders = [
            "",
            "30s",
            "cqt",
            "ft",
            "mfcc",
            "spectr",
            "tonnetz",
            "cens",
            "features",
            "mel_spect",
            "power_spectr",
            "stft",
            "uploads",
        ];

        for folder in &folders {
            let path = if folder.is_empty() {
                Path::new(base).to_path_buf()
            } else {
                Path::new(base).join(folder)
            };
            std::fs::create_dir_all(path)?;
        }

        Ok(())
    }

    pub fn clear_server_data() -> io::Result<()> {
    
    let server_data = env::var("SERVER_DATA").expect("UPLOADS DIR env var not found");
    
    let base_path = Path::new(&server_data);

    if base_path.exists() {
        std::fs::remove_dir_all(base_path).expect("Failed to remove SERVER_DATA directory")
    }

    Ok(())
    }

    pub fn recreate_server_data() -> io::Result<()> {
    
        let server_data = env::var("SERVER_DATA").expect("UPLOADS DIR env var not found");
        
        let base_path = Path::new(&server_data);
    
        if base_path.exists() {
            std::fs::create_dir_all(base_path).expect("Failed to remove SERVER_DATA directory")
        }
    
        Ok(())
    }


    pub async fn start_4hourly_task() {
        tokio::spawn(async {
            loop {
                let now = Utc::now();
                let next_run_hour = ((now.hour() / 4) + 1) * 4 % 24;
    
                let mut next_run = now
                    .date_naive()
                    .and_hms_opt(next_run_hour, 0, 0)
                    .unwrap();
    
                if next_run <= now.naive_utc() {
                    next_run += Duration::hours(4);
                }
    
                let sleep_duration = (next_run - now.naive_utc()).to_std().unwrap();
                sleep_until(Instant::now() + sleep_duration).await;
    
                if let Err(e) = drop_all_uploads().await {
                    tracing::error!("Failed to drop uploads: {:?}", e);
                }
            }
        });
    }

    



}
