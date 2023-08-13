use std::convert::Infallible;

use anyhow::{Ok, Result};
use axum::{
    extract::DefaultBodyLimit,
    response::sse::Event,
    routing::{get, post},
    Router,
};
use filedrop_lib::EventData;
use futures::channel::mpsc::Sender;
use tokio::sync::mpsc::channel;
use tower_http::services::ServeDir;

mod db;
mod subscribe;
mod upload;
mod users;

#[tokio::main]
async fn main() -> Result<()> {
    #[cfg(not(debug_assertions))]
    {
        use tokio::fs;
        if fs::try_exists("./cache").await? {
            fs::remove_dir_all("./cache").await?;
        }
        fs::create_dir("./cache").await?;
    }

    db::init().await;

    let (event_send, event_recieve) = channel::<EventData>(1024);
    let (sub_send, subscriber_recv) = channel::<Sender<Result<Event, Infallible>>>(1024);

    let state = ServerState {
        event_send: event_send.clone(),
        sub_send,
    };

    let app = Router::new()
        .route("/upload", post(upload::upload))
        .route("/subscribe", get(subscribe::subscribe))
        .route("/get_md/:user_id", get(users::get_user))
        .route("/create", post(users::create_group))
        .route("/join", post(users::join_group))
        .nest_service("/download/", ServeDir::new("cache"))

        .layer(DefaultBodyLimit::max(1_000_000_000)) // 1gb
        .with_state(state);

    tokio::spawn(subscribe::event_respond(event_recieve, event_send,subscriber_recv));

    axum::Server::bind(&"0.0.0.0:3987".parse()?)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
#[derive(Clone)]
pub struct ServerState {
    event_send: tokio::sync::mpsc::Sender<EventData>,
    sub_send: tokio::sync::mpsc::Sender<Sender<Result<Event, Infallible>>>,
}
