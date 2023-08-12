use std::{convert::Infallible, vec};

use axum::{
    extract::State,
    response::{sse::Event, Sse},
};
use filedrop_lib::EventData;
use futures::{channel::mpsc::Sender, Stream};
use tokio::sync::mpsc::Receiver;

use crate::ServerState;

pub async fn subscribe(
    State(state): State<ServerState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    println!("Recieved subscriber!");
    let (sender, receiver) = futures::channel::mpsc::channel(1024);
    state
        .sub_send
        .send(sender)
        .await
        .expect("Failed to send sub to responder. This is real bad :(");

    println!("Recieved subscriber!");
    Sse::new(receiver)
}

pub async fn event_respond(
    mut event_rx: Receiver<EventData>,
    mut sub_rx: Receiver<Sender<Result<Event, Infallible>>>,
) {
    let mut subcribers = vec![];

    while let Some(event) = event_rx.recv().await {
        println!("{event:?}");

        //Omdan til en SSE event
        let Ok(sse_event) = Event::default().json_data(event.clone()) else {
            println!("Failed to parse {event:?} into valid sse event");
            continue;
        };

        //Når den modtager en event. Skal den tjekke om der er kommet nye subscribers
        while let Ok(sub) = sub_rx.try_recv() {
            subcribers.push(sub);
        }

        //Alle de senders hvor det lykkeds at sende, skal forblive til næste besked.
        let mut living_subscribers = vec![];

        for mut sub in subcribers {
            if sub.start_send(Ok(sse_event.clone())).is_ok() {
                living_subscribers.push(sub);
            }
        }

        subcribers = living_subscribers;
    }
}
