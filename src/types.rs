use axum::{response::IntoResponse, response::Response};
use chrono::NaiveDateTime;
use reqwest::StatusCode;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SolarCurrentStatisticsAverages {
    pub last_15_mins: f64,
    pub last_1_hour: f64,
    pub last_3_hours: f64,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SolarCurrentStatistics {
    pub averages: SolarCurrentStatisticsAverages,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SolarCurrentResponse {
    pub current_production_wh: f64,
    pub today_production_kwh: f64,
    pub yesterday_production_kwh: f64,
    pub month_production_kwh: f64,
    pub all_time_production_kwh: f64,
    pub statistics: SolarCurrentStatistics,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerationHistory {
    pub wh: f64,
    pub at: NaiveDateTime,
    pub timestamp: i64,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SolarHistoryResponse {
    pub today: Vec<GenerationHistory>,
    pub yesterday: Vec<GenerationHistory>,
}

pub enum AppError {
    Error(anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::Error(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Something went wrong: {}", e),
            )
                .into_response(),
        }
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self::Error(err.into())
    }
}
