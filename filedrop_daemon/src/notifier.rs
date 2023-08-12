use anyhow::Result;
use notify_rust::Notification;
use tokio::sync::oneshot;

pub async fn ask_download(filename: &str, group: &str, sender: &str) -> Result<bool> {
    let (sx, rx) = oneshot::channel::<bool>();

    Notification::new()
        .summary(&format!("\"{filename}\" shared with {group} by {sender}"))
        .action("download", "download")
        .icon("firefox")
        .show()?
        .wait_for_action(|action| match action {
            "download" => sx.send(true).unwrap(),
            _ => sx.send(false).unwrap(),
        });

    Ok(rx.await?)
}

#[tokio::test]
async fn test_download() -> Result<()> {
    let res = ask_download("filename", "group", "sender").await?;
    println!("{res}");

    Ok(())
}
