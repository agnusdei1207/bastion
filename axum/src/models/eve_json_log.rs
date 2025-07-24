use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct EveJsonLog {
    #[serde(flatten)]  // 모든 필드를 동적으로 확장
    pub extra_fields: HashMap<String, Value>,
    #[serde(default)]  // 없으면 기본값
    pub tags: Vec<String>,
}
