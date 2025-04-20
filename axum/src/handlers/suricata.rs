
use tokio::process::Command;


// 수리카타 상태 확인
pub async fn get_suricata_status() -> Result<String, String> {
    // 컨테이너 환경에서는 다음과 같이 구현할 수 있음
    // 1. suricatasc 명령어로 직접 리로드
    let output = Command::new("docker")
        .args(&["exec", "suricata", "suricatasc", "-c", "uptime"])
        .output()
        .await
        .map_err(|e| format!("Failed to execute command: {}", e))?;
    
    if !output.status.success() {
        return Err(format!("Command failed with status: {}", output.status));
    }
    
    let uptime = String::from_utf8_lossy(&output.stdout);
    Ok(uptime.to_string())
}

// 수리카타 규칙 리로드
pub async fn reload_suricata_rules() -> Result<(), String> {
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
    
    Ok(())
}

// 수리카타 규칙 통계 확인
pub async fn get_suricata_rule_stats() -> Result<String, String> {
    // 컨테이너 환경에서는 다음과 같이 구현할 수 있음
    // 1. suricatasc 명령어로 직접 리로드
    let output = Command::new("docker")
        .args(&["exec", "suricata", "suricatasc", "-c", "ruleset-stats"])
        .output()
        .await
        .map_err(|e| format!("Failed to execute command: {}", e))?;
    
    if !output.status.success() {
        return Err(format!("Command failed with status: {}", output.status));
    }
    
    let stats = String::from_utf8_lossy(&output.stdout);
    Ok(stats.to_string())
}