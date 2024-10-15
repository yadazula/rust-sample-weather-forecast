use crate::models::error::AppError;
use crate::models::geo::{GeoResponse, LatLong};
use crate::models::weather::{
    IndexTemplate, WeatherDisplayTemplate, WeatherQuery, WeatherResponse,
};
use anyhow::Context;
use axum::extract::{Query, State};
use sqlx::PgPool;
use tracing::debug;

pub async fn index() -> IndexTemplate {
    IndexTemplate
}

pub async fn weather(
    Query(params): Query<WeatherQuery>,
    State(pool): State<PgPool>,
) -> Result<WeatherDisplayTemplate, AppError> {
    let lat_long = get_lat_long(&pool, &params.city).await?;
    let weather = fetch_weather(lat_long).await?;
    Ok(WeatherDisplayTemplate::new(params.city, weather))
}

async fn get_lat_long(pool: &PgPool, name: &str) -> Result<LatLong, anyhow::Error> {
    debug!("Checking if city is already in database for city: {}", name);
    let lat_long = sqlx::query_as::<_, LatLong>(
        "SELECT lat AS latitude, long AS longitude FROM cities WHERE name = $1",
    )
    .bind(name)
    .fetch_optional(pool)
    .await?;

    if let Some(lat_long) = lat_long {
        debug!("City found in database, returning lat/long: {:?}", lat_long);
        return Ok(lat_long);
    }

    let lat_long = fetch_lat_long(name).await?;
    sqlx::query("INSERT INTO cities (name, lat, long) VALUES ($1, $2, $3)")
        .bind(name)
        .bind(lat_long.latitude)
        .bind(lat_long.longitude)
        .execute(pool)
        .await?;

    debug!(
        "City not found in database, fetched from API: {:?}",
        lat_long
    );
    Ok(lat_long)
}

async fn fetch_lat_long(city: &str) -> Result<LatLong, anyhow::Error> {
    let endpoint = format!(
        "https://geocoding-api.open-meteo.com/v1/search?name={}&count=1&language=en&format=json",
        city
    );

    debug!("Fetching lat/long for city: {}", city);
    let response = reqwest::get(&endpoint).await?.json::<GeoResponse>().await?;
    debug!("Fetch response: {:?}", response);

    response
        .results
        .first()
        .cloned()
        .context("No results found")
}

async fn fetch_weather(lat_long: LatLong) -> Result<WeatherResponse, anyhow::Error> {
    let endpoint = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&hourly=temperature_2m",
        lat_long.latitude, lat_long.longitude
    );

    debug!("Fetching weather for lat/long: {:?}", lat_long);
    let response = reqwest::get(&endpoint)
        .await?
        .json::<WeatherResponse>()
        .await?;

    Ok(response)
}
