use std::sync::mpsc;

use anyhow::{Ok, Result};
use eframe::NativeOptions;
use egui::CentralPanel;

pub fn ask_download(filename: &str, group: &str, sender: &str) -> Result<bool> {
    dbg!("Asking download: ", filename, group, sender);
    let (sx, rx) = mpsc::channel::<bool>();
    notif_sound();
    non_native_popup(filename.into(), group.into(), sender.into(), sx);

    Ok(rx.recv().unwrap())
}

fn notif_sound() {
    use soloud::*;
    let sl = Soloud::default().unwrap();
    let mut wav = audio::Wav::default();
    wav.load_mem(include_bytes!("../notif.mp3")).unwrap();
    sl.play(&wav);
    while sl.voice_count() > 0 {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}

#[test]
fn test_download() -> Result<()> {
    let res = ask_download("filename", "group", "sender")?;
    println!("{res}");

    Ok(())
}

fn non_native_popup(filename: String, group: String, sender: String, sx1: mpsc::Sender<bool>) {
    let native_options = NativeOptions {
        initial_window_size: Some((200., 100.).into()),
        initial_window_pos: Some((1920., 1080. / 2.).into()), //Virker ikke?
        always_on_top: true,
        resizable: false,
        ..Default::default()
    };
    let sx = sx1.clone();
    eframe::run_native(
        &format!("{sender} shared a file with {group}"),
        native_options,
        Box::new(|_cc| {
            Box::new(PopupApp {
                filename,
                group,
                sender,
                sx,
            })
        }),
    )
    .unwrap();
    let _ = sx1.send(false);
}

struct PopupApp {
    filename: String,
    group: String,
    sender: String,
    sx: mpsc::Sender<bool>,
}

impl eframe::App for PopupApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading(&format!(
                    "{} shared a file with {}",
                    self.sender, self.group
                ));
                ui.label(&self.filename);
                if ui.button("download").clicked() {
                    let sx = self.sx.clone();

                    sx.send(true).unwrap();

                    frame.close();
                }
            });
        });
    }
}
