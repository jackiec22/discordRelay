use axum::{
    routing::{get, post},
    Json, Router,
};
use reqwest::Client;
use serde_json::{json, Value};
use std::{env, net::SocketAddr};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    // Load .env and get your Discord webhook URL
    dotenvy::dotenv().ok();
    let discord_url = env::var("DISCORD_WEBHOOK_URL").expect("missing DISCORD_WEBHOOK_URL");
    let client = Client::new();

    // Build the Axum app with POST and health routes
    let app =
        Router::new()
            .route(
                "/send",
                post({
                    let discord_url = discord_url.clone();
                    move |Json(body): Json<Value>| {
                        let client = client.clone();
                        let discord_url = discord_url.clone();
                        async move {
                            let resp = client.post(&discord_url).json(&body).send().await.map_err(
                                |e| (axum::http::StatusCode::BAD_GATEWAY, e.to_string()),
                            )?;

                            Ok::<_, (axum::http::StatusCode, String)>((
                                axum::http::StatusCode::from_u16(resp.status().as_u16()).unwrap(),
                                "ok".to_string(),
                            ))
                        }
                    }
                }),
            )
            .route("/health", get(health_check)); // ✅ new health route

    // Bind to port 8080
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = SocketAddr::from(([0, 0, 0, 0], port.parse().unwrap()));
    let listener = TcpListener::bind(addr).await.unwrap();
    println!("Relay listening on {}", addr);

    // Run the server
    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> (axum::http::StatusCode, &'static str) {
    (axum::http::StatusCode::OK, "ok")
}
