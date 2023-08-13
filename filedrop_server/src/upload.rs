use std::{str::FromStr, time::Duration};

use axum::extract::{Multipart, State};
use filedrop_lib::EventData;
use tokio::fs;
use uuid::Uuid;

use crate::{db, ServerState};

pub async fn upload(State(state): State<ServerState>, mut multipart: Multipart) {
    let mut group_id = None;
    let mut sender = None;
    while let Ok(Some(field)) = multipart.next_field().await {
        if field.name() == Some("group_id") {
            group_id = Uuid::from_str(&field.text().await.unwrap()).ok();
            continue;
        } else if field.name() == Some("sender") {
            sender = Some(field.text().await.unwrap());
            continue;
        }

        //Tillader upload af MAAAnge filer af gangen
        let Some(filename) = field.file_name().map(ToOwned::to_owned) else {
            continue;
        };
        let filename = filename.split('/').last().unwrap().to_string();

        let bytes = match field.bytes().await {
            Ok(bytes) => bytes,
            Err(err) => {
                println!("Failed to read bytes of {filename}. {err}");
                continue;
            }
        };

        let file_id = Uuid::new_v4();

        if let Err(err) = fs::write(format!("./cache/{file_id}"), bytes).await {
            println!("Failed to save {filename}. {err}");
        }

        //Find the groupname from the groupid
        let groupname = sqlx::query!(
            "select name from groups where id = $1",
            &group_id.unwrap().to_string()
        )
        .fetch_one(db::get())
        .await
        .unwrap()
        .name
        .unwrap();

        let event_data = EventData {
            filename,
            file_id,
            groupname,
            sender: sender.clone().unwrap(),
            group_id: group_id.unwrap(),
        };
        if let Err(err) = state.event_send.send(event_data).await {
            println!("Failed to send event: {err:?}");
        }

        tokio::spawn(remove_old(file_id));
    }
}

async fn remove_old(file_id: Uuid) {
    tokio::time::sleep(Duration::from_secs(3600)).await;
    fs::remove_file(format!("./cache/{file_id}"))
        .await
        .expect("Failed to remove 1 hour old file");
}
