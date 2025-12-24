use crate::{
    goodwe::{self, GoodWeSemsAPI},
    weather::{self, WeatherAPI},
};
use futures::FutureExt;
use sqlx::PgPool;
use std::{panic::AssertUnwindSafe, time::Duration};

#[derive(Clone)]
pub struct BackgroundTask {
    pool: PgPool,
    solar_api: GoodWeSemsAPI,
    weather_api: WeatherAPI,
}

#[derive(thiserror::Error, Debug)]
pub enum BackgroundTaskError {
    #[error("a http error occurred: {0}")]
    SolarAPI(#[from] goodwe::GoodWeSemsAPIError),
    #[error("unknown error occurred: {0}")]
    WeatherAPI(#[from] weather::WeatherAPIError),
    #[error("a database error occurred: {0}")]
    Database(#[from] sqlx::Error),
    #[error("unknown error occurred: {0}")]
    Unknown(#[from] anyhow::Error),
}

impl BackgroundTask {
    pub fn new(pool: PgPool, solar_api: GoodWeSemsAPI, weather_api: WeatherAPI) -> Self {
        Self {
            pool,
            solar_api,
            weather_api,
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
