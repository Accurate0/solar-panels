use tracing::instrument;
use types::{UVXMLDocument, WeatherDetails};

#[derive(Clone, Debug)]
pub struct WeatherAPI {
    http: reqwest::Client,
}

pub mod types;

#[derive(thiserror::Error, Debug)]
pub enum WeatherAPIError {
    #[error("a http error occurred: {0}")]
    Http(#[from] reqwest::Error),
    #[error("a xml error occurred: {0}")]
    Xml(#[from] quick_xml::de::DeError),
    #[error("unknown error occurred: {0}")]
    Unknown(#[from] anyhow::Error),
}

impl WeatherAPI {
    const UV_LEVELS_XML: &str = "https://uvdata.arpansa.gov.au/xml/uvvalues.xml";
    const WEATHER_API_DETAILS: &str = "https://api.weather.bom.gov.au/v1/locations/{}/observations";
    pub const PERTH_NAME: &str = "per";
    pub const JANDAKOT_GEOCODE: &str = "qd63he";

    pub fn new() -> Self {
        Self {
            http: reqwest::ClientBuilder::new().build().unwrap(),
        }
    }

    #[instrument(skip(self))]
    pub async fn get_weather_details(
        &self,
        geocode: &str,
    ) -> Result<WeatherDetails, WeatherAPIError> {
        let weather_details = self
            .http
            .get(Self::WEATHER_API_DETAILS.replace("{}", geocode))
            .send()
            .await?
            .error_for_status()?
            .json::<WeatherDetails>()
            .await?;

        tracing::info!("fetched weather details");

        Ok(weather_details)
    }

    #[instrument(skip(self))]
    pub async fn get_uv_level(&self, name: &str) -> Result<f64, WeatherAPIError> {
        let uv_levels_xml = self
            .http
            .get(Self::UV_LEVELS_XML)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;

        let uv_levels = quick_xml::de::from_str::<UVXMLDocument>(&uv_levels_xml)?;

        tracing::info!("fetched uv level data");

        uv_levels
            .location
            .into_iter()
            .find(|l| l.name == name)
            .map(|l| l.index)
            .ok_or(anyhow::Error::msg("perth not found"))
            .map_err(Into::into)
    }
}
