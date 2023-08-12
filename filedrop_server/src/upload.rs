use axum::extract::{Multipart, State};
use filedrop_lib::EventData;
use tokio::fs;
use uuid::Uuid;

use crate::ServerState;

pub async fn upload(State(state): State<ServerState>, mut multipart: Multipart) {
    while let Ok(Some(field)) = multipart.next_field().await {
        //Tillader upload af MAAAnge filer af gangen
        let Some(filename) = field.file_name().map(ToOwned::to_owned) else {
            continue;
        };

        let Ok(bytes) = field.bytes().await else {
            println!("Failed to read bytes of {filename}");
            continue;
        };

        let file_id = Uuid::new_v4();

        if let Err(err) = fs::write(format!("./cache/{file_id}"), bytes).await {
            println!("Failed to save {filename}: {err}");
        }
        let event_data = EventData {
            filename,
            file_id,
            group: "Software".into(),
            sender: "Thor".into(),
        };
        if let Err(err) = state.event_send.send(event_data).await {
            println!("Failed to send event: {err:?}");
        }
    }
}
