use axum::{
    Json,
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    routing::get,
};
use chrono::{FixedOffset, Utc};
use goodwe::{GoodWeSemsAPI, GoodWeSemsAPIError, types::PlantDetailsByPowerStationIdResponse};
use reqwest::Method;
use sqlx::{Connection, postgres::PgPoolOptions, prelude::FromRow};
use std::{future::IntoFuture, ops::Deref, sync::Arc, time::Duration};
use tower::limit::GlobalConcurrencyLimitLayer;
use tower_http::{
    LatencyUnit,
    cors::{AllowHeaders, AllowOrigin, CorsLayer},
    trace::{DefaultOnRequest, DefaultOnResponse, TraceLayer},
};
use tracing::Level;
use tracing_subscriber::{filter::Targets, layer::SubscriberExt, util::SubscriberInitExt};
use twilight_cache_inmemory::{DefaultCacheModels, InMemoryCacheBuilder, ResourceType};
use twilight_gateway::{
    ConfigBuilder, Event, EventType, EventTypeFlags, Intents, Shard, ShardId, StreamExt,
};
use twilight_http::Client as HttpClient;
use twilight_model::{
    application::interaction::InteractionContextType, oauth::ApplicationIntegrationType,
};
use twilight_util::builder::{
    command::CommandBuilder,
    embed::{EmbedBuilder, EmbedFieldBuilder},
};
use types::{
    AppError, GenerationHistory, SolarCurrentResponse, SolarCurrentStatistics,
    SolarCurrentStatisticsAverages, SolarHistoryResponse,
};
use vesper::{
    framework::DefaultError,
    macros::{command, error_handler},
    prelude::{DefaultCommandResult, Framework, SlashContext},
};

mod goodwe;
mod types;

#[derive(Clone)]
struct BotContext(Arc<BotContextInner>);

