use std::collections::HashMap;

use axum::http::{HeaderMap, HeaderValue};
use base64::{Engine, prelude::BASE64_STANDARD};
use reqwest::{
    Method,
    header::{ACCEPT, CONTENT_TYPE},
};
use sqlx::PgPool;
use tracing::instrument;
use types::{
    LoginData, LoginRequest, LoginResponse, PlantDetailsByPowerStationIdResponse, SavedSolarData,
};

pub mod types;

#[derive(Clone, Debug)]
pub struct GoodWeSemsAPI {
    db: PgPool,
    username: String,
    password: String,
    powerstation_id: String,
    http: reqwest::Client,
}

const LOGIN_URL: &str = "https://www.semsportal.com/api/v2/Common/CrossLogin";
const GET_POWERSTATION_DETAILS_URL: &str =
    "https://au.semsportal.com/api/v3/PowerStation/GetPlantDetailByPowerstationId";

#[derive(thiserror::Error, Debug)]
pub enum GoodWeSemsAPIError {
    #[error("a http error occurred: {0}")]
    Http(#[from] reqwest::Error),
    #[error("a database error occurred: {0}")]
    Database(#[from] sqlx::Error),
}

impl GoodWeSemsAPI {
    pub fn db(&self) -> &PgPool {
        &self.db
    }

    pub fn new(db: PgPool, username: String, password: String, powerstation_id: String) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());
        headers.insert(ACCEPT, "application/json".parse().unwrap());

        Self {
            db,
            username,
            password,
            powerstation_id,
            http: reqwest::ClientBuilder::new()
                .default_headers(headers)
                .build()
                .expect("must build http client"),
        }
    }

    pub async fn get_latest_saved_solar_data(&self) -> Result<SavedSolarData, GoodWeSemsAPIError> {
        let solar_data = sqlx::query!(
            "SELECT raw_data, temperature, uv_level FROM solar_data_tsdb ORDER BY time DESC LIMIT 1"
        )
        .fetch_one(&self.db)
        .await?;

        Ok(SavedSolarData {
            raw_data: serde_json::from_value(solar_data.raw_data).unwrap(),
            temperature: solar_data.temperature,
            uv_level: solar_data.uv_level,
        })
    }

    #[instrument(skip(self))]
    pub async fn get_solar_data(
        &self,
        login: LoginData,
    ) -> Result<PlantDetailsByPowerStationIdResponse, GoodWeSemsAPIError> {
        let request = self
            .http
            .request(Method::POST, GET_POWERSTATION_DETAILS_URL)
            .form(&{
                let mut map = HashMap::new();
                map.insert("powerStationId", &self.powerstation_id);
                map
            })
            .header(
                "token",
                BASE64_STANDARD.encode(serde_json::to_string(&login).unwrap()),
            )
            .build()?;

        let response = self
            .http
            .execute(request)
            .await?
            .error_for_status()?
            .json::<PlantDetailsByPowerStationIdResponse>()
            .await?;

        Ok(response)
    }

    #[instrument(skip(self))]
    pub async fn get_new_or_cached_login_data(&self) -> Result<LoginData, GoodWeSemsAPIError> {
        let latest_login_data =
            sqlx::query!("SELECT * FROM cached_token ORDER BY created_at DESC LIMIT 1")
                .fetch_optional(&self.db)
                .await?;

        let now = chrono::offset::Utc::now().naive_utc();
        if let Some(latest_login) = latest_login_data {
            if (now - latest_login.created_at).num_minutes() > 5 {
                self.login_and_save().await
            } else {
                tracing::info!("fetched existing login data");
                Ok(serde_json::from_value::<LoginData>(latest_login.login_data).unwrap())
            }
        } else {
            self.login_and_save().await
        }
    }

    async fn login_and_save(&self) -> Result<LoginData, GoodWeSemsAPIError> {
        let response = self.login().await?;
        let login_data = serde_json::to_value(&response.data).unwrap();
        sqlx::query!(
            "INSERT INTO cached_token (login_data) VALUES ($1)",
            login_data
        )
        .execute(&self.db)
        .await?;

        tracing::info!("saved login info");

        Ok(response.data)
    }

    pub async fn login(&self) -> Result<LoginResponse, GoodWeSemsAPIError> {
        let request = self
            .http
            .request(Method::POST, LOGIN_URL)
            .json(&LoginRequest {
                account: self.username.clone(),
                pwd: self.password.clone(),
            }).header(
            "token",
            // base64 of
            // {"uid":"","timestamp":0,"token":"","client":"web","version":"","language":"en"}
            "eyJ1aWQiOiIiLCJ0aW1lc3RhbXAiOjAsInRva2VuIjoiIiwiY2xpZW50Ijoid2ViIiwidmVyc2lvbiI6IiIsImxhbmd1YWdlIjoiZW4ifQ=="
                .parse::<HeaderValue>()
                .unwrap(),
        )
            .build()?;

        let response = self
            .http
            .execute(request)
            .await?
            .error_for_status()?
            .json::<LoginResponse>()
            .await?;

        Ok(response)
    }
}
