use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct RuleRequest {
    pub rule_content: String,
    pub rule_type: String, // "ids" 또는 "ips"
    pub filename: Option<String>, // 저장할 파일명 (옵션)
}

#[derive(Debug, Serialize)]
pub struct RuleResponse {
    pub success: bool,
    pub message: String,
    pub rule_id: Option<String>,
}
