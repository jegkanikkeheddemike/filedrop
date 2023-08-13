use anyhow::{Ok, Result};
use notify_rust::Notification;
use tokio::sync::oneshot;

pub async fn ask_download(filename: &str, group: &str, sender: &str) -> Result<bool> {
    #[cfg(target_os = "linux")]
    {
        let (sx, rx) = oneshot::channel::<bool>();
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
        Ok(rx.await?)
    }
    #[cfg(target_os = "windows")]
    {
        // TODO
        // Det her skal erstattes med
        // https://docs.rs/winrt-toast/latest/winrt_toast/
        // siden notify-rust ikke understøtter actions i windows
        Notification::new()
            .summary(&format!("{sender} shared file with {group}"))
            .body(&format!("{filename}"))
            .action("download", "download")
            .show()?;
        Ok(true)
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        compile_error!("Unsupported os. Does not know how to handle notifications")
    }
}

#[tokio::test]
async fn test_download() -> Result<()> {
    let res = ask_download("filename", "group", "sender").await?;
    println!("{res}");

    Ok(())
}
