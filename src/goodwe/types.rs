use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

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
    pub api: String,
    pub msg_socket_adr: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlantDetailsByPowerStationIdResponse {
    pub language: String,
    pub function: Value,
    pub has_error: bool,
    pub msg: String,
    pub code: String,
    pub data: PlantDetailsData,
    pub components: PlantDetailsComponents,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlantDetailsData {
    pub info: Info,
    pub kpi: Kpi,
    pub is_evcharge: bool,
    pub is_tigo: bool,
    pub is_powerflow: bool,
    pub is_sec: bool,
    pub is_genset: bool,
    pub is_micro_grid: bool,
    pub is_micro_inverter: bool,
    pub has_layout: bool,
    #[serde(rename = "layout_id")]
    pub layout_id: String,
    pub is_meter: bool,
    pub is_environmental: bool,
    #[serde(rename = "powercontrol_status")]
    pub powercontrol_status: i64,
    pub charts_types_by_plant: Vec<ChartsTypesByPlant>,
    pub soc: Vec<Value>,
    pub industry_soc: Vec<Value>,
    #[serde(rename = "isSec1000EtPlant")]
    pub is_sec1000et_plant: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Info {
    #[serde(rename = "powerstation_id")]
    pub powerstation_id: String,
    pub time: String,
    #[serde(rename = "date_format_ym")]
    pub date_format_ym: String,
    pub stationname: String,
    pub address: String,
    #[serde(rename = "battery_capacity")]
    pub battery_capacity: f64,
    #[serde(rename = "create_time")]
    pub create_time: String,
    pub capacity: f64,
    #[serde(rename = "powerstation_type")]
    pub powerstation_type: String,
    pub status: i64,
    #[serde(rename = "is_stored")]
    pub is_stored: bool,
    #[serde(rename = "only_bps")]
    pub only_bps: bool,
    #[serde(rename = "only_bpu")]
    pub only_bpu: bool,
    #[serde(rename = "time_span")]
    pub time_span: f64,
    #[serde(rename = "org_code")]
    pub org_code: String,
    #[serde(rename = "org_name")]
    pub org_name: String,
    #[serde(rename = "local_date")]
    pub local_date: String,
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

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChartsTypesByPlant {
    pub date: String,
    pub type_name: String,
    pub chart_indices: Vec<ChartIndice>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChartIndice {
    pub index_name: String,
    pub index_label: String,
    pub chart_index_id: String,
    pub date_range: Vec<DateRange>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DateRange {
    pub text: String,
    pub value: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub now: String,
    pub date_formater: Value,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlantDetailsComponents {
    pub para: String,
    pub lang_ver: i64,
    pub time_span: i64,
    pub api: String,
    pub msg_socket_adr: Value,
}
