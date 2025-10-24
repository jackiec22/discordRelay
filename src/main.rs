use axum::{routing::post, Json, Router};
use reqwest::Client;
use serde_json::Value;
use std::{env, net::SocketAddr};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    // Get your real Discord webhook URL from env var
    dotenvy::dotenv().ok();
    let discord_url = env::var("DISCORD_WEBHOOK_URL").expect("missing DISCORD_WEBHOOK_URL");
    let client = Client::new();

    // Build the Axum app with one POST endpoint
    let app = Router::new().route(
        "/send",
        post({
            let discord_url = discord_url.clone();
            move |Json(body): Json<Value>| {
                let client = client.clone();
                let discord_url = discord_url.clone();
                async move {
                    let resp = client
                        .post(&discord_url)
                        .json(&body)
                        .send()
                        .await
                        .map_err(|e| (axum::http::StatusCode::BAD_GATEWAY, e.to_string()))?;

                    Ok::<_, (axum::http::StatusCode, String)>((
                        axum::http::StatusCode::from_u16(resp.status().as_u16()).unwrap(),
                        "ok".to_string(),
                    ))
                }
            }
        }),
    );

    // Bind to port 8080
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    let listener = TcpListener::bind(addr).await.unwrap();
    println!("Relay listening on {}", addr);

    // Run the server
    axum::serve(listener, app).await.unwrap();
}