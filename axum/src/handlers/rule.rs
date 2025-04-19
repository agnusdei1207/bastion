use std::path::Path;
use std::fs::{self, OpenOptions, File};
use std::io::{Write, Read, Seek, SeekFrom};
use axum::{
    extract::{Json, Path as PathExtractor},
    http::StatusCode,
    response::IntoResponse,
};
use tracing::{error, info};
use crate::models::rule::{RuleDetailResponse, RuleInfo, RuleRequest, RulesListResponse};
use crate::models::rule::RuleResponse;
use tokio::process::Command;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

// 룰 추가 핸들러
pub async fn add_rule(Json(payload): Json<RuleRequest>) -> impl IntoResponse {
    let rules_dir = "/var/lib/suricata/rules";
    let filename = "custom.rules";

    let file_path = Path::new(rules_dir).join(filename);

    // 룰 유효성 검증
    if let Err(validation_error) = validate_rule_syntax(&payload.rule_content) {
        return (
            StatusCode::BAD_REQUEST,
            Json(RuleResponse {
                success: false,
                message: validation_error,
                rule_id: None,
            })
        );
    }

    // 디렉토리 존재 확인
    if let Some(parent) = file_path.parent() {
        if !parent.exists() {
            if let Err(e) = fs::create_dir_all(parent) {
                error!("Failed to create directory: {}", e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(RuleResponse {
                        success: false,
                        message: format!("Failed to create directory: {}", e),
                        rule_id: None,
                    })
                );
            }
        }
    }

    // 파일이 너무 크면 백업
    let backup_if_needed = || -> Result<(), String> {
        if let Ok(metadata) = fs::metadata(&file_path) {
            // 파일이 1MB보다 크면 백업 생성
            if metadata.len() > 1_000_000 {
                let backup_path = file_path.with_extension("rules.bak");
                fs::copy(&file_path, &backup_path)
                    .map_err(|e| format!("Failed to create backup: {}", e))?;
                info!("Created backup of rules file at {:?}", backup_path);
            }
        }
        Ok(())
    };
    
    if let Err(e) = backup_if_needed() {
        error!("{}", e);
        // 백업 실패해도 계속 진행
    }

    // 룰 파일에 추가
    let mut file = match OpenOptions::new()
        .create(true)
        .append(true)
        .open(&file_path) {
        Ok(file) => file,
        Err(e) => {
            error!("Failed to open file: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(RuleResponse {
                    success: false,
                    message: format!("Failed to open file: {}", e),
                    rule_id: None,
                })
            );
        }
    };

    // 룰 내용 준비
    let mut rule_content = payload.rule_content.clone();
    if !rule_content.ends_with('\n') {
        rule_content.push('\n');
    }

    // 파일에 룰 작성
    if let Err(e) = file.write_all(rule_content.as_bytes()) {
        error!("Failed to write to file: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(RuleResponse {
                success: false,
                message: format!("Failed to write to file: {}", e),
                rule_id: None,
            })
        );
    }

    // 변경사항 즉시 디스크에 반영
    if let Err(e) = file.sync_all() {
        error!("Failed to sync file to disk: {}", e);
        // 이미 쓰기는 성공했으므로 경고만 로그로 남김
    }

    // 성공 응답
    let rule_id = generate_rule_id(&rule_content);
    info!("Rule added successfully: {} with ID: {}", rule_content, rule_id);
    
    // 추가된 규칙에 대해 Suricata 규칙 리로드 명령 실행
    match reload_suricata_rules().await {
        Ok(_) => (
            StatusCode::CREATED,
            Json(RuleResponse {
                success: true,
                message: "Rule added and applied successfully".to_string(),
                rule_id: Some(rule_id),
            })
        ),
        Err(e) => (
            StatusCode::PARTIAL_CONTENT,
            Json(RuleResponse {
                success: true,
                message: format!("Rule added but reload failed: {}", e),
                rule_id: Some(rule_id),
            })
        )
    }
}