impl Deref for BotContext {
    type Target = BotContextInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

struct BotContextInner {
    solar_api: GoodWeSemsAPI,
}

async fn handle_event(event: Event, _http: Arc<HttpClient>) -> anyhow::Result<()> {
    match event {
        Event::GatewayHeartbeatAck
        | Event::MessageCreate(_)
        | Event::MessageUpdate(_)
        | Event::MessageDelete(_) => {}
        // Other events here...
        e => {
            tracing::warn!("unhandled event: {e:?}")
        }
    }

    Ok(())
}

#[error_handler]
async fn handle_interaction_error(ctx: &mut SlashContext<BotContext>, error: DefaultError) {
    let fut = async {
        let error = if error.to_string().contains("Missing Access") {
            "This channel is not accessible to the bot...".to_string()
        } else {
            error.to_string()
        };

        let embed = EmbedBuilder::new()
            .title("oops")
            .description(error)
            .color(0xcc6666)
            .validate()?
            .build();

        ctx.interaction_client
            .update_response(&ctx.interaction.token)
            .embeds(Some(&[embed]))
            .await?;

        Ok::<(), anyhow::Error>(())
    };

    if let Err(e) = fut.await {
        tracing::error!("error in updating message: {e:?}");
    }

    tracing::error!("error in interaction: {error:?}");
}

#[command]
#[description = "get latest solar details"]
#[error_handler(handle_interaction_error)]
async fn solar(ctx: &mut SlashContext<BotContext>) -> DefaultCommandResult {
    ctx.defer(false).await?;

    let solar_data = ctx.data.solar_api.get_latest_saved_solar_data().await?;
    let SolarCurrentStatistics { averages } = solar_statistics(&ctx.data.solar_api).await?;

    let embed = EmbedBuilder::new()
        .title("Solar")
        .field(
            EmbedFieldBuilder::new("Current", format!("{} Wh", solar_data.data.kpi.pac)).inline(),
        )
        .field(
            EmbedFieldBuilder::new(
                "Total for today",
                format!("{} kWh", solar_data.data.kpi.power),
            )
            .inline(),
        )
        .field(
            EmbedFieldBuilder::new(
                "All time total",
                format!("{} kWh", solar_data.data.kpi.total_power),
            )
            .inline(),
        )
        .field(
            EmbedFieldBuilder::new("15 min avg", format!("{} Wh", averages.last_15_mins)).inline(),
        )
        .field(
            EmbedFieldBuilder::new("1 hour avg", format!("{} Wh", averages.last_1_hour)).inline(),
        )
        .field(
            EmbedFieldBuilder::new("3 hour avg", format!("{} Wh", averages.last_3_hours)).inline(),
        )
        .color(0x40944c)
        .validate()?
        .build();

    ctx.interaction_client
        .update_response(&ctx.interaction.token)
        .embeds(Some(&[embed]))
        .await?;

    Ok(())
}

pub async fn get_average_for_last_n_minutes(
    s: &'static str,
    solar_api: &GoodWeSemsAPI,
) -> Result<Option<f64>, anyhow::Error> {
    let query = format!(
        "SELECT avg(current_kwh), time_bucket('{s} minutes', time) as time_bucket FROM solar_data_tsdb WHERE (time + '8 hour') > (NOW() + '8 hour') - INTERVAL '{s} MINUTE' GROUP BY time_bucket",
    );

    #[derive(FromRow)]
    struct Row {
        avg: Option<f64>,
    }

    let avg_15_mins: Row = sqlx::query_as(&query).fetch_one(solar_api.db()).await?;

    Ok(avg_15_mins.avg)
}

fn round(n: f64) -> f64 {
    (n * 100.0).round() / 100.0
}

pub async fn solar_statistics(
    solar_api: &GoodWeSemsAPI,
) -> Result<SolarCurrentStatistics, anyhow::Error> {
    let avg_15_mins = get_average_for_last_n_minutes("15", solar_api);
    let avg_1_hour = get_average_for_last_n_minutes("60", solar_api);
    let avg_3_hours = get_average_for_last_n_minutes("180", solar_api);

    let (avg_15_mins, avg_1_hour, avg_3_hours) =
        futures::try_join!(avg_15_mins, avg_1_hour, avg_3_hours)?;

    Ok(SolarCurrentStatistics {
        averages: SolarCurrentStatisticsAverages {
            last_15_mins: round(avg_15_mins.unwrap()),
            last_1_hour: round(avg_1_hour.unwrap()),
            last_3_hours: round(avg_3_hours.unwrap()),
        },
    })
}

async fn solar_current(
    State(ctx): State<BotContext>,
) -> Result<Json<SolarCurrentResponse>, AppError> {
    let resp = ctx.solar_api.get_latest_saved_solar_data().await?;
    let yesterday_results = sqlx::query!(
        "SELECT raw_data FROM solar_data_tsdb WHERE (time + '8 hour')::date = (now() + '8 hour')::date - INTEGER '1' ORDER BY time DESC LIMIT 1"
    )
    .fetch_one(ctx.solar_api.db())
    .await?;

    let yesterday_value =
        serde_json::from_value::<PlantDetailsByPowerStationIdResponse>(yesterday_results.raw_data)?;

    Ok(Json(SolarCurrentResponse {
        yesterday_production_kwh: yesterday_value.data.kpi.power,
        month_production_kwh: resp.data.kpi.month_generation,
        current_production_wh: resp.data.kpi.pac,
        today_production_kwh: resp.data.kpi.power,
        all_time_production_kwh: resp.data.kpi.total_power,
        statistics: solar_statistics(&ctx.solar_api).await?,
    }))
}

// We're gonna need this soon: https://docs.timescale.com/use-timescale/latest/query-data/advanced-analytic-queries/
async fn solar_history(
    State(ctx): State<BotContext>,
) -> Result<Json<SolarHistoryResponse>, AppError> {
    let now = chrono::offset::Utc::now()
        .with_timezone(&chrono_tz::Australia::Perth)
        .fixed_offset();

    let (today, yesterday): (Vec<_>, Vec<_>) = sqlx::query!(
        "SELECT avg(current_kwh), time_bucket('5 minutes', time) as bucket_time FROM solar_data_tsdb WHERE (time + '8 hour') > (NOW() + '8 hour') - INTERVAL '48 HOUR' GROUP BY bucket_time ORDER BY bucket_time ASC"
    )
    .fetch_all(ctx.solar_api.db())
    .await?
    .into_iter()
    .map(|r| {
        GenerationHistory {
            at: r.bucket_time.unwrap(),
            at_utc: r.bucket_time.unwrap().and_local_timezone(Utc).unwrap().to_rfc3339(),
            wh: r.avg.unwrap()
        }
    })
    .partition(|r| {
        let history_date =
            r.at.checked_add_offset(FixedOffset::east_opt(8 * 3600).unwrap())
                .unwrap()
                .date();

        now.date_naive() == history_date
    });

    Ok(Json(SolarHistoryResponse { today, yesterday }))
}

async fn health(ctx: State<BotContext>) -> StatusCode {
    let resp = ctx.solar_api.db().acquire().await;

    if resp.is_err() {
        return StatusCode::SERVICE_UNAVAILABLE;
    }

    let resp = resp.unwrap().ping().await;
    match resp {
        Ok(_) => StatusCode::NO_CONTENT,
        Err(_) => StatusCode::SERVICE_UNAVAILABLE,
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(Targets::default().with_default(Level::INFO))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let database_url = std::env::var("DATABASE_URL")?;
    let token = std::env::var("DISCORD_TOKEN")?;
    let goodwe_username = std::env::var("GOODWE_API_USERNAME")?;
    let goodwe_password = std::env::var("GOODWE_API_PASSWORD")?;
    let goodwe_powerstation_id = std::env::var("GOODWE_API_POWERSTATION_ID")?;

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    let solar_api = GoodWeSemsAPI::new(
        pool,
        goodwe_username,
        goodwe_password,
        goodwe_powerstation_id,
    );

    let context = BotContext(
        BotContextInner {
            solar_api: solar_api.clone(),
        }
        .into(),
    );

    let config = ConfigBuilder::new(token.clone(), Intents::GUILD_MESSAGES).build();

    let mut shard = Shard::with_config(ShardId::ONE, config);

    let http = Arc::new(HttpClient::new(token));

    let cache = InMemoryCacheBuilder::<DefaultCacheModels>::new()
        .resource_types(ResourceType::MESSAGE | ResourceType::GUILD)
        .build();

    let routes = axum::Router::new()
        .route("/health", get(health))
        .route("/current", get(solar_current))
        .route("/history", get(solar_history));

    let app = axum::Router::new()
        .nest("/api", routes)
        .layer(
            CorsLayer::new()
                .allow_origin(AllowOrigin::list([
                    "http://localhost:5173".parse()?,
                    "https://solar-panels.anurag.sh".parse()?,
                ]))
                .allow_methods([Method::GET, Method::OPTIONS])
                .allow_headers(AllowHeaders::any()),
        )
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request<Body>| {
                    tracing::info_span!("api", uri = request.uri().to_string())
                })
                .on_request(DefaultOnRequest::new().level(Level::INFO))
                .on_response(
                    DefaultOnResponse::new()
                        .level(Level::INFO)
                        .latency_unit(LatencyUnit::Millis),
                ),
        )
        .layer(GlobalConcurrencyLimitLayer::new(2048))
        .with_state(context.clone());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    tracing::info!("spawning axum");
    tokio::spawn(axum::serve(listener, app).into_future());

