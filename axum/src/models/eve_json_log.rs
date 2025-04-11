use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct EveJsonLog {
    #[serde(flatten)]  // 모든 필드를 동적으로 확장
    pub extra_fields: HashMap<String, Value>,
    
    // 공통 필드 (옵셔널 처리)
    pub timestamp: Option<String>,
    pub event_type: Option<String>,
    pub src_ip: Option<String>,
    pub dest_ip: Option<String>,

    #[serde(default)]  // 없으면 기본값
    pub tags: Vec<String>,
}
