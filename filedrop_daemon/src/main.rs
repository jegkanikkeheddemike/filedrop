#![windows_subsystem = "windows"]
use std::{env, ops::Deref};

use anyhow::{Ok, Result};
use filedrop_lib::{localdata::get_localdata, EventData};
use notifier::ask_download;
use notify_rust::Notification;
use once_cell::sync::Lazy;
use sse_client::EventSource;
use tokio::fs;
mod notifier;

static REMOTE_ADDR: Lazy<String> = Lazy::new(|| {
    env::var("REMOTE_ADDR").unwrap_or_else(|_| "http://koebstoffer.info:3987/".into())
});

fn main() {
    let (msg_sx, msg_rx) = std::sync::mpsc::channel::<EventData>();

    std::thread::spawn(move || {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                let source = format!(
                    "{}subscribe/{}",
                    REMOTE_ADDR.deref(),
                    get_localdata().user_id
                );
                println!("Source: {source}");
                let event_source = EventSource::new(&source).unwrap();

                event_source.on_open(|| {
                    Notification::new()
                        .summary("File drop daemon connected.")
                        .show()
                        .unwrap();
                });

                for message in event_source.receiver().iter() {
                    let res = serde_json::from_str(&message.data);
                    let data = match res {
                        Result::Ok(data) => data,
                        Result::Err(err) => {
                            dbg!("ERROR: ", err);
                            continue;
                        }
                    };
                    msg_sx.send(data).unwrap();
                }
            });
    });

    loop {
        let data = msg_rx.recv().unwrap();
        if let Err(err) = handle_msg(data) {
            dbg!("Err: ", err);
        };
    }
}

fn handle_msg(data: EventData) -> Result<()> {
    if data.filename == "//PING" {
        return Ok(());
    }
    dbg!(&data);

    if ask_download(&data.filename, &data.groupname, &data.sender)? {
        //Holy shit det her er en absolute FORFÆRDELIGT løsning men whatever.
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                let response =
                    reqwest::get(format!("{}download/{}", REMOTE_ADDR.deref(), data.file_id));
                let mut filepath = dirs::download_dir().expect("unsupported os");
                filepath.push(data.filename);
                fs::write(&filepath, response.await.unwrap().bytes().await.unwrap())
                    .await
                    .unwrap();
                println!("{:?}", open::that(filepath));
            });
    }

    Ok(())
}
