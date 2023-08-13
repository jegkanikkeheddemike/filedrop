use anyhow::Result;
use notify_rust::Notification;
use tokio::sync::oneshot;

pub async fn ask_download(filename: &str, group: &str, sender: &str) -> Result<bool> {
    let (sx, rx) = oneshot::channel::<bool>();
    #[cfg(target_os = "linux")]
    {
        Notification::new()
            .summary(&format!("{sender} shared file with {group}"))
            .body(&format!("{filename}"))
            .action("download", "download")
            .show()?
            .wait_for_action(|action| match action {
                "download" => sx.send(true).unwrap(),
                other => {
                    println!("action: {other}");
                    sx.send(false).unwrap()
                }
            });
    }
    #[cfg(not(target_os = "linux"))]
    {
        //TODO
        // Det her skal erstattes med
        // https://docs.rs/winrt-toast/latest/winrt_toast/
        // siden notify-rust ikke understøtter actions i windows
        Notification::new()
            .summary(&format!("{sender} shared file with {group}"))
            .body(&format!("{filename}"))
            .action("download", "download")
            .show()?;
        sx.send(true).unwrap();
    }
    Ok(rx.await?)
}

#[tokio::test]
async fn test_download() -> Result<()> {
    let res = ask_download("filename", "group", "sender").await?;
    println!("{res}");

    Ok(())
}
