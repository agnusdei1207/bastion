use axum::{
    http::StatusCode,
    Json,
};
use reqwest;
use dotenvy::dotenv;
use std::env;
use tracing::{error, warn, info};
use serde_json::Value;

use crate::models::eve_json_log::EveJsonLog;

pub async fn send_eve_json_log(
    body: axum::body::Body
) -> Result<Json<Value>, (StatusCode, String)> {
    dotenv().ok();
    
    // 요청 본문을 문자열로 변환
    let bytes = match axum::body::to_bytes(body, usize::MAX).await {
        Ok(bytes) => bytes,
        Err(e) => {
            error!("요청 본문을 읽지 못함: {}", e);
            return Err((
                StatusCode::BAD_REQUEST,
                "요청 본문을 읽을 수 없습니다".to_string(),
            ));
        }
    };

    
    // 문자열을 JSON으로 파싱
    let json_data: Value = match serde_json::from_slice(&bytes) {
        Ok(val) => val,
        Err(e) => {
            error!("JSON 파싱 실패: {}", e);
            return Err((
                StatusCode::BAD_REQUEST,
                "유효하지 않은 JSON 형식입니다".to_string(),
            ));
        }
    };
    
    // JSON 배열 또는 단일 객체 처리
    let items = match &json_data {
        Value::Array(items) => {
            info!("배열로 된 이벤트 로그를 수신했습니다. 항목 수: {}", items.len());
            items.clone()
        },
        Value::Object(_) => {
            info!("단일 객체로 된 이벤트 로그를 수신했습니다.");
            vec![json_data.clone()]
        },
        _ => {
            error!("지원하지 않는 JSON 형식: 배열이나 객체가 아닙니다");
            return Err((
                StatusCode::BAD_REQUEST,
                "지원하지 않는 JSON 형식입니다. 배열이나 객체여야 합니다".to_string(),
            ));
        }
    };
    
    let external_api_url = match env::var("CENTRAL_API_SERVER_URL") {
        Ok(url) => url,
        Err(e) => {
            error!("환경 변수 읽기 실패: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "서버 구성 오류".to_string(),
            ));
        }
    };

    let client = reqwest::Client::new();
    let mut results = Vec::new();
    
    // 각 이벤트 항목을 개별적으로 처리
    for item in items {
        // 항목을 EveJsonLog로 변환 시도
        let log_item = match serde_json::from_value::<EveJsonLog>(item.clone()) {
            Ok(log) => log,
            Err(e) => {
                warn!("항목을 EveJsonLog로 변환 실패: {}", e);
                // 에러 로깅 후 다음 항목 처리
                continue;
            }
        };

        info!("EveJsonLog 항목: {:?}", log_item);
        
        // 외부 API 호출
        let response: reqwest::Response = match client
            .post(&external_api_url)
            .json(&log_item)
            .send()
            .await {
                Ok(res) => res,
                Err(e) => {
                    error!("외부 API 호출 실패: {}", e);
                    continue;  // 에러 로깅 후 다음 항목 처리
                }
            };

        if !response.status().is_success() {
            let status = response.status();
            warn!("외부 API 오류 응답: {}", status);
            // 에러 로깅 후 다음 항목 처리
            continue;
        }

        match response.json::<Value>().await {
            Ok(result) => results.push(result),
            Err(e) => {
                error!("응답 JSON 파싱 실패: {}", e);
                continue;  // 에러 로깅 후 다음 항목 처리
            }
        }
    }
    
    // 처리 결과 반환
    if results.is_empty() {
        Err((
            StatusCode::BAD_GATEWAY,
            "모든 이벤트 처리에 실패했습니다".to_string(),
        ))
    } else {
        let response = if results.len() == 1 {
            results[0].clone()  // 단일 결과인 경우
        } else {
            Value::Array(results)  // 여러 결과인 경우 배열로 반환
        };
        
        Ok(Json(response))
    }
}