// 룰 삭제 핸들러
pub async fn delete_rule(PathExtractor(rule_id): PathExtractor<String>) -> impl IntoResponse {
    let rules_dir = "/var/lib/suricata/rules";
    let filename = "custom.rules";

    let file_path = Path::new(rules_dir).join(filename);
    
    // 파일이 존재하는지 확인
    if !file_path.exists() {
        return (
            StatusCode::NOT_FOUND,
            Json(RuleResponse {
                success: false,
                message: "Rules file does not exist".to_string(),
                rule_id: None,
            })
        );
    }
    
    // 파일 읽기
    let mut content = String::new();
    match File::open(&file_path) {
        Ok(mut file) => {
            if let Err(e) = file.read_to_string(&mut content) {
                error!("Failed to read rules file: {}", e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(RuleResponse {
                        success: false,
                        message: format!("Failed to read rules file: {}", e),
                        rule_id: None,
                    })
                );
            }
        },
        Err(e) => {
            error!("Failed to open rules file: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(RuleResponse {
                    success: false,
                    message: format!("Failed to open rules file: {}", e),
                    rule_id: None,
                })
            );
        }
    }
    
    // 각 룰을 줄바꿈으로 분리
    let rules: Vec<&str> = content.split('\n').collect();
    
    // 룰 ID와 일치하는 룰 찾기
    let mut found_rule = false;
    let new_content: Vec<String> = rules.into_iter()
        .filter(|rule| {
            if rule.trim().is_empty() {
                return true; // 빈 라인은 유지
            }
            let current_id = generate_rule_id(rule);
            if current_id == rule_id {
                found_rule = true;
                false // 삭제할 룰은 필터링
            } else {
                true // 다른 룰은 유지
            }
        })
        .map(|s| s.to_string())
        .collect();
    
    if !found_rule {
        return (
            StatusCode::NOT_FOUND,
            Json(RuleResponse {
                success: false,
                message: format!("Rule with ID '{}' not found", rule_id),
                rule_id: None,
            })
        );
    }
    
    // 변경된 내용을 파일에 다시 쓰기
    match File::create(&file_path) {
        Ok(mut file) => {
            let new_content = new_content.join("\n");
            if let Err(e) = file.write_all(new_content.as_bytes()) {
                error!("Failed to write updated rules: {}", e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(RuleResponse {
                        success: false,
                        message: format!("Failed to write updated rules: {}", e),
                        rule_id: None,
                    })
                );
            }
            // 변경사항 즉시 디스크에 반영
            if let Err(e) = file.sync_all() {
                error!("Failed to sync file to disk: {}", e);
                // 쓰기는 성공했으므로 계속 진행
            }
        },
        Err(e) => {
            error!("Failed to create rules file: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(RuleResponse {
                    success: false,
                    message: format!("Failed to create rules file: {}", e),
                    rule_id: None,
                })
            );
        }
    }
    
    // 룰 제거 후 Suricata 리로드
    info!("Rule with ID {} removed successfully", rule_id);
    
    match reload_suricata_rules().await {
        Ok(_) => (
            StatusCode::OK,
            Json(RuleResponse {
                success: true,
                message: "Rule deleted and changes applied successfully".to_string(),
                rule_id: Some(rule_id),
            })
        ),
        Err(e) => (
            StatusCode::PARTIAL_CONTENT,
            Json(RuleResponse {
                success: true,
                message: format!("Rule deleted but reload failed: {}", e),
                rule_id: Some(rule_id),
            })
        )
    }
}

