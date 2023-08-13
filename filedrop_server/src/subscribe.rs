use std::{collections::HashMap, convert::Infallible, time::Duration, vec};

use axum::{
    extract::{Path, State},
    response::{sse::Event, Sse},
};
use filedrop_lib::EventData;
use futures::{channel::mpsc::Sender, Stream};
use tokio::sync::mpsc::Receiver;
use uuid::Uuid;

use crate::ServerState;

pub async fn subscribe(
    State(state): State<ServerState>,
    Path(user_id): Path<Uuid>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let (sender, receiver) = futures::channel::mpsc::channel(1024);
    state
        .sub_send
        .send((sender, user_id))
        .await
        .expect("Failed to send sub to responder. This is real bad :(");

    Sse::new(receiver)
}

pub async fn event_respond(
    mut event_rx: Receiver<EventData>,
    event_sx: tokio::sync::mpsc::Sender<EventData>,
    mut sub_rx: Receiver<(Sender<Result<Event, Infallible>>, Uuid)>,
) {
    let mut subcribers = HashMap::new();

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
        while let Ok((sub, user_id)) = sub_rx.try_recv() {
            subcribers.insert(user_id, sub);
        }
        let mut failed = vec![];
        for (user_id, sub) in &mut subcribers {
            if sub.start_send(Ok(sse_event.clone())).is_err() {
                failed.push(*user_id);
            }
        }
        for user_id in failed {
            subcribers.remove(&user_id);
        }
    }
}
