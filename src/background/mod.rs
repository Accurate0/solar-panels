use crate::{
    get_average_for_last_n_minutes,
    goodwe::{self, GoodWeSemsAPI},
    weather::{self, WeatherAPI},
};
use futures::FutureExt;
use serde::Serialize;
use sqlx::PgPool;
use std::{panic::AssertUnwindSafe, time::Duration};

#[derive(Clone)]
pub struct BackgroundTask {
    pool: PgPool,
    solar_api: GoodWeSemsAPI,
    weather_api: WeatherAPI,
    http_client: reqwest::Client,
}

#[derive(thiserror::Error, Debug)]
pub enum BackgroundTaskError {
    #[error("a http error occurred: {0}")]
    SolarAPI(#[from] goodwe::GoodWeSemsAPIError),
    #[error("unknown error occurred: {0}")]
    WeatherAPI(#[from] weather::WeatherAPIError),
    #[error("a database error occurred: {0}")]
    Database(#[from] sqlx::Error),
    #[error("a http error occurred: {0}")]
    Http(#[from] reqwest::Error),
    #[error("unknown error occurred: {0}")]
    Unknown(#[from] anyhow::Error),
}

#[derive(Serialize)]
pub struct SolarIngestAvgPayload {
    pub mins_15: Option<f64>,
    pub mins_60: Option<f64>,
    pub mins_180: Option<f64>,
}

#[derive(Serialize)]
pub struct SolarIngestPayload {
    pub current_kwh: f64,
    pub average_kwh: SolarIngestAvgPayload,
    pub uv_level: Option<f64>,
}

impl BackgroundTask {
    pub fn new(pool: PgPool, solar_api: GoodWeSemsAPI, weather_api: WeatherAPI) -> Self {
        Self {
            pool,
            solar_api,
            weather_api,
            http_client: reqwest::ClientBuilder::new().build().unwrap(),
        }
    }

    pub fn start(&self) -> tokio::task::JoinHandle<()> {
        let self_cloned = self.clone();
        tokio::spawn(async move {
            loop {
                let self_cloned = self_cloned.clone();
                let fut = async move {
                    tracing::info!("fetching data");
                    let login_data = self_cloned.solar_api.get_new_or_cached_login_data().await?;
                    let solar_data = self_cloned.solar_api.get_solar_data(login_data).await?;

                    let kwh = solar_data.data.kpi.pac;
                    let raw_data = serde_json::to_value(&solar_data).unwrap();

                    tracing::info!("fetched solar data: {kwh}");

                    let uv_level = self_cloned
                        .weather_api
                        .get_uv_level(WeatherAPI::PERTH_NAME)
                        .await;

                    if let Err(ref e) = uv_level {
                        tracing::error!("error getting uv level: {e}");
                    }

                    let uv_level = uv_level.ok();
                    tracing::info!("fetched uv level: {uv_level:?}");

                    let weather_details = self_cloned
                        .weather_api
                        .get_weather_details(WeatherAPI::JANDAKOT_GEOCODE)
                        .await;

                    if let Err(ref e) = weather_details {
                        tracing::error!("error getting weather details: {e}");
                    }

                    let current_temperature = weather_details.ok().map(|w| w.data.temp);
                    tracing::info!("fetched weather details: {current_temperature:?}");

                    sqlx::query!(
                        "INSERT INTO solar_data_tsdb (current_kwh, raw_data, uv_level, temperature) VALUES ($1, $2, $3, $4)",
                        kwh,
                        raw_data,
                        uv_level,
                        current_temperature
                    )
                    .execute(&self_cloned.pool)
                    .await?;

                    if let Some(home_gateway_api_base) = std::env::var("HOME_GATEWAY_BASE_URL").ok()
                    {
                        let url = format!("{home_gateway_api_base}/v1/ingest/solar");
                        let api_key =
                            std::env::var("HOME_GATEWAY_API_KEY").unwrap_or_else(|_| "".to_owned());

                        // FIXME: expensive
                        let avg_15_mins =
                            get_average_for_last_n_minutes(15, &self_cloned.solar_api).await?;
                        let avg_1_hour =
                            get_average_for_last_n_minutes(60, &self_cloned.solar_api).await?;
                        let avg_3_hours =
                            get_average_for_last_n_minutes(180, &self_cloned.solar_api).await?;

                        let response = self_cloned
                            .http_client
                            .post(url)
                            .header("X-Api-Key", api_key)
                            .json(&SolarIngestPayload {
                                current_kwh: kwh,
                                average_kwh: SolarIngestAvgPayload {
                                    mins_15: avg_15_mins,
                                    mins_60: avg_1_hour,
                                    mins_180: avg_3_hours,
                                },
                                uv_level,
                            })
                            .send()
                            .await?;

                        tracing::info!("home-gateway response: {}", response.status());
                    };

                    Ok::<(), BackgroundTaskError>(())
                };

                if let Err(e) = AssertUnwindSafe(fut).catch_unwind().await {
                    tracing::error!("error fetching data: {e:?}");
                }

                tokio::time::sleep(Duration::from_secs(60)).await
            }
        })
    }
}