// 모든 룰 목록 조회 핸들러
pub async fn list_rules() -> impl IntoResponse {
    let rules_dir = "/var/lib/suricata/rules";
    let filename = "custom.rules";
    let file_path = Path::new(rules_dir).join(filename);
    
    // 파일이 존재하는지 확인
    if !file_path.exists() {
        return (
            StatusCode::OK,
            Json(RulesListResponse {
                success: true,
                rules: Vec::new(),
                count: 0,
            })
        );
    }
    
    // 파일 읽기
    let mut content = String::new();
    match File::open(&file_path) {
        Ok(mut file) => {
            if let Err(e) = file.read_to_string(&mut content) {
                error!("Failed to read rules file: {}", e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(RulesListResponse {
                        success: false,
                        rules: Vec::new(),
                        count: 0,
                    })
                );
            }
        },
        Err(e) => {
            error!("Failed to open rules file: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(RulesListResponse {
                    success: false,
                    rules: Vec::new(),
                    count: 0,
                })
            );
        }
    }
    
    // 각 룰 파싱
    let mut rules = Vec::new();
    for line in content.split('\n') {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        
        let id = generate_rule_id(line);
        
        // sid 추출
        let sid = extract_option(line, "sid");
        
        // msg 추출
        let msg = extract_option(line, "msg");
        
        // action 추출 (alert, drop 등)
        let action = line.split_whitespace().next().unwrap_or("unknown").to_string();
        
        rules.push(RuleInfo {
            id,
            content: line.to_string(),
            sid,
            msg,
            action,
        });
    }
    
    (
        StatusCode::OK,
        Json(RulesListResponse {
            success: true,
            count: rules.len(),
            rules,
        })
    )
}

// 특정 ID의 룰 상세 조회 핸들러 (단순화 버전)
pub async fn get_rule_one_by_id(PathExtractor(rule_id): PathExtractor<String>) -> impl IntoResponse {
    let rules_dir = "/var/lib/suricata/rules";
    let filename = "custom.rules";
    let file_path = Path::new(rules_dir).join(filename);
    
    // 파일이 존재하는지 확인
    if !file_path.exists() {
        return (
            StatusCode::NOT_FOUND,
            Json(RuleDetailResponse {
                success: false,
                rule_content: None,
                rule_id: None,
                message: Some("Rules file does not exist".to_string()),
            })
        );
    }
    
    // 파일 읽기
    let mut content = String::new();
    match File::open(&file_path) {
        Ok(mut file) => {
            if let Err(e) = file.read_to_string(&mut content) {
                error!("Failed to read rules file: {}", e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(RuleDetailResponse {
                        success: false,
                        rule_content: None,
                        rule_id: None,
                        message: Some(format!("Failed to read rules file: {}", e)),
                    })
                );
            }
        },
        Err(e) => {
            error!("Failed to open rules file: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(RuleDetailResponse {
                    success: false,
                    rule_content: None,
                    rule_id: None,
                    message: Some(format!("Failed to open rules file: {}", e)),
                })
            );
        }
    }
    
    // 각 룰 검색
    for line in content.split('\n') {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        
        let current_id = generate_rule_id(line);
        
        // 요청된 ID와 일치하는지 확인
        if current_id == rule_id {
            return (
                StatusCode::OK,
                Json(RuleDetailResponse {
                    success: true,
                    rule_content: Some(line.to_string()),
                    rule_id: Some(rule_id),
                    message: None,
                })
            );
        }
    }
    
    // 룰을 찾지 못한 경우
    (
        StatusCode::NOT_FOUND,
        Json(RuleDetailResponse {
            success: false,
            rule_content: None,
            rule_id: None,
            message: Some(format!("Rule with ID '{}' not found", rule_id)),
        })
    )
}

// 옵션 추출 헬퍼 함수
fn extract_option(rule: &str, option_name: &str) -> Option<String> {
    if let Some(options_part) = rule.split('(').nth(1) {
        if let Some(options_end) = options_part.rfind(')') {
            let options = &options_part[..options_end];
            for option in options.split(';') {
                let option = option.trim();
                if option.starts_with(&format!("{}:", option_name)) {
                    let value_start = option_name.len() + 1; // +1 for ":"
                    if option.len() > value_start {
                        let value = &option[value_start..];
                        // 따옴표 처리
                        if value.starts_with('"') && value.ends_with('"') && value.len() >= 2 {
                            return Some(value[1..value.len()-1].to_string());
                        }
                        return Some(value.to_string());
                    }
                }
            }
        }
    }
    None
}

