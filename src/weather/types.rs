use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UVXMLDocument {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    pub location: Vec<Location>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Location {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "$text")]
    pub text: Option<String>,
    pub name: String,
    pub index: f64,
    pub time: String,
    pub date: String,
    pub fulldate: String,
    pub utcdatetime: String,
    pub status: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WeatherDetails {
    pub metadata: WeatherMetadata,
    pub data: WeatherData,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WeatherMetadata {
    #[serde(rename = "response_timestamp")]
    pub response_timestamp: String,
    #[serde(rename = "issue_time")]
    pub issue_time: String,
    #[serde(rename = "observation_time")]
    pub observation_time: String,
    pub copyright: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WeatherData {
    pub temp: f64,
    #[serde(rename = "temp_feels_like")]
    pub temp_feels_like: f64,
    pub wind: Wind,
    pub gust: Gust,
    #[serde(rename = "max_gust")]
    pub max_gust: MaxGust,
    #[serde(rename = "max_temp")]
    pub max_temp: MaxTemp,
    #[serde(rename = "min_temp")]
    pub min_temp: MinTemp,
    #[serde(rename = "rain_since_9am")]
    pub rain_since_9am: i64,
    pub humidity: i64,
    pub station: Station,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Wind {
    #[serde(rename = "speed_kilometre")]
    pub speed_kilometre: i64,
    #[serde(rename = "speed_knot")]
    pub speed_knot: i64,
    pub direction: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Gust {
    #[serde(rename = "speed_kilometre")]
    pub speed_kilometre: i64,
    #[serde(rename = "speed_knot")]
    pub speed_knot: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MaxGust {
    #[serde(rename = "speed_kilometre")]
    pub speed_kilometre: i64,
    #[serde(rename = "speed_knot")]
    pub speed_knot: i64,
    pub time: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MaxTemp {
    pub time: String,
    pub value: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MinTemp {
    pub time: String,
    pub value: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Station {
    #[serde(rename = "bom_id")]
    pub bom_id: String,
    pub name: String,
    pub distance: i64,
}
