use std::process::Command;
use std::time::{SystemTime, Duration};
use std::fs;
use std::path::Path;
use std::env;
use std::io;

use axum::{routing::get, Error, Router};
use routes::routes;
use tracing::{info, error, Level};
use tracing_subscriber;
use dotenvy::dotenv;

mod handlers;
mod models;
mod routes;
mod cors;

use tokio::{fs::File, io::{AsyncBufReadExt, BufReader}};
use serde_json::Value;
use reqwest::Client;

use crate::cors::cors::create_cors;

// ---------------------------
// 변경된 부분: Lazy 사용
use once_cell::sync::Lazy;

static LAST_PROCESSED_TIME: Lazy<std::sync::Mutex<SystemTime>> = Lazy::new(|| {
    std::sync::Mutex::new(SystemTime::now())
});

static CONFIG: Lazy<Config> = Lazy::new(|| {
    Config::from_env()
});

pub struct Config {
    pub central_api_server_url: String,
    pub log_watch_interval: u64,
    pub cleanup_interval: u64,
    pub log_dir: String,
    pub active_log_file: String,
    pub max_log_size_mb: u64,
    pub port: u16,
}

impl Config {
    pub fn from_env() -> Self {
        let central_api_server_url = env::var("CENTRAL_API_SERVER_URL")
            .unwrap_or_else(|_| String::from("http://127.0.0.1:3000"));
        
        let log_watch_interval = env::var("LOG_WATCH_INTERVAL")
            .ok()
            .and_then(|val| val.parse::<u64>().ok())
            .unwrap_or(2);
        
        let cleanup_interval = env::var("CLEANUP_INTERVAL")
            .ok()
            .and_then(|val| val.parse::<u64>().ok())
            .unwrap_or(60);

        let default_log_dir = match env::var("HOME") {
            Ok(home) => format!("{}/suricata_logs", home),
            Err(_) => String::from("./suricata_logs"),
        };
        
        let log_dir = env::var("LOG_DIR").unwrap_or_else(|_| default_log_dir);

        let active_log_file = env::var("ACTIVE_LOG_FILE")
            .unwrap_or_else(|_| format!("{}/eve.json", log_dir));
        
        let max_log_size_mb = env::var("MAX_LOG_SIZE_MB")
            .ok()
            .and_then(|val| val.parse::<u64>().ok())
            .unwrap_or(100);
        
        let port = env::var("PORT")
            .ok()
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or(3000);
        
        info!("환경 변수 설정:");
        info!("CENTRAL_API_SERVER_URL: {}", central_api_server_url);
        info!("LOG_WATCH_INTERVAL: {}초", log_watch_interval);
        info!("CLEANUP_INTERVAL: {}초", cleanup_interval);  
        info!("LOG_DIR: {}", log_dir);
        info!("ACTIVE_LOG_FILE: {}", active_log_file);
        info!("MAX_LOG_SIZE_MB: {}MB", max_log_size_mb);
        info!("PORT: {}", port);
        
        Config {
            central_api_server_url,
            log_watch_interval,
            cleanup_interval,
            log_dir,
            active_log_file,
            max_log_size_mb,
            port,
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv().ok();
    setup_logging();

    load_and_print_configs();

    // ensure_log_directory_exists 에러 발생 시 로그를 남기고 Error로 전파
    ensure_log_directory_exists(&CONFIG.log_dir).await.map_err(|e| {
        error!("로그 디렉토리 생성 실패: {}", e);
        Error::new(e)
    })?;

    let app = setup_routes();

    start_background_tasks().await;

    start_server(app, CONFIG.port).await?;
    Ok(())
}

fn setup_logging() {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();
}

fn load_and_print_configs() {
    info!("환경 설정 로드됨:");
    info!("중앙 API 서버: {}", CONFIG.central_api_server_url);
    info!("로그 감시 주기: {}초", CONFIG.log_watch_interval);
    info!("로그 정리 주기: {}초", CONFIG.cleanup_interval);
    info!("로그 디렉토리: {}", CONFIG.log_dir);
    info!("활성 로그 파일: {}", CONFIG.active_log_file);
    info!("최대 로그 크기: {}MB", CONFIG.max_log_size_mb);
    info!("서버 포트: {}", CONFIG.port);
}

async fn ensure_log_directory_exists(log_dir: &str) -> Result<(), io::Error> {
    match fs::create_dir_all(log_dir) {
        Ok(_) => {
            info!("로그 디렉토리 확인 완료: {}", log_dir);
            Ok(())
        },
        Err(e) => {
            error!("로그 디렉토리 생성 실패: {}. 직접 생성해 주세요: {}", e, log_dir);
            Err(e)
        }
    }
}

fn setup_routes() -> Router {
    Router::new()
        .route("/", get(root))
        .merge(routes())
        .layer(create_cors())
}

async fn start_background_tasks() {
    let _active_log_file = CONFIG.active_log_file.clone();
    let _log_dir = CONFIG.log_dir.clone();

    std::thread::spawn(move || {
        run_suricata();
    });

    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        monitor_logs_and_forward().await;
    });

