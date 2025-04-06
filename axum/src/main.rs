use std::process::Command;
use std::time::{SystemTime, Duration};
use std::fs;
use std::path::Path;
use std::env;
use std::io;

use axum::{
     routing::get, Error, Router
};
use routes::routes;
use tracing::{info, Level};
use tracing_subscriber;
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
    pub port: u16,
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
        
        // 기본 로그 디렉토리를 현재 사용자의 홈 디렉토리로 변경
        let default_log_dir = match env::var("HOME") {
            Ok(home) => format!("{}/suricata_logs", home),
            Err(_) => String::from("./suricata_logs"), // 홈 디렉토리를 찾을 수 없는 경우 현재 디렉토리
        };
        
        let log_dir = env::var("LOG_DIR").unwrap_or_else(|_| default_log_dir);
        
        // active_log_file이 직접 지정되지 않으면 log_dir 기반으로 만듦
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
        
        // 생성된 환경변수 확인
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
    // .env 파일 로드 (dotenvy 사용)
    dotenv().ok();
    
    // 로깅 설정
    setup_logging();

    // 설정 로드 및 출력
    let configs = load_and_print_configs().await;
    
    // 로그 디렉토리 생성
    ensure_log_directory_exists(&configs.log_dir).await;

    // 서버 라우팅 설정
    let app = setup_routes();

    // 백그라운드 작업 시작
    start_background_tasks(&configs).await;

    // 서버 시작
    start_server(app, configs.port).await?;
 
    Ok(())
}

// 로깅 설정
fn setup_logging() {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();
}

// 설정 로드 및 출력
async fn load_and_print_configs() -> Config {
    let config = CONFIG.lock().await;
    info!("환경 설정 로드됨:");
    info!("중앙 API 서버: {}", config.central_api_server_url);
    info!("로그 감시 주기: {}초", config.log_watch_interval);
    info!("로그 정리 주기: {}초", config.cleanup_interval);
    info!("로그 디렉토리: {}", config.log_dir);
    info!("활성 로그 파일: {}", config.active_log_file);
    info!("최대 로그 크기: {}MB", config.max_log_size_mb);
    info!("서버 포트: {}", config.port);
    
    config.clone()
}

// 로그 디렉토리 생성
async fn ensure_log_directory_exists(log_dir: &str) -> Result<(), io::Error> {
    if let Err(e) = fs::create_dir_all(log_dir) {
        info!("로그 디렉토리 생성 실패: {}. 직접 생성해 주세요: {}", e, log_dir);
        return Err(e);
    }
    info!("로그 디렉토리 생성 또는 확인됨: {}", log_dir);
    Ok(())
}

// 서버 라우팅 설정
fn setup_routes() -> Router {
    Router::new()
        .route("/", get(root))
        .merge(routes())
        .layer(create_cors())
}

// 백그라운드 작업 시작
async fn start_background_tasks(configs: &Config) {
    // 설정 복제
    let active_log_file = configs.active_log_file.clone();
    let log_dir = configs.log_dir.clone();
    
    // Suricata 프로세스 실행 - 별도 스레드
    std::thread::spawn(move || {
        run_suricata();
    });
    
    // Suricata 로그 모니터링 - 별도 비동기 태스크
    let watch_config = configs.clone();
    tokio::spawn(async move {
        // 로그 파일이 생성될 때까지 짧게 대기
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        
        // 로그 모니터링 시작
        monitor_logs_and_forward(watch_config).await;
    });

    // 로그 정리 태스크 시작
    let cleanup_config = configs.clone();
    tokio::spawn(async move {
        cleanup_old_logs(cleanup_config).await;
    });

    info!("🚀 통합 NIDPS 서비스 시작됨 - Suricata와 Axum이 단일 프로세스로 실행 중");
}

// 서버 시작
async fn start_server(app: Router, port: u16) -> Result<(), Error> {
    // 포트가 사용 중인 경우 대체 포트 시도
    let mut current_port = port;
    let max_port_attempts = 10; // 최대 10개 포트 시도
    let mut listener = None;

    for attempt in 0..max_port_attempts {
        match tokio::net::TcpListener::bind(format!("0.0.0.0:{}", current_port)).await {
            Ok(l) => {
                listener = Some(l);
                info!("서버가 http://0.0.0.0:{} 에서 실행 중입니다", current_port);
                break;
            },
            Err(e) => {
                info!("포트 {}에 바인딩 실패({}/{}): {}, 다른 포트 시도...", 
                     current_port, attempt + 1, max_port_attempts, e);
                current_port += 1; // 다음 포트 시도
            }
        }
    }

    // 리스너를 얻지 못했으면 오류 반환
    let listener = match listener {
        Some(l) => l,
        None => {
            return Err(Error::new(io::Error::new(
                io::ErrorKind::AddrInUse,
                format!("포트 {} - {}까지 모두 사용 중", port, port + max_port_attempts - 1)
            )));
        }
    };

    axum::serve(listener, app).await.unwrap();
    Ok(())
}
 
