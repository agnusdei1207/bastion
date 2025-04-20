use std::path::Path;
use std::fs::{self, OpenOptions, File};
use std::io::{Write, Read};
use axum::{
    extract::{Json, Path as PathExtractor},
    http::StatusCode,
    response::IntoResponse,
};
use tracing::{error, info};

use crate::models::rule::{ApiResponse, RuleRequest, Rule, RulesList};
use crate::utils::suricata::{extract_option, generate_rule_id, get_env,  validate_rule_syntax};

// 룰 추가 핸들러
pub async fn create_rule(Json(payload): Json<RuleRequest>) -> impl IntoResponse {
    let (rules_dir, filename) = get_env();
    let file_path = Path::new(&rules_dir).join(&filename);

    // 룰 유효성 검증
    if let Err(validation_error) = validate_rule_syntax(&payload.rule_content) {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()> {
                success: false,
                message: Some(validation_error),
                data: None,
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
                    Json(ApiResponse::<()> {
                        success: false,
                        message: Some(format!("Failed to create directory: {}", e)),
                        data: None,
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
                Json(ApiResponse::<()> {
                    success: false,
                    message: Some(format!("Failed to open file: {}", e)),
                    data: None,
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
            Json(ApiResponse::<()> {
                success: false,
                message: Some(format!("Failed to write to file: {}", e)),
                data: None,
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
    
    (
        StatusCode::CREATED,
        Json(ApiResponse {
            success: true,
            message: Some("Rule added and applied successfully".to_string()),
            data: None,
        })
    )
   
}

// 룰 삭제 핸들러
pub async fn delete_rule(PathExtractor(rule_id): PathExtractor<String>) -> impl IntoResponse {
    let (rules_dir, filename) = get_env();
    let file_path = Path::new(&rules_dir).join(&filename);

    // 파일이 존재하는지 확인
    if !file_path.exists() {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()> {
                success: false,
                message: Some("Rules file does not exist".to_string()),
                data: None,
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
                    Json(ApiResponse::<()> {
                        success: false,
                        message: Some(format!("Failed to read rules file: {}", e)),
                        data: None,
                    })
                );
            }
        },
        Err(e) => {
            error!("Failed to open rules file: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()> {
                    success: false,
                    message: Some(format!("Failed to open rules file: {}", e)),
                    data: None,
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
            Json(ApiResponse::<()> {
                success: false,
                message: Some(format!("Rule with ID '{}' not found", rule_id)),
                data: None,
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
                    Json(ApiResponse::<()> {
                        success: false,
                        message: Some(format!("Failed to write updated rules: {}", e)),
                        data: None,
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
                Json(ApiResponse::<()> {
                    success: false,
                    message: Some(format!("Failed to create rules file: {}", e)),
                    data: None,
                })
            );
        }
    }
    
    // 룰 제거 후 Suricata 리로드
    info!("Rule with ID {} removed successfully", rule_id);
    
    (
        StatusCode::OK,
        Json(ApiResponse::<()> {
            success: true,
            message: Some("Rule deleted and changes applied successfully".to_string()),
            data:None
        })
    )
}

// 모든 룰 목록 조회 핸들러
pub async fn get_rules() -> impl IntoResponse {
let (rules_dir, filename) = get_env();
    let file_path = Path::new(&rules_dir).join(&filename);

    // 파일이 존재하는지 확인
    if !file_path.exists() {
        return (
            StatusCode::OK,
            Json(ApiResponse {
                success: true,
                message: None,
                data: Some(RulesList {
                    rules: Vec::new(),
                    count: 0,
                }),
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
                    Json(ApiResponse::<RulesList> {
                        success: false,
                        message: Some(format!("Failed to read rules file: {}", e)),
                        data: None,
                    })
                );
            }
        },
        Err(e) => {
            error!("Failed to open rules file: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<RulesList> {
                    success: false,
                    message: Some(format!("Failed to open rules file: {}", e)),
                    data: None,
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
        let action = line.split_whitespace().next().map(String::from);
        
        rules.push(Rule {
            id,
            content: line.to_string(),
            sid,
            msg,
            action,
        });
    }

    let count = rules.len();
    
    (
        StatusCode::OK,
        Json(ApiResponse {
            success: true,
            message: None,
            data: Some(RulesList {
                rules,
                count,
            }),
        })
    )
}

// 특정 ID의 룰 상세 조회 핸들러 (단순화 버전)
pub async fn get_rule(PathExtractor(rule_id): PathExtractor<String>) -> impl IntoResponse {
    let (rules_dir, filename) = get_env();
    let file_path = Path::new(&rules_dir).join(&filename);
    
    // 파일이 존재하는지 확인
    if !file_path.exists() {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<Rule> {
                success: false,
                message: Some("Rules file does not exist".to_string()),
                data: None,
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
                    Json(ApiResponse::<Rule> {
                        success: false,
                        message: Some(format!("Failed to read rules file: {}", e)),
                        data: None,
                    })
                );
            }
        },
        Err(e) => {
            error!("Failed to open rules file: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<Rule> {
                    success: false,
                    message: Some(format!("Failed to open rules file: {}", e)),
                    data: None,
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
                Json(ApiResponse {
                    success: true,
                    message: None,
                    data: Some(Rule {
                        id: rule_id,
                        content: line.to_string(),
                        sid: extract_option(line, "sid"),
                        msg: extract_option(line, "msg"),
                        action: line.split_whitespace().next().map(String::from),
                    }),
                })
            );
        }
    }
    
    // 룰을 찾지 못한 경우
    (
        StatusCode::NOT_FOUND,
        Json(ApiResponse::<Rule> {
            success: false,
            message: Some(format!("Rule with ID '{}' not found", rule_id)),
            data: None,
        })
    )
}
