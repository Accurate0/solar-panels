use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

pub struct SavedSolarData {
    pub raw_data: PlantDetailsByPowerStationIdResponse,
    pub temperature: Option<f64>,
    pub uv_level: Option<f64>,
}

#[derive(serde::Serialize)]
pub struct LoginRequest {
    pub account: String,
    pub pwd: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginResponse {
    pub has_error: bool,
    pub code: i64,
    pub msg: String,
    pub data: LoginData,
    pub components: LoginComponents,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginData {
    pub uid: String,
    pub timestamp: i64,
    pub token: String,
    pub client: String,
    pub version: String,
    pub language: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginComponents {
    pub para: Value,
    pub lang_ver: i64,
    pub time_span: i64,
    pub api: Option<String>,
    pub msg_socket_adr: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlantDetailsByPowerStationIdResponse {
    pub data: PlantDetailsData,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlantDetailsData {
    pub kpi: Kpi,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Kpi {
    #[serde(rename = "month_generation")]
    pub month_generation: f64,
    pub pac: f64,
    pub power: f64,
    #[serde(rename = "total_power")]
    pub total_power: f64,
    #[serde(rename = "day_income")]
    pub day_income: f64,
    #[serde(rename = "total_income")]
    pub total_income: f64,
    #[serde(rename = "yield_rate")]
    pub yield_rate: f64,
    pub currency: String,
}
