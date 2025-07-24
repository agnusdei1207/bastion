use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct RuleRequest {
    pub rule_content: String,
    pub rule_type: Option<String>, 
    pub filename: Option<String>,
}

// 통합된 API 응답 구조체
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

// 기본 룰 정보
#[derive(Debug, Serialize)]
pub struct Rule {
    pub id: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub msg: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
}

// 규칙 목록
#[derive(Debug, Serialize)]
pub struct RulesList {
    pub rules: Vec<Rule>,
    pub count: usize,
}