use axum::extract::{Query, State};
use sqlx::PgPool;
use anyhow::Context;
use crate::error::AppError;
use crate::model::{City, GeoResponse, IndexTemplate, LatLong, StatsTemplate, WeatherDisplayTemplate, WeatherQuery, WeatherResponse};

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
    let cities = sqlx::query_as::<_, City>("SELECT name, lat AS latitude, long AS longitude FROM cities ORDER BY id DESC LIMIT 10")
        .fetch_all(pool)
        .await?;
    Ok(cities)
}

pub async fn weather(
    Query(params): Query<WeatherQuery>,
    State(pool): State<PgPool>,
) -> Result<WeatherDisplayTemplate, AppError> {
    let lat_long = get_lat_long(&pool, &params.city).await?;
    let weather = fetch_weather(lat_long).await?;
    Ok(WeatherDisplayTemplate::new(params.city, weather))
}

pub async fn index() -> IndexTemplate {
    IndexTemplate
}

pub async fn stats(State(pool): State<PgPool>) -> Result<StatsTemplate, AppError> {
    let cities = get_last_cities(&pool).await?;
    Ok(StatsTemplate { cities })
}