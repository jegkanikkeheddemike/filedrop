use std::{env, ops::Deref};

use anyhow::{Ok, Result};
use filedrop_lib::EventData;
use notifier::ask_download;
use once_cell::sync::Lazy;
use sse_client::EventSource;
use tokio::fs;
mod notifier;

static REMOTE_ADDR: Lazy<String> = Lazy::new(|| {
    env::var("REMOTE_ADDR").unwrap_or_else(|_| "http://koebstoffer.info:3987/".into())
});

#[tokio::main]
async fn main() {
    let source = format!("{}subscribe", REMOTE_ADDR.deref());
    println!("Source: {source}");
    let event_source = EventSource::new(&source).unwrap();

    for message in event_source.receiver().iter() {
        tokio::spawn(async move {
            if let Err(err) = handle_message(&message.data).await {
                println!("\"{}\" error: {err}", message.type_);
            }
        });
    }
}

async fn handle_message(data: &str) -> Result<()> {
    let data: EventData = serde_json::from_str(data)?;

    if ask_download(&data.filename, &data.groupname, &data.sender).await? {
        let response =
            reqwest::get(format!("{}download/{}", REMOTE_ADDR.deref(), data.file_id)).await?;
        let mut filepath = dirs::download_dir().expect("unsupported os");
        filepath.push(data.filename);
        fs::write(&filepath, response.bytes().await?).await?;
        println!("{:?}", open::that(filepath));
    }

    Ok(())
}
