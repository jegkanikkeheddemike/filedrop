use std::{convert::Infallible, time::Duration, vec};

use axum::{
    extract::State,
    response::{sse::Event, Sse},
};
use filedrop_lib::EventData;
use futures::{channel::mpsc::Sender, Stream};
use tokio::sync::mpsc::Receiver;
use uuid::Uuid;

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
    event_sx: tokio::sync::mpsc::Sender<EventData>,
    mut sub_rx: Receiver<Sender<Result<Event, Infallible>>>,
) {
    let mut subcribers = vec![];

    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(5)).await;
            //Ping
            event_sx
                .send(EventData {
                    filename: "//PING".into(),
                    file_id: Uuid::from_u128(0),
                    groupname: "".into(),
                    group_id: Uuid::from_u128(0),
                    sender: "".into(),
                })
                .await
                .unwrap();
        }
    });

    while let Some(event) = event_rx.recv().await {
        if event.filename != "//PING" {
            println!("{event:?}");
        }
        

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
