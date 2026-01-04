use axum::{
    extract::{Path, Request},
    http::StatusCode,
    middleware::{self, Next},
    response::{Html, Response},
    routing::{get, post, put, delete},
    Json, Router,
};
use dotenv::dotenv;
use reqwest::header::{HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use std::env;

mod movie;
use movie::Movie;

const MOVIE_LIMIT: i64 = 100;

#[derive(Serialize)]
struct ApiInfo {
    message: String,
    endpoints: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CreateMovie {
    title: String,
    tagline: Option<String>,
    popularity: Option<f64>,
    release_date: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct UpdateMovie {
    title: String,
    tagline: Option<String>,
    popularity: Option<f64>,
    release_date: Option<String>,
}

pub fn build_router() -> Router {
    dotenv().ok();

    Router::new()
        .route("/", get(movies_html_handler()))
        .route("/about", get(about_handler))
        .route("/movies", get(index_handler()).post(create_movie_handler))
        .route("/movies/:id", put(update_movie_handler).delete(delete_movie_handler))
        .route("/movies.json", get(movies_json_handler))
        .route("/api", get(root_handler))
        .fallback(not_found_handler)
        .layer(middleware::from_fn(log_requests))
}


async fn get_movies() -> Result<Vec<Movie>, reqwest::Error> {
    let supabase_url = env::var("SUPABASE_URL").expect("missing SUPABASE_URL");
    let supabase_key = env::var("SUPABASE_KEY").expect("missing SUPABASE_KEY");

    let url = format!(
        "{}/rest/v1/movies?select=*&limit={}&order=id.desc",
        supabase_url, MOVIE_LIMIT
    );

    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();
    headers.insert("apikey", HeaderValue::from_str(&supabase_key).unwrap());
    headers.insert(
        "Authorization",
        HeaderValue::from_str(&format!("Bearer {}", supabase_key)).unwrap(),
    );

    let res = client.get(url).headers(headers).send().await?;
    let movies: Vec<Movie> = res.json().await?;
    Ok(movies)
}

async fn create_movie_handler(
    Json(payload): Json<CreateMovie>,
) -> Result<Json<Movie>, (StatusCode, String)> {
    let supabase_url = env::var("SUPABASE_URL").expect("missing SUPABASE_URL");
    let supabase_key = env::var("SUPABASE_KEY").expect("missing SUPABASE_KEY");

    let url = format!("{}/rest/v1/movies", supabase_url);

    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();
    headers.insert("apikey", HeaderValue::from_str(&supabase_key).unwrap());
    headers.insert(
        "Authorization",
        HeaderValue::from_str(&format!("Bearer {}", supabase_key)).unwrap(),
    );
    headers.insert("Content-Type", HeaderValue::from_static("application/json"));
    headers.insert("Prefer", HeaderValue::from_static("return=representation"));

    let res = client
        .post(url)
        .headers(headers)
        .json(&payload)
        .send()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if !res.status().is_success() {
        let error_text = res.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err((StatusCode::BAD_REQUEST, error_text));
    }

    let movies: Vec<Movie> = res
        .json()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    movies
        .into_iter()
        .next()
        .ok_or((StatusCode::INTERNAL_SERVER_ERROR, "No movie returned".to_string()))
        .map(Json)
}

async fn update_movie_handler(
    Path(id): Path<i32>,
    Json(payload): Json<UpdateMovie>,
) -> Result<Json<Movie>, (StatusCode, String)> {
    let supabase_url = env::var("SUPABASE_URL").expect("missing SUPABASE_URL");
    let supabase_key = env::var("SUPABASE_KEY").expect("missing SUPABASE_KEY");

    let url = format!("{}/rest/v1/movies?id=eq.{}", supabase_url, id);

    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();
    headers.insert("apikey", HeaderValue::from_str(&supabase_key).unwrap());
    headers.insert(
        "Authorization",
        HeaderValue::from_str(&format!("Bearer {}", supabase_key)).unwrap(),
    );
    headers.insert("Content-Type", HeaderValue::from_static("application/json"));
    headers.insert("Prefer", HeaderValue::from_static("return=representation"));

    let res = client
        .patch(url)
        .headers(headers)
        .json(&payload)
        .send()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if !res.status().is_success() {
        let error_text = res.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err((StatusCode::BAD_REQUEST, error_text));
    }

    let movies: Vec<Movie> = res
        .json()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    movies
        .into_iter()
        .next()
        .ok_or((StatusCode::NOT_FOUND, "Movie not found".to_string()))
        .map(Json)
}

async fn delete_movie_handler(Path(id): Path<i32>) -> Result<StatusCode, (StatusCode, String)> {
    let supabase_url = env::var("SUPABASE_URL").expect("missing SUPABASE_URL");
    let supabase_key = env::var("SUPABASE_KEY").expect("missing SUPABASE_KEY");

    let url = format!("{}/rest/v1/movies?id=eq.{}", supabase_url, id);

    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();
    headers.insert("apikey", HeaderValue::from_str(&supabase_key).unwrap());
    headers.insert(
        "Authorization",
        HeaderValue::from_str(&format!("Bearer {}", supabase_key)).unwrap(),
    );

    let res = client
        .delete(url)
        .headers(headers)
        .send()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if !res.status().is_success() {
        let error_text = res.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err((StatusCode::BAD_REQUEST, error_text));
    }

    Ok(StatusCode::NO_CONTENT)
}

async fn index_handler() -> Html<String> {
    Html(include_str!("index.html").to_string())
}

async fn about_handler() -> Html<String> {
    Html(include_str!("about.html").to_string())
}

async fn root_handler() -> Json<ApiInfo> {
    Json(ApiInfo {
        message: "Movie API Server".to_string(),
        endpoints: vec![
            "GET /movies".to_string(),
            "GET /movies.json".to_string(),
            "POST /movies".to_string(),
            "PUT /movies/:id".to_string(),
            "DELETE /movies/:id".to_string(),
            "/about".to_string(),
        ],
    })
}

async fn movies_json_handler() -> Result<Json<Vec<Movie>>, StatusCode> {
    let movies = get_movies().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(movies))
}


async fn movies_html_handler() -> Html<String> {
    Html(include_str!("movie.html").to_string())
}

async fn log_requests(req: Request, next: Next) -> Result<Response, StatusCode> {
    println!("{} {}", req.method(), req.uri().path());
    let res = next.run(req).await;
    Ok(res)
}

async fn not_found_handler() -> (StatusCode, Html<String>) {
    (
        StatusCode::NOT_FOUND,
        Html(
            r#"
            <!doctype html>
            <html lang="en">
            <head><meta charset="utf-8"><title>Not Found</title></head>
            <body><h1>404 Not Found</h1></body>
            </html>
            "#
            .to_string(),
        ),
    )
}