    tokio::spawn(async move {
        cleanup_old_logs().await;
    });

    info!("🚀 통합 NIDPS 서비스 시작됨 - Suricata와 Axum이 단일 프로세스로 실행 중");
}

async fn start_server(app: Router, port: u16) -> Result<(), Error> {
    let mut current_port = port;
    let max_port_attempts = 10;
    let mut listener = None;

    for attempt in 0..max_port_attempts {
        match tokio::net::TcpListener::bind(format!("0.0.0.0:{}", current_port)).await {
            Ok(l) => {
                listener = Some(l);
                info!("서버 실행 중: http://0.0.0.0:{}", current_port);
                break;
            },
            Err(e) => {
                error!("포트 {} 바인딩 실패 ({}/{}): {}, 다른 포트 시도 중...", current_port, attempt + 1, max_port_attempts, e);
                current_port += 1;
            }
        }
    }

    let listener = match listener {
        Some(l) => l,
        None => {
            let err_msg = format!("포트 {}부터 {}까지 모두 사용 중", port, port + max_port_attempts - 1);
            error!("{}", err_msg);
            return Err(Error::new(io::Error::new(io::ErrorKind::AddrInUse, err_msg)));
        }
    };

    axum::serve(listener, app).await.unwrap_or_else(|e| {
        error!("서버 실행 중 오류 발생: {}", e);
    });
    Ok(())
}

async fn root() -> &'static str {
    "Friede sei mit euch!"
}

pub fn run_suricata() {
    info!("Suricata 실행 중...");
    if env::var("SKIP_SURICATA").is_ok() {
        info!("SKIP_SURICATA 설정 감지됨. Suricata 실행 생략");
        return;
    }

    let config_path = env::var("SURICATA_CONFIG_PATH").unwrap_or_else(|_| String::from("/etc/suricata/suricata.yaml"));
    let interface = env::var("SURICATA_INTERFACE").unwrap_or_else(|_| String::from("eth0"));

    let mut cmd = Command::new("suricata");
    cmd.args(&[
        "-c", &config_path,
        "-i", &interface,
        "--af-packet",
        "--runmode=autofp",
        "--set=af-packet.0.copy-mode=ips"
    ]);

    match cmd.spawn() {
        Ok(child) => info!("Suricata 실행됨: {:?}", child),
        Err(e) => {
            error!("Suricata 실행 실패: {}. 개발 환경에서는 무시 가능", e);
        }
    }
}

async fn cleanup_old_logs() {
    info!("로그 정리 태스크 시작");
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(CONFIG.cleanup_interval)).await;

        let last_time = match LAST_PROCESSED_TIME.lock() {
            Ok(guard) => *guard,
            Err(e) => {
                error!("LAST_PROCESSED_TIME lock 실패: {}", e);
                SystemTime::now()
            }
        };
        clean_old_log_files(&CONFIG.log_dir, &CONFIG.active_log_file, last_time).await;
    }
}

async fn clean_old_log_files(log_dir: &str, active_log_file: &str, last_time: SystemTime) {
    match fs::read_dir(log_dir) {
        Ok(entries) => {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path != Path::new(active_log_file) {
                    process_log_file(&path, active_log_file, last_time).await;
                }
            }
        },
        Err(e) => {
            error!("로그 디렉토리 읽기 실패: {}", e);
        }
    }
}

async fn process_log_file(path: &Path, active_log_file: &str, last_time: SystemTime) {
    if let Some(file_name) = path.file_name() {
        let file_name = file_name.to_string_lossy();
        if file_name.starts_with("eve.") && file_name.ends_with(".json") && file_name != "eve.json" {
            if let Ok(metadata) = fs::metadata(path) {
                if let Ok(modified_time) = metadata.modified() {
                    if modified_time < last_time {
                        info!("오래된 로그 삭제: {}", file_name);
                        if let Err(e) = fs::remove_file(path) {
                            error!("파일 삭제 실패: {}", e);
                        }
                    }
                }
            }
        }
    }
}

