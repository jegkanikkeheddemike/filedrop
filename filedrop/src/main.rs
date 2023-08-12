use anyhow::{Ok, Result};
use eframe::{CreationContext, NativeOptions};
use egui::{Button, CentralPanel};
use pinboard::Pinboard;
use reqwest::{
    multipart::{Form, Part},
    StatusCode,
};
use std::{env::args, fmt::Display, sync::Arc};
use tinyfiledialogs::open_file_dialog;

#[tokio::main]
async fn main() {
    let file = args().nth(1);

    let native_options = NativeOptions {
        initial_window_size: Some((1080. / 5., 1500. / 5.).into()),
        initial_window_pos: Some((1920., 1080. / 2.).into()), //Virker ikke?
        ..Default::default()
    };
    eframe::run_native(
        "File Drop",
        native_options,
        Box::new(|cc| Box::new(Application::new(cc, file))),
    )
    .unwrap();
}

struct Application {
    status: Arc<Pinboard<FileStatus>>,
}

#[derive(Debug, Clone)]
enum FileStatus {
    Unselected,
    Selected(String),
    Sending,
    Success,
    Failed(String, String),
}

impl Application {
    fn new(_cc: &CreationContext<'_>, file: Option<String>) -> Self {
        Self {
            status: Arc::new(Pinboard::new(
                file.map(FileStatus::Selected)
                    .unwrap_or(FileStatus::Unselected),
            )),
        }
    }
}

impl eframe::App for Application {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| match self.status.read().unwrap() {
            FileStatus::Selected(filename) => {
                ui.vertical_centered(|ui| {
                    ui.heading(format!("Upload {filename}"));
                    let button = ui.add_sized((100., 40.), Button::new("upload"));

                    if button.clicked() {
                        tokio::spawn(upload_file(filename.clone(), self.status.clone()));
                        self.status.set(FileStatus::Sending);
                    }
                });
            }
            FileStatus::Unselected => {
                ui.vertical_centered(|ui| {
                    ui.heading("Select file");
                    let button = ui.add_sized((100., 40.), Button::new("file"));

                    if button.clicked() {
                        if let Some(file) = open_file_dialog("select file", "~", None) {
                            self.status.set(FileStatus::Selected(file));
                        }
                    }
                });
            }
            FileStatus::Failed(filename, error) => {
                ui.vertical_centered(|ui| {
                    ui.heading("Failed");
                    let button = ui.add_sized((100., 40.), Button::new("retry"));

                    if button.clicked() {
                        tokio::spawn(upload_file(filename.clone(), self.status.clone()));
                        self.status.set(FileStatus::Sending);
                    }
                    ui.label(error);
                });
            }
            FileStatus::Sending => {
                ui.vertical_centered(|ui| {
                    ui.heading(format!("Uploading"));
                });
            }
            FileStatus::Success => {
                ui.vertical_centered(|ui| {
                    ui.heading(format!("Success"));
                    let button = ui.add_sized((100., 40.), Button::new("upload another"));

                    if button.clicked() {
                        if let Some(file) = open_file_dialog("select file", "~", None) {
                            self.status.set(FileStatus::Selected(file));
                        }
                    }
                });
            }
        });
    }
}

async fn upload_file(filename: String, board: Arc<Pinboard<FileStatus>>) {
    async fn inner(filename: String) -> Result<()> {
        let bytes = std::fs::read(&filename)?;
        let parts = Form::new().part("file", Part::bytes(bytes).file_name(filename));

        let response = reqwest::Client::new()
            .post("http://koebstoffer.info:3987/upload")
            .multipart(parts)
            .send()
            .await?;
        if response.status().is_success() {
            Ok(())
        } else {
            Err(ResponseError {
                _status_code: response.status(),
            })?
        }
    }

    if let Err(err) = inner(filename.clone()).await {
        board.set(FileStatus::Failed(filename, err.to_string()))
    } else {
        board.set(FileStatus::Success)
    }
}

#[derive(Debug)]
struct ResponseError {
    _status_code: StatusCode,
}

impl Display for ResponseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for ResponseError {}