// Suricata 규칙 검증 함수
fn validate_rule_syntax(rule: &str) -> Result<(), String> {
    // 1. 기본 형식 검증: action, header, options 구조 확인
    let rule = rule.trim();
    
    // 빈 규칙 확인
    if rule.is_empty() {
        return Err("Rule cannot be empty".to_string());
    }
    
    // 주석 규칙은 항상 유효
    if rule.starts_with('#') {
        return Ok(());
    }
    
    // 2. 기본 구조 검증: 'action proto src_ip src_port -> dst_ip dst_port (options)'
    let parts: Vec<&str> = rule.split('(').collect();
    if parts.len() != 2 {
        return Err("Rule must contain header and options parts separated by '('".to_string());
    }
    
    let header = parts[0].trim();
    let options = parts[1].trim();
    
    // 3. 헤더 부분 검증
    let header_parts: Vec<&str> = header.split_whitespace().collect();
    if header_parts.len() < 7 {
        return Err("Header must contain at least: action, proto, src_ip, src_port, direction, dst_ip, dst_port".to_string());
    }
    
    // 4. 액션 검증
    let action = header_parts[0];
    match action {
        "alert" | "drop" | "reject" | "pass" | "log" => {},
        _ => return Err(format!("Invalid action: {}. Must be one of: alert, drop, reject, pass, log", action)),
    }
    
    // 5. 방향 연산자 검증
    if header_parts[4] != "->" && header_parts[4] != "<>" {
        return Err(format!("Invalid direction operator: {}. Must be -> or <>", header_parts[4]));
    }
    
    // 6. 옵션 부분 검증
    if !options.ends_with(')') {
        return Err("Options must end with ')'".to_string());
    }
    
    // 옵션 내용 검증 (괄호 제거)
    let options = &options[..options.len() - 1];
    let option_parts: Vec<&str> = options.split(';').collect();
    
    // 최소 하나의 옵션은 있어야 함
    if option_parts.is_empty() || option_parts[0].trim().is_empty() {
        return Err("At least one option is required".to_string());
    }
    
    // 7. 필수 옵션 검증: sid, msg
    let has_sid = option_parts.iter().any(|opt| opt.trim().starts_with("sid:"));
    let has_msg = option_parts.iter().any(|opt| opt.trim().starts_with("msg:"));
    
    if !has_sid {
        return Err("Missing required option: sid".to_string());
    }
    
    if !has_msg {
        return Err("Missing required option: msg".to_string());
    }
    
    // 8. sid 형식 검증 (숫자여야 함)
    for opt in option_parts {
        let opt = opt.trim();
        if opt.starts_with("sid:") {
            let sid_value = opt[4..].trim();
            if sid_value.parse::<u64>().is_err() {
                return Err(format!("Invalid sid format: {}. Must be a number", sid_value));
            }
        }
    }
    
    Ok(())
}

// Suricata 규칙 리로드 함수
async fn reload_suricata_rules() -> Result<(), String> {
    // 컨테이너 환경에서는 다음과 같이 구현할 수 있음
    // 1. suricatasc 명령어로 직접 리로드
    let output = Command::new("docker")
        .args(&["exec", "suricata", "suricatasc", "-c", "reload-rules"])
        .output()
        .await
        .map_err(|e| format!("Failed to execute reload command: {}", e))?;
        
    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to reload rules: {}", error));
    }
    
    info!("Suricata rules reloaded successfully");
    Ok(())
}

// 규칙 ID 생성 함수
fn generate_rule_id(rule_content: &str) -> String {
    let mut hasher = DefaultHasher::new();
    rule_content.hash(&mut hasher);
    format!("rule_{:x}", hasher.finish())
}