async fn root() -> &'static str {
    "Friede sei mit euch!"
}


pub fn run_suricata() {
    info!("Suricata 실행 중...");
    
    // 개발 환경에서는 Suricata 실행을 건너뛸 수 있는 옵션 제공
    if env::var("SKIP_SURICATA").is_ok() {
        info!("환경 변수 SKIP_SURICATA가 설정되어 Suricata 실행을 건너뜁니다");
        return;
    }
    
    // 환경 변수에서 Suricata 설정 파일 경로 및 인터페이스 확인
    let config_path = env::var("SURICATA_CONFIG_PATH").unwrap_or_else(|_| String::from("/etc/suricata/suricata.yaml"));
    let interface = env::var("SURICATA_INTERFACE").unwrap_or_else(|_| String::from("eth0"));
    
    // Suricata 실행 명령어와 매개변수 설정
    let mut cmd = Command::new("suricata");
    cmd.args(&[
        "-c", &config_path,
        "-i", &interface,
        "--af-packet",
        "--runmode=autofp",
        "--set=af-packet.0.copy-mode=ips"
    ]);
    
    match cmd.spawn() {
        Ok(child) => {
            info!("Suricata 실행됨: {:?}", child);
        },
        Err(e) => {
            info!("Suricata 실행 실패: {}. 개발 환경에서는 정상입니다.", e);
            info!("로그 모니터링은 계속 수행합니다.");
        }
    }
}

// 주기적으로 오래된 로그 파일 정리
async fn cleanup_old_logs(config: Config) {
    info!("로그 정리 태스크 시작");
    
    loop {
        // 환경 변수로 지정된 시간만큼 대기
        tokio::time::sleep(tokio::time::Duration::from_secs(config.cleanup_interval)).await;
        
        let last_time = *LAST_PROCESSED_TIME.lock().await;
        info!("로그 정리 수행 중...");
        
        clean_old_log_files(&config.log_dir, &config.active_log_file, last_time).await;
    }
}

// 오래된 로그 파일 삭제
async fn clean_old_log_files(log_dir: &str, active_log_file: &str, last_time: SystemTime) {
    // 로그 디렉토리에서 모든 파일 읽기 (디렉토리 자체는 유지)
    match fs::read_dir(log_dir) {
        Ok(entries) => {
            for entry in entries.flatten() {
                let path = entry.path();
                
                // 현재 활성 로그 파일과 디렉토리는 건드리지 않음
                if path.is_file() && path != Path::new(active_log_file) {
                    process_log_file(&path, active_log_file, last_time).await;
                }
            }
        },
        Err(e) => {
            info!("로그 디렉토리 읽기 실패: {}", e);
        }
    }
}

// 개별 로그 파일 처리
async fn process_log_file(path: &Path, active_log_file: &str, last_time: SystemTime) {
    // 아카이브된 로그 파일인지 확인 (패턴: eve.TIMESTAMP.json)
    if let Some(file_name) = path.file_name() {
        let file_name = file_name.to_string_lossy();
        if file_name.starts_with("eve.") && file_name.ends_with(".json") && file_name != "eve.json" {
            // 파일 수정 시간 확인
            if let Ok(metadata) = fs::metadata(path) {
                if let Ok(modified_time) = metadata.modified() {
                    // 마지막으로 처리한 시간보다 이전 파일만 삭제
                    if modified_time < last_time {
                        info!("처리 완료된 로그 파일 삭제: {}", file_name);
                        if let Err(e) = fs::remove_file(path) {
                            info!("파일 삭제 실패: {}", e);
                        }
                    }
                }
            }
        }
    }
}

// 로그 모니터링 및 전송
pub async fn monitor_logs_and_forward(config: Config) {
    info!("Suricata 로그 모니터링 시작 ({})", config.active_log_file);
    
    let mut last_rotation_check = SystemTime::now();
    let check_interval = Duration::from_secs(30); // 30초마다 로그 파일 크기 확인
    
    loop {
        // 현재 시간이 마지막 체크 시간 + 인터벌보다 크면 로그 로테이션 체크
        let now = SystemTime::now();
        if now.duration_since(last_rotation_check).unwrap_or_default() > check_interval {
            check_log_rotation(&config).await;
            last_rotation_check = now;
        }
        
        // 로그 파일 모니터링
        if let Err(e) = monitor_log_file(&config).await {
            info!("로그 모니터링 오류: {}", e);
        }
        
        // 재시도 전 잠시 대기 - 환경 변수에서 가져온 시간만큼 대기
        tokio::time::sleep(tokio::time::Duration::from_secs(config.log_watch_interval)).await;
    }
}

