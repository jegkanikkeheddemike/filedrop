use std::convert::Infallible;

use anyhow::{Ok, Result};
use axum::{
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
        event_send,
        sub_send,
    };

    let app = Router::new()
        .route("/upload", post(upload::upload))
        .route("/subscribe", get(subscribe::subscribe))
        .route("/get_md/:user_id", get(users::get_user))
        .nest_service("/download/", ServeDir::new("cache"))
        .with_state(state);

    tokio::spawn(subscribe::event_respond(event_recieve, subscriber_recv));

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
