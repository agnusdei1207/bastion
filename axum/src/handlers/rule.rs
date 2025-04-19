use std::path::Path;
use std::fs::{self, OpenOptions};
use std::io::Write;
use axum::{
    extract::Json,
    http::StatusCode,
    response::IntoResponse,
};
use tracing::{error, info};
use crate::models::rule::RuleRequest;
use crate::models::rule::RuleResponse;

pub async fn add_rule(
    Json(payload): Json<RuleRequest>
) -> impl IntoResponse {
    let rules_dir = "/var/lib/suricata/rules";
    let filename = "custom.rules";

    // json -> join으로 수정
    let file_path = Path::new(rules_dir).join(filename);

    if !payload.rule_content.contains("alert") &&
       !payload.rule_content.contains("drop") {
        return (
            StatusCode::BAD_REQUEST,
            Json(RuleResponse {
                success: false,
                message: "Rule content must contain 'alert' or 'drop'".to_string(),
                rule_id: None,
            })
        );
    }

    // Parent -> parent로 수정 (대소문자 오류)
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

    // close() -> clone()으로 수정
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

    // 성공 응답
    info!("Rule added successfully: {}", rule_content);
    
    (
        StatusCode::CREATED,
        Json(RuleResponse {
            success: true,
            message: "Rule added successfully".to_string(),
            rule_id: Some(generate_rule_id(&rule_content)),
        })
    )
}

// 규칙 ID 생성 함수 추가 
fn generate_rule_id(rule_content: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    rule_content.hash(&mut hasher);
    format!("rule_{:x}", hasher.finish())
}