// 로그 파일 모니터링
async fn monitor_log_file(config: &Config) -> Result<(), io::Error> {
    match File::open(&config.active_log_file).await {
        Ok(file) => {
            let reader = BufReader::new(file);
            let client = Client::new();
            let mut lines = reader.lines();
            
            info!("로그 파일 열림, 모니터링 중...");

            // 라인 단위로 처리
            while let Ok(Some(line)) = lines.next_line().await {
                process_log_line(&line, &config.central_api_server_url, &client).await;
            }
            
            // 파일 읽기가 끝나면 잠시 대기 후 재시도
            info!("로그 파일의 끝에 도달. 잠시 후 다시 시도합니다.");
            Ok(())
        },
        Err(e) => {
            info!("로그 파일을 열 수 없음: {}. 재시도 중...", e);
            Err(e)
        }
    }
}

// 개별 로그 라인 처리
async fn process_log_line(line: &str, api_url: &str, client: &Client) {
    // 라인 처리 시작 - 이벤트 시간 기록
    *LAST_PROCESSED_TIME.lock().await = SystemTime::now();
    
    // line이 유효한 JSON인 경우 처리
    match serde_json::from_str::<Value>(line) {
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

// 로그 파일이 너무 커지면 로테이션
async fn check_log_rotation(config: &Config) {
    let max_log_size = config.max_log_size_mb * 1024 * 1024; // MB -> bytes
    
    // 현재 활성 로그 파일 확인
    match fs::metadata(&config.active_log_file) {
        Ok(metadata) => {
            let size = metadata.len();
            
            // 파일이 최대 크기를 초과하면 로테이션
            if size > max_log_size {
                rotate_log_file(config, size, max_log_size).await;
            }
        },
        Err(e) => {
            info!("로그 파일 정보를 가져오는 데 실패: {}", e);
            
            // 로그 파일이 없으면 빈 파일 생성
            create_empty_log_file(config).await;
        }
    }
}

// 로그 파일 로테이션 수행
async fn rotate_log_file(config: &Config, size: u64, max_log_size: u64) {
    info!("로그 파일이 최대 크기({}MB)를 초과함, 로테이션 수행", max_log_size / 1024 / 1024);
    
    // 새 이름으로 현재 파일 이동 (타임스탬프 사용)
    let now = chrono::Utc::now();
    let timestamp = now.format("%Y%m%d%H%M%S").to_string();
    
    // 로그 디렉토리에서 파일 이름만 추출
    let file_name = Path::new(&config.active_log_file)
        .file_name()
        .unwrap_or_else(|| std::ffi::OsStr::new("eve.json"))
        .to_string_lossy();
    
    // 파일 이름과 타임스탬프를 결합하여 백업 경로 생성
    let backup_file = format!("{}.{}", file_name, timestamp);
    let backup_path = Path::new(&config.log_dir).join(backup_file);
    
    // 원자적으로 이름 변경 시도
    match fs::rename(&config.active_log_file, &backup_path) {
        Ok(_) => {
            info!("로그 로테이션 완료: {} -> {:?}", config.active_log_file, backup_path);
            
            // 빈 파일 생성
            create_empty_log_file(config).await;
        },
        Err(e) => {
            info!("로그 파일 로테이션 실패: {}", e);
        }
    }
}

// 빈 로그 파일 생성
async fn create_empty_log_file(config: &Config) {
    // 디렉토리가 존재하는지 확인
    if let Some(parent) = Path::new(&config.active_log_file).parent() {
        if !parent.exists() {
            let _ = fs::create_dir_all(parent);
        }
    }
    
    if let Err(create_err) = fs::File::create(&config.active_log_file) {
        info!("로그 파일 생성 실패: {}", create_err);
    } else {
        info!("새 로그 파일 생성됨: {}", config.active_log_file);
        
        // 권한 설정
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(metadata) = fs::metadata(&config.active_log_file) {
                let mut perms = metadata.permissions();
                perms.set_mode(0o666);  // rw-rw-rw-
                let _ = fs::set_permissions(&config.active_log_file, perms);
            }
        }
    }
}

// Config 구현시 Clone 트레이트 추가
impl Clone for Config {
    fn clone(&self) -> Self {
        Config {
            central_api_server_url: self.central_api_server_url.clone(),
            log_watch_interval: self.log_watch_interval,
            cleanup_interval: self.cleanup_interval,
            log_dir: self.log_dir.clone(),
            active_log_file: self.active_log_file.clone(),
            max_log_size_mb: self.max_log_size_mb,
            port: self.port,
        }
    }
}