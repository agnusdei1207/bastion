use axum::{
    http::StatusCode,
    Json,
};
use reqwest;
use dotenvy::dotenv;
use std::env;
use tracing::{error, warn};

use crate::models::eve_json_log::EveJsonLog;

pub async fn send_eve_json_log(
    Json(new_post): Json<EveJsonLog>
) -> Result<Json<EveJsonLog>, (StatusCode, String)> {
    dotenv().ok();

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

    // 4. 외부 API 호출
    let response: reqwest::Response = match client
        .post(&external_api_url)
        .json(&new_post)
        .send()
        .await
    {
        Ok(res) => res,
        Err(e) => {
            error!("외부 API 요청 실패: {}", e);
            return Err((
                StatusCode::BAD_GATEWAY,
                "업스트림 서버 연결 실패".to_string(),
            ));
        }
    };

    if !response.status().is_success() {
        let status = response.status();
        warn!("외부 API 오류 응답: {}", status);
        return Err((
            StatusCode::BAD_GATEWAY,
            format!("업스트림 서버 오류: {}", status),
        ));
    }

    match response.json::<EveJsonLog>().await {
        Ok(post) => Ok(Json(post)),
        Err(e) => {
            error!("JSON 파싱 실패: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "응답 데이터 처리 실패".to_string(),
            ))
        }
    }
}