pub async fn monitor_logs_and_forward() {
    info!("Suricata 로그 모니터링 시작 ({})", CONFIG.active_log_file);

    let mut last_rotation_check = SystemTime::now();
    let check_interval = Duration::from_secs(30);

    loop {
        let now = SystemTime::now();
        if now.duration_since(last_rotation_check).unwrap_or_default() > check_interval {
            check_log_rotation().await;
            last_rotation_check = now;
        }

        if let Err(e) = monitor_log_file().await {
            error!("로그 모니터링 오류: {}", e);
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(CONFIG.log_watch_interval)).await;
    }
}

async fn monitor_log_file() -> Result<(), io::Error> {
    match File::open(&CONFIG.active_log_file).await {
        Ok(file) => {
            let reader = BufReader::new(file);
            let client = Client::new();
            let mut lines = reader.lines();

            while let Ok(Some(line)) = lines.next_line().await {
                process_log_line(&line, &CONFIG.central_api_server_url, &client).await;
            }
            Ok(())
        },
        Err(e) => {
            error!("로그 파일 열기 실패: {}", e);
            Err(e)
        }
    }
}

async fn process_log_line(line: &str, api_url: &str, client: &Client) {
    if let Ok(mut guard) = LAST_PROCESSED_TIME.lock() {
        *guard = SystemTime::now();
    } else {
        error!("LAST_PROCESSED_TIME lock 실패");
    }

    match serde_json::from_str::<Value>(line) {
        Ok(json) => {
            if let Some(event_type) = json.get("event_type") {
                info!("이벤트 발견: {}", event_type);
            }

            let url = format!("{}/log", api_url);
            if let Err(e) = client.post(&url).json(&json).send().await {
                error!("로그 전송 실패: {}", e);
            }
        },
        Err(e) => {
            error!("JSON 파싱 실패: {}", e);
        }
    }
}

async fn check_log_rotation() {
    let max_log_size = CONFIG.max_log_size_mb * 1024 * 1024;

    match fs::metadata(&CONFIG.active_log_file) {
        Ok(metadata) => {
            let size = metadata.len();
            if size > max_log_size {
                rotate_log_file(size, max_log_size).await;
            }
        },
        Err(e) => {
            error!("로그 파일 정보 조회 실패: {}", e);
            create_empty_log_file().await;
        }
    }
}

async fn rotate_log_file(size: u64, max_log_size: u64) {
    info!("로그 로테이션 시작 ({} > {} bytes)", size, max_log_size);

    let now = chrono::Utc::now();
    let timestamp = now.format("%Y%m%d%H%M%S").to_string();

    let file_name = Path::new(&CONFIG.active_log_file)
        .file_name()
        .unwrap_or_else(|| std::ffi::OsStr::new("eve.json"))
        .to_string_lossy();

    let backup_file = format!("{}.{}", file_name, timestamp);
    let backup_path = Path::new(&CONFIG.log_dir).join(backup_file);

    match fs::rename(&CONFIG.active_log_file, &backup_path) {
        Ok(_) => {
            info!("로테이션 완료: {:?}", backup_path);
            create_empty_log_file().await;
        },
        Err(e) => {
            error!("로테이션 실패: {}", e);
        }
    }
}

async fn create_empty_log_file() {
    if let Some(parent) = Path::new(&CONFIG.active_log_file).parent() {
        if !parent.exists() {
            if let Err(e) = fs::create_dir_all(parent) {
                error!("로그 디렉토리 생성 실패: {}", e);
            }
        }
    }

    match fs::File::create(&CONFIG.active_log_file) {
        Ok(_) => {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Ok(metadata) = fs::metadata(&CONFIG.active_log_file) {
                    let mut perms = metadata.permissions();
                    perms.set_mode(0o666);
                    if let Err(e) = fs::set_permissions(&CONFIG.active_log_file, perms) {
                        error!("파일 권한 설정 실패: {}", e);
                    }
                }
            }
            info!("새 로그 파일 생성됨: {}", CONFIG.active_log_file);
        },
        Err(e) => {
            error!("빈 로그 파일 생성 실패: {}", e);
        }
    }
}