    let app_id = http.current_user_application().await?.model().await?.id;

    let framework = Arc::new(
        Framework::builder(Arc::clone(&http), app_id, context)
            .command(solar)
            .build(),
    );

    framework.register_global_commands().await?;
    let interaction_client = http.interaction(app_id);
    let global_commands = interaction_client.global_commands().await?.model().await?;

    let mut updated_commands = Vec::with_capacity(global_commands.len());
    for global_command in global_commands {
        let mut command = CommandBuilder::new(
            global_command.name,
            global_command.description,
            global_command.kind,
        )
        .integration_types(vec![
            ApplicationIntegrationType::GuildInstall,
            ApplicationIntegrationType::UserInstall,
        ])
        .contexts(vec![
            InteractionContextType::BotDm,
            InteractionContextType::PrivateChannel,
            InteractionContextType::Guild,
        ]);

        for option in global_command.options {
            command = command.option(option);
        }

        updated_commands.push(command.build());
    }

    tracing::info!("updating commands: {}", updated_commands.len());
    interaction_client
        .set_global_commands(&updated_commands)
        .await?;

    if cfg!(debug_assertions) {
        tracing::info!("skipping background thread in debug");
    } else {
        tracing::info!("spawning background thread");
        tokio::spawn(async move {
            loop {
                let solar_api = solar_api.clone();
                let fut = async move {
                    let login_data = solar_api.get_new_or_cached_login_data().await?;
                    solar_api.save_solar_data(login_data).await?;
                    Ok::<(), GoodWeSemsAPIError>(())
                };

                if let Err(e) = fut.await {
                    tracing::error!("error fetching data: {e}");
                }

                tokio::time::sleep(Duration::from_secs(60)).await
            }
        });
    }

    tracing::info!("starting event loop");
    while let Some(event) = shard.next_event(EventTypeFlags::all()).await {
        let Ok(event) = event else {
            let source = event.unwrap_err();
            tracing::warn!(source = ?source, "error receiving event");

            continue;
        };

        if matches!(event.kind(), EventType::GatewayHeartbeatAck) {
            continue;
        }

        cache.update(&event);

        if matches!(event.kind(), EventType::Ready) {
            tracing::info!("connected on shard");
            continue;
        }

        if let Event::InteractionCreate(i) = event {
            let clone = Arc::clone(&framework);
            tokio::spawn(async move {
                let inner = i.0;
                clone.process(inner).await;
            });

            continue;
        }

        tokio::spawn(handle_event(event, Arc::clone(&http)));
    }

    Ok(())
}
