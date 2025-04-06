use std::process::Command;
use std::time::{SystemTime, Duration};
use std::fs;
use std::path::Path;
use std::env;

use axum::{
     routing::get, Error, Router
};
use routes::routes;
use tracing::{info, Level};
use tracing_subscriber;
// dotenv 대신 dotenvy 사용
use dotenvy::dotenv;

mod handlers;
mod models;
mod routes;
mod cors;


use tokio::{fs::File, io::{AsyncBufReadExt, BufReader}, sync::Mutex};
use serde_json::Value;
use reqwest::Client;


use crate::cors::cors::create_cors;

// 마지막으로 처리한 시간 저장
lazy_static::lazy_static! {
    pub static ref LAST_PROCESSED_TIME: Mutex<SystemTime> = Mutex::new(SystemTime::now());
    
    // 환경 변수에서 설정 값 로드
    pub static ref CONFIG: Mutex<Config> = Mutex::new(Config::from_env());
}

// 환경 설정을 담을 구조체
pub struct Config {
    pub central_api_server_url: String,
    pub log_watch_interval: u64,
    pub cleanup_interval: u64,
    pub log_dir: String,
    pub active_log_file: String,
    pub max_log_size_mb: u64,
}

impl Config {
    pub fn from_env() -> Self {
        // 기본값 설정
        let central_api_server_url = env::var("CENTRAL_API_SERVER_URL")
            .unwrap_or_else(|_| String::from("http://127.0.0.1:3000"));
        
        let log_watch_interval = env::var("LOG_WATCH_INTERVAL")
            .ok()
            .and_then(|val| val.parse::<u64>().ok())
            .unwrap_or(5);
        
        let cleanup_interval = env::var("CLEANUP_INTERVAL")
            .ok()
            .and_then(|val| val.parse::<u64>().ok())
            .unwrap_or(60);
        
        let log_dir = env::var("LOG_DIR")
            .unwrap_or_else(|_| String::from("/var/log/suricata"));
        
        // active_log_file이 직접 지정되지 않으면 log_dir 기반으로 만듦
        let active_log_file = env::var("ACTIVE_LOG_FILE")
            .unwrap_or_else(|_| format!("{}/eve.json", log_dir));
        
        let max_log_size_mb = env::var("MAX_LOG_SIZE_MB")
            .ok()
            .and_then(|val| val.parse::<u64>().ok())
            .unwrap_or(100);
        
        Config {
            central_api_server_url,
            log_watch_interval,
            cleanup_interval,
            log_dir,
            active_log_file,
            max_log_size_mb,
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // .env 파일 로드 (dotenvy 사용)
    dotenv().ok();
    
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    // 설정 로드 및 출력
    {
        let config = CONFIG.lock().await;
        info!("환경 설정 로드됨:");
        info!("중앙 API 서버: {}", config.central_api_server_url);
        info!("로그 감시 주기: {}초", config.log_watch_interval);
        info!("로그 정리 주기: {}초", config.cleanup_interval);
        info!("로그 디렉토리: {}", config.log_dir);
        info!("활성 로그 파일: {}", config.active_log_file);
        info!("최대 로그 크기: {}MB", config.max_log_size_mb);
    }

    let app: Router = Router::new()
        .route("/", get(root))
        .merge(routes())
        .layer(
            create_cors()
        );

    // 로그 디렉토리 생성 (없는 경우)
    let config = CONFIG.lock().await;
    if let Err(e) = fs::create_dir_all(&config.log_dir) {
        info!("로그 디렉토리 생성 실패: {}", e);
    }
    drop(config);

    // Suricata 프로세스 실행 - 별도 스레드
    std::thread::spawn(|| {
        run_suricata();
    });
    
    // Suricata 로그 모니터링 - 별도 비동기 태스크
    tokio::spawn(async {
        // 로그 파일이 생성될 때까지 짧게 대기
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        
        // 로그 모니터링 시작
        monitor_logs_and_forward().await;
    });

    // 로그 정리 태스크 시작
    tokio::spawn(async {
        cleanup_old_logs().await;
    });

    info!("🚀 통합 NIDPS 서비스 시작됨 - Suricata와 Axum이 단일 프로세스로 실행 중");

    let listener: tokio::net::TcpListener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    info!("Server is running on http://0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
 
    Ok(())
}
 
async fn root() -> &'static str {
    "Friede sei mit euch!"
}

pub fn run_suricata() {
    info!("Suricata 실행 중...");
    // 환경 변수에서 Suricata 설정 파일 경로 확인
    let config_path = env::var("SURICATA_CONFIG_PATH").unwrap_or_else(|_| String::from("/etc/suricata/suricata.yaml"));
    let interface = env::var("SURICATA_INTERFACE").unwrap_or_else(|_| String::from("eth0"));
    
    let result = Command::new("suricata")
        .args(&["-c", &config_path, "-i", &interface])
        .spawn()
        .expect("Failed to launch Suricata");
    
    info!("Suricata 실행됨: {:?}", result);
}

// 주기적으로 오래된 로그 파일 정리
async fn cleanup_old_logs() {
    info!("로그 정리 태스크 시작");
    
    loop {
        // 환경 변수에서 정리 주기 가져오기
        let config = CONFIG.lock().await;
        let interval = config.cleanup_interval;
        let log_dir = config.log_dir.clone();
        let active_log_file = config.active_log_file.clone();
        drop(config);
        
        tokio::time::sleep(tokio::time::Duration::from_secs(interval)).await;
        
        let last_time = *LAST_PROCESSED_TIME.lock().await;
        info!("로그 정리 수행 중...");
        
        // 로그 디렉토리에서 모든 파일 읽기 (디렉토리 자체는 유지)
        match fs::read_dir(&log_dir) {
            Ok(entries) => {
                for entry in entries.flatten() {
                    let path = entry.path();
                    
                    // 현재 활성 로그 파일과 디렉토리는 건드리지 않음
                    if path.is_file() && path != Path::new(&active_log_file) {
                        // 아카이브된 로그 파일인지 확인 (패턴: eve.TIMESTAMP.json)
                        if let Some(file_name) = path.file_name() {
                            let file_name = file_name.to_string_lossy();
                            if file_name.starts_with("eve.") && file_name.ends_with(".json") && file_name != "eve.json" {
                                // 파일 수정 시간 확인
                                if let Ok(metadata) = fs::metadata(&path) {
                                    if let Ok(modified_time) = metadata.modified() {
                                        // 마지막으로 처리한 시간보다 이전 파일만 삭제
                                        if modified_time < last_time {
                                            info!("처리 완료된 로그 파일 삭제: {}", file_name);
                                            if let Err(e) = fs::remove_file(&path) {
                                                info!("파일 삭제 실패: {}", e);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            },
            Err(e) => {
                info!("로그 디렉토리 읽기 실패: {}", e);
            }
        }
    }
}

pub async fn monitor_logs_and_forward() {
    let config = CONFIG.lock().await;
    let active_log_file = config.active_log_file.clone();
    let watch_interval = config.log_watch_interval;
    let api_url = config.central_api_server_url.clone();
    drop(config);
    
    info!("Suricata 로그 모니터링 시작 ({})", active_log_file);
    
    let mut last_rotation_check = SystemTime::now();
    let check_interval = Duration::from_secs(30); // 30초마다 로그 파일 크기 확인
    
    loop {
        // 현재 시간이 마지막 체크 시간 + 인터벌보다 크면 로그 로테이션 체크
        let now = SystemTime::now();
        if now.duration_since(last_rotation_check).unwrap_or_default() > check_interval {
            check_log_rotation().await;
            last_rotation_check = now;
        }
        
        match File::open(&active_log_file).await {
            Ok(file) => {
                let reader = BufReader::new(file);
                let client = Client::new();
                let mut lines = reader.lines();
                
                info!("로그 파일 열림, 모니터링 중...");

                // 라인 단위로 처리
                while let Ok(Some(line)) = lines.next_line().await {
                    // 라인 처리 시작 - 이벤트 시간 기록
                    *LAST_PROCESSED_TIME.lock().await = SystemTime::now();
                    
                    // line이 유효한 JSON인 경우 처리
                    match serde_json::from_str::<Value>(&line) {
                        Ok(json) => {
                            // 이벤트 타입 로깅
                            if let Some(event_type) = json.get("event_type") {
                                info!("로그 이벤트 발견: {}", event_type);
                            } else {
                                info!("로그 이벤트 발견: 타입 없음");
                            }
                            
                            // API 서버로 전송
                            let url = format!("{}/log", api_url);
                            match client.post(&url).json(&json).send().await {
                                Ok(_) => {
                                    if cfg!(debug_assertions) {
                                        info!("로그 전송 성공");
                                    }
                                },
                                Err(e) => {
                                    info!("로그 전송 실패: {}", e);
                                }
                            }
                        },
                        Err(e) => {
                            info!("JSON 파싱 실패: {}", e);
                        }
                    }
                }
                
                // 파일 읽기가 끝나면 잠시 대기 후 재시도
                info!("로그 파일의 끝에 도달. 잠시 후 다시 시도합니다.");
            },
            Err(e) => {
                info!("로그 파일을 열 수 없음: {}. 재시도 중...", e);
            }
        }
        
        // 재시도 전 잠시 대기 - 환경 변수에서 가져온 시간만큼 대기
        tokio::time::sleep(tokio::time::Duration::from_secs(watch_interval)).await;
    }
}

// 로그 파일이 너무 커지면 로테이션
async fn check_log_rotation() {
    let config = CONFIG.lock().await;
    let max_log_size = config.max_log_size_mb * 1024 * 1024; // MB -> bytes
    let active_log_file = config.active_log_file.clone();
    let log_dir = config.log_dir.clone();
    drop(config);
    
    // 현재 활성 로그 파일 확인
    match fs::metadata(&active_log_file) {
        Ok(metadata) => {
            let size = metadata.len();
            
            // 파일이 최대 크기를 초과하면 로테이션
            if size > max_log_size {
                info!("로그 파일이 최대 크기({}MB)를 초과함, 로테이션 수행", max_log_size / 1024 / 1024);
                
                // 새 이름으로 현재 파일 이동 (타임스탬프 사용)
                let now = chrono::Utc::now();
                let timestamp = now.format("%Y%m%d%H%M%S").to_string();
                
                // 로그 디렉토리에서 파일 이름만 추출
                let file_name = Path::new(&active_log_file)
                    .file_name()
                    .unwrap_or_else(|| std::ffi::OsStr::new("eve.json"))
                    .to_string_lossy();
                
                // 파일 이름과 타임스탬프를 결합하여 백업 경로 생성
                let backup_file = format!("{}.{}", file_name, timestamp);
                let backup_path = Path::new(&log_dir).join(backup_file);
                
                // 원자적으로 이름 변경 시도
                match fs::rename(&active_log_file, &backup_path) {
                    Ok(_) => {
                        info!("로그 로테이션 완료: {} -> {:?}", active_log_file, backup_path);
                        
                        // 빈 파일 생성 - 수리카타가 계속 쓸 수 있도록
                        if let Err(e) = fs::File::create(&active_log_file) {
                            info!("새 로그 파일 생성 실패: {}", e);
                        } else {
                            // 수리카타가 파일에 쓸 수 있도록 퍼미션 조정 (필요한 경우)
                            #[cfg(unix)]
                            {
                                use std::os::unix::fs::PermissionsExt;
                                if let Ok(metadata) = fs::metadata(&active_log_file) {
                                    let mut perms = metadata.permissions();
                                    perms.set_mode(0o666);  // rw-rw-rw-
                                    let _ = fs::set_permissions(&active_log_file, perms);
                                }
                            }
                        }
                    },
                    Err(e) => {
                        info!("로그 파일 로테이션 실패: {}", e);
                    }
                }
            }
        },
        Err(e) => {
            info!("로그 파일 정보를 가져오는 데 실패: {}", e);
            
            // 로그 파일이 없으면 빈 파일 생성
            if !Path::new(&active_log_file).exists() {
                // 디렉토리가 존재하는지 확인
                if let Some(parent) = Path::new(&active_log_file).parent() {
                    if !parent.exists() {
                        let _ = fs::create_dir_all(parent);
                    }
                }
                
                if let Err(create_err) = fs::File::create(&active_log_file) {
                    info!("로그 파일 생성 실패: {}", create_err);
                } else {
                    info!("새 로그 파일 생성됨: {}", active_log_file);
                    
                    // 권한 설정
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        if let Ok(metadata) = fs::metadata(&active_log_file) {
                            let mut perms = metadata.permissions();
                            perms.set_mode(0o666);  // rw-rw-rw-
                            let _ = fs::set_permissions(&active_log_file, perms);
                        }
                    }
                }
            }
        }
    }
}