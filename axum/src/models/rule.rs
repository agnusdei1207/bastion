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



#[derive(Serialize)]
pub struct RuleInfo {
    pub id: String,
    pub content: String,
    pub sid: Option<String>,
    pub msg: Option<String>,
    pub action: String,
}

#[derive(Serialize)]
pub struct RulesListResponse {
    pub success: bool,
    pub rules: Vec<RuleInfo>,
    pub count: usize,
}

 
#[derive(Serialize)]
pub struct RuleDetailResponse {
    pub success: bool,
    pub rule_content: Option<String>,
    pub rule_id: Option<String>,
    pub message: Option<String>,
}