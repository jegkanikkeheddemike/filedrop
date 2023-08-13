use anyhow::{Ok, Result};
use eframe::{CreationContext, NativeOptions};
use egui::{Button, CentralPanel, ComboBox};
use filedrop_lib::Group;
use pinboard::Pinboard;
use reqwest::{
    multipart::{Form, Part},
    StatusCode,
};
use std::{env::args, fmt::Display, sync::Arc};
use tinyfiledialogs::open_file_dialog;

mod localdata;

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
    groups: Arc<Pinboard<Vec<Group>>>,
    selected_group: usize,
}

#[derive(Debug, Clone)]
enum FileStatus {
    Unselected,
    Selected(String),
    Sending,
    Success,
    Failed(String, String),
    LoadingMd,
    FailedMd(Option<String>, String),
}

impl Application {
    fn new(_cc: &CreationContext<'_>, file: Option<String>) -> Self {
        let status = Arc::new(Pinboard::new(FileStatus::LoadingMd));
        let groups = Arc::new(Pinboard::new(vec![]));
        tokio::spawn(load_md(file, status.clone(), groups.clone()));
        Self {
            status,
            groups,
            selected_group: 0,
        }
    }
}

impl eframe::App for Application {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| match self.status.read().unwrap() {
            FileStatus::Selected(filename) => {
                ui.vertical_centered(|ui| {
                    let short_name = filename.split('/').last().unwrap();
                    ui.heading(format!("Upload {short_name}"));

                    let groups = self.groups.read().unwrap();
                    ui.add_space(10.);
                    ComboBox::from_label("Group").show_index(
                        ui,
                        &mut self.selected_group,
                        groups.len(),
                        |i| &groups[i].name,
                    );
                    ui.add_space(10.);

                    let button = ui.add_sized((100., 40.), Button::new("upload"));

                    if button.clicked() {
                        tokio::spawn(upload_file(
                            filename.clone(),
                            self.status.clone(),
                            groups[self.selected_group].clone(),
                        ));
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
                    let group = self.groups.read().unwrap()[self.selected_group].clone();
                    if button.clicked() {
                        tokio::spawn(upload_file(filename.clone(), self.status.clone(), group));
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
                        self.status.set(FileStatus::Unselected);
                    }
                });
            }
            FileStatus::LoadingMd => {
                ui.vertical_centered(|ui| {
                    ui.heading(format!("Loading meta"));
                });
            }
            FileStatus::FailedMd(filename, error) => {
                ui.heading("Failed loading meta");
                let button = ui.add_sized((100., 40.), Button::new("retry"));

                if button.clicked() {
                    tokio::spawn(load_md(filename, self.status.clone(), self.groups.clone()));
                    self.status.set(FileStatus::Sending);
                }
                ui.label(error);
            }
        });
    }
}

async fn upload_file(filename: String, board: Arc<Pinboard<FileStatus>>, group: Group) {
    async fn inner(filename: String, group: Group) -> Result<()> {
        let bytes = std::fs::read(&filename)?;

        let localdata = localdata::get_localdata();

        let parts = Form::new()
            .part("group_id", Part::text(group.id.to_string()))
            .part("sender", Part::text(localdata.username))
            .part("file", Part::bytes(bytes).file_name(filename));

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

    if let Err(err) = inner(filename.clone(), group).await {
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

async fn load_md(
    file: Option<String>,
    board: Arc<Pinboard<FileStatus>>,
    groups: Arc<Pinboard<Vec<Group>>>,
) {
    async fn load_inner(
        file: Option<String>,
        board: &Pinboard<FileStatus>,
        group_board: Arc<Pinboard<Vec<Group>>>,
    ) -> Result<()> {
        let localdata = localdata::get_localdata();
        let resp = reqwest::get(format!(
            "http://koebstoffer.info:3987/get_md/{}",
            localdata.user_id
        ))
        .await?;
        let groups: Vec<Group> = serde_json::from_str(&resp.text().await?)?;
        group_board.set(groups);
        if let Some(file) = file {
            board.set(FileStatus::Selected(file));
        } else {
            board.set(FileStatus::Unselected);
        }

        Ok(())
    }
    if let Err(err) = load_inner(file.clone(), &board, groups).await {
        board.set(FileStatus::FailedMd(file, err.to_string()))
    }
}
