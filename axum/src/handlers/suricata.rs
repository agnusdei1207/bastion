use tokio::process::Command;
use tracing::{error, info};

// 수리카타 상태 확인
pub async fn get_suricata_status() -> Result<String, String> {
    // 컨테이너 환경에서는 다음과 같이 구현할 수 있음
    let output = match Command::new("docker")
        .args(&["exec", "suricata", "suricatasc", "-c", "uptime"])
        .output()
        .await {
            Ok(output) => output,
            Err(e) => {
                error!("Failed to execute Suricata status command: {}", e);
                return Err(format!("Failed to execute command: {}", e));
            }
        };
    
    if !output.status.success() {
        let stderr = match std::str::from_utf8(&output.stderr) {
            Ok(s) => s.to_string(),
            Err(_) => "Failed to decode stderr output".to_string(),
        };
        error!("Suricata status command failed: {}", stderr);
        return Err(format!("Command failed with status: {} ({})", output.status, stderr));
    }
    
    match std::str::from_utf8(&output.stdout) {
        Ok(uptime) => Ok(uptime.to_string()),
        Err(e) => {
            error!("Failed to decode Suricata output: {}", e);
            Err("Failed to decode command output".to_string())
        }
    }
}

// 수리카타 규칙 리로드
pub async fn reload_suricata_rules() -> Result<(), String> {
    // 컨테이너 환경에서는 다음과 같이 구현할 수 있음
    let output = match Command::new("docker")
        .args(&["exec", "suricata", "suricatasc", "-c", "reload-rules"])
        .output()
        .await {
            Ok(output) => output,
            Err(e) => {
                error!("Failed to execute reload command: {}", e);
                return Err(format!("Failed to execute reload command: {}", e));
            }
        };
    
    if !output.status.success() {
        let error = match std::str::from_utf8(&output.stderr) {
            Ok(s) => s.to_string(),
            Err(_) => "Failed to decode stderr output".to_string(),
        };
        error!("Failed to reload Suricata rules: {}", error);
        return Err(format!("Failed to reload rules: {}", error));
    }
    
    info!("Successfully reloaded Suricata rules");
    Ok(())
}

// 수리카타 규칙 통계 확인
pub async fn get_suricata_rule_statistics() -> Result<String, String> {
    // 컨테이너 환경에서는 다음과 같이 구현할 수 있음
    let output = match Command::new("docker")
        .args(&["exec", "suricata", "suricatasc", "-c", "ruleset-stats"])
        .output()
        .await {
            Ok(output) => output,
            Err(e) => {
                error!("Failed to execute statistics command: {}", e);
                return Err(format!("Failed to execute command: {}", e));
            }
        };
    
    if !output.status.success() {
        let stderr = match std::str::from_utf8(&output.stderr) {
            Ok(s) => s.to_string(),
            Err(_) => "Failed to decode stderr output".to_string(),
        };
        error!("Rule statistics command failed: {}", stderr);
        return Err(format!("Command failed with status: {} ({})", output.status, stderr));
    }
    
    match std::str::from_utf8(&output.stdout) {
        Ok(stats) => Ok(stats.to_string()),
        Err(e) => {
            error!("Failed to decode statistics output: {}", e);
            Err("Failed to decode command output".to_string())
        }
    }
}

// 인터페이스 통계 확인
pub async fn get_interface_statistics() -> Result<String, String> {
    dotenvy::dotenv().ok();
    let interface = std::env::var("NETWORK_INTERFACE").unwrap_or_else(|_| "eth0".to_string());
    
    // 특수 문자 검증 (명령어 주입 방지)
    if interface.contains(|c: char| !c.is_alphanumeric() && c != '_' && c != '-') {
        return Err("Invalid interface name".to_string());
    }
    
    let output = match Command::new("docker")
        .args(&["exec", "suricata", "suricatasc", "-c", &format!("iface-stat {}", interface)])
        .output()
        .await {
            Ok(output) => output,
            Err(e) => {
                error!("Failed to execute interface stats command: {}", e);
                return Err(format!("Failed to execute command: {}", e));
            }
        };
    
    if !output.status.success() {
        let stderr = match std::str::from_utf8(&output.stderr) {
            Ok(s) => s.to_string(),
            Err(_) => "Failed to decode stderr output".to_string(),
        };
        error!("Interface stats command failed: {}", stderr);
        return Err(format!("Command failed with status: {} ({})", output.status, stderr));
    }
    
    match std::str::from_utf8(&output.stdout) {
        Ok(stats) => Ok(stats.to_string()),
        Err(e) => {
            error!("Failed to decode interface stats output: {}", e);
            Err("Failed to decode command output".to_string())
        }
    }
}