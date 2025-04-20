use std::{env, hash::{DefaultHasher, Hash, Hasher}};

use dotenvy::dotenv;

// 환경변수 조회
pub fn get_env() -> (String, String) {
    dotenv().ok();
    let rules_dir = env::var("SURICATA_RULES_DIR").unwrap_or_else(|_| "/var/lib/suricata/rules".to_string());
    let default_filename = env::var("SURICATA_CUSTOM_RULE_FILENAME").unwrap_or_else(|_| "custom.rules".to_string());
    (rules_dir, default_filename)
}


// Suricata 규칙 검증 함수
pub fn validate_rule_syntax(rule: &str) -> Result<(), String> {
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


// 규칙 ID 생성 함수
pub fn generate_rule_id(rule_content: &str) -> String {
    let mut hasher = DefaultHasher::new();
    rule_content.hash(&mut hasher);
    format!("rule_{:x}", hasher.finish())
}


// 옵션 추출 헬퍼 함수
pub fn extract_option(rule: &str, option_name: &str) -> Option<String> {
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
