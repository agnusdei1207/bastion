use std::process::Command;

use axum::{
     routing::get, Error, Router
};
use routes::routes;
use tracing::{info, Level};
use tracing_subscriber;

mod handlers;
mod models;
mod routes;
mod cors;


use tokio::{fs::File, io::{AsyncBufReadExt, BufReader}};
use serde_json::Value;
use reqwest::Client;


use crate::cors::cors::create_cors;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    let app: Router = Router::new()
        .route("/", get(root))
        .merge(routes())
        .layer(
            create_cors()
        );

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
    let result = Command::new("suricata")
        .args(&["-c", "/etc/suricata/suricata.yaml", "-i", "eth0"])
        .spawn()
        .expect("Failed to launch Suricata");
    
    info!("Suricata 실행됨: {:?}", result);
}

pub async fn monitor_logs_and_forward() {
    info!("Suricata 로그 모니터링 시작 (/var/log/suricata/eve.json)");
    
    loop {
        match File::open("/var/log/suricata/eve.json").await {
            Ok(file) => {
                let reader = BufReader::new(file);
                let client = Client::new();
                let mut lines = reader.lines();
                
                info!("로그 파일 열림, 모니터링 중...");

                // 수정된 부분: next_line()은 Result<Option<String>, Error>를 반환
                while let Ok(Some(line)) = lines.next_line().await {
                    // line이 유효한 문자열인 경우에만 처리
                    match serde_json::from_str::<Value>(&line) {
                        Ok(json) => {
                            // json 값이 범위 내에 있도록 수정
                            if let Some(event_type) = json.get("event_type") {
                                info!("로그 이벤트 발견: {}", event_type);
                            } else {
                                info!("로그 이벤트 발견: 타입 없음");
                            }
                            
                            // 내부 로그 처리 API로 전송
                            match client.post("http://127.0.0.1:3000/log").json(&json).send().await {
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
        
        // 파일을 열 수 없거나 읽기가 완료된 경우 잠시 대기 후 다시 시도
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    }
}