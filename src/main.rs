use anyhow::Context;
use askama::Template;
use axum::{routing::get, Router};
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use sqlx::PgPool;

// Make our own error that wraps `anyhow::Error`.
struct AppError(anyhow::Error);

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`. That way you don't need to do that manually.
impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate;

#[derive(Deserialize)]
pub struct WeatherQuery {
    pub city: String,
}

#[derive(Deserialize, Debug)]
pub struct WeatherResponse {
    pub latitude: f64,
    pub longitude: f64,
    pub timezone: String,
    pub hourly: Hourly,
}

#[derive(Deserialize, Debug)]
pub struct Hourly {
    pub time: Vec<String>,
    pub temperature_2m: Vec<f64>,
}

#[derive(Template, Deserialize, Debug)]
#[template(path = "weather.html")]
struct WeatherDisplay {
    city: String,
    forecasts: Vec<Forecast>,
}

#[derive(Deserialize, Debug)]
pub struct Forecast {
    pub date: String,
    pub temperature: String,
}

impl WeatherDisplay {
    /// Create a new `WeatherDisplay` from a `WeatherResponse`.
    fn new(city: String, response: WeatherResponse) -> Self {
        let display = WeatherDisplay {
            city,
            forecasts: response
                .hourly
                .time
                .iter()
                .zip(response.hourly.temperature_2m.iter())
                .map(|(date, temperature)| Forecast {
                    date: date.to_string(),
                    temperature: temperature.to_string(),
                })
                .collect(),
        };
        display
    }
}

#[derive(Deserialize, Debug)]
pub struct GeoResponse {
    pub results: Vec<LatLong>,
}

#[derive(sqlx::FromRow, Deserialize, Debug, Clone)]
pub struct LatLong {
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(sqlx::FromRow, Deserialize, Debug, Clone)]
pub struct City {
    pub name: String,
}

#[derive(Template)]
#[template(path = "stats.html")]
struct StatsTemplate {
    pub cities: Vec<City>,
}

async fn get_lat_long(pool: &PgPool, name: &str) -> Result<LatLong, anyhow::Error> {
    let lat_long = sqlx::query_as::<_, LatLong>(
        "SELECT lat AS latitude, long AS longitude FROM cities WHERE name = $1",
    )
        .bind(name)
        .fetch_optional(pool)
        .await?;

    if let Some(lat_long) = lat_long {
        return Ok(lat_long);
    }

    let lat_long = fetch_lat_long(name).await?;
    sqlx::query("INSERT INTO cities (name, lat, long) VALUES ($1, $2, $3)")
        .bind(name)
        .bind(lat_long.latitude)
        .bind(lat_long.longitude)
        .execute(pool)
        .await?;

    Ok(lat_long)
}

async fn fetch_lat_long(city: &str) -> Result<LatLong, anyhow::Error> {
    let endpoint = format!(
        "https://geocoding-api.open-meteo.com/v1/search?name={}&count=1&language=en&format=json",
        city
    );
    let response = reqwest::get(&endpoint).await?.json::<GeoResponse>().await?;
    response.results.get(0).cloned().context("No results found")
}

async fn fetch_weather(lat_long: LatLong) -> Result<WeatherResponse, anyhow::Error> {
    let endpoint = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&hourly=temperature_2m",
        lat_long.latitude, lat_long.longitude
    );
    let response = reqwest::get(&endpoint).await?.json::<WeatherResponse>().await?;
    Ok(response)
}

async fn get_last_cities(pool: &PgPool) -> Result<Vec<City>, AppError> {
    let cities = sqlx::query_as::<_, City>("SELECT name FROM cities ORDER BY id DESC LIMIT 10")
        .fetch_all(pool)
        .await?;
    Ok(cities)
}

async fn weather(
    Query(params): Query<WeatherQuery>,
    State(pool): State<PgPool>,
) -> Result<WeatherDisplay, AppError> {
    let lat_long = get_lat_long(&pool, &params.city).await?;
    let weather = fetch_weather(lat_long).await?;
    Ok(WeatherDisplay::new(params.city, weather))
}

async fn index() -> IndexTemplate {
    IndexTemplate
}

async fn stats(State(pool): State<PgPool>) -> Result<StatsTemplate, AppError> {
    let cities = get_last_cities(&pool).await?;
    Ok(StatsTemplate { cities })
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let db_connection_str = std::env::var("DATABASE_URL").context("DATABASE_URL must be set")?;

    let pool = sqlx::PgPool::connect(&db_connection_str)
        .await
        .context("can't connect to database")?;

    let app = Router::new()
        .route("/", get(index))
        .route("/weather", get(weather))
        .route("/stats", get(stats))
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;

    axum::serve(listener, app)
        .await?;

    Ok(())
}