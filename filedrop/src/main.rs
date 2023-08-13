use anyhow::{Ok, Result};
use eframe::{CreationContext, NativeOptions};
use egui::{Button, CentralPanel, ComboBox, Context, TextEdit};
use filedrop_lib::{CreateGroupForm, Group, JoinGroupForm};
use localdata::get_localdata;
use pinboard::Pinboard;
use reqwest::{
    multipart::{Form, Part},
    StatusCode,
};
use std::{env::args, fmt::Display, str::FromStr, sync::Arc};
use tinyfiledialogs::open_file_dialog;
use uuid::Uuid;

mod localdata;

#[tokio::main]
async fn main() {
    let file = args().nth(1);

    let native_options = NativeOptions {
        initial_window_size: Some((250., 300.).into()),
        initial_window_pos: Some((1920., 1080. / 2.).into()), //Virker ikke?
        resizable: false,
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
    username_input: String,
    create_group_input: String,
    join_group_input: String,
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
            username_input: get_localdata().username,
            create_group_input: String::new(),
            join_group_input: String::new(),
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
                    if !groups.is_empty() {
                        ComboBox::from_label("Group").show_index(
                            ui,
                            &mut self.selected_group,
                            groups.len(),
                            |i| &groups[i].name,
                        );
                        if ui
                            .button(format!("join id: {}", groups[self.selected_group].id))
                            .on_hover_text("Click to copy")
                            .clicked()
                        {
                            ui.output_mut(|o| {
                                o.copied_text = groups[self.selected_group].id.to_string()
                            });
                        }

                        ui.add_space(10.);
                        ui.label("username");
                        let input = ui.text_edit_singleline(&mut self.username_input);
                        if input.changed() {
                            localdata::set_username(self.username_input.clone());
                        }

                        ui.add_space(10.);

                        let button = ui.add_sized((100., 40.), Button::new("upload"));

                        if button.clicked() {
                            tokio::spawn(upload_file(
                                filename.clone(),
                                self.status.clone(),
                                groups[self.selected_group].clone(),
                                ui.ctx().clone(),
                            ));
                            self.status.set(FileStatus::Sending);
                        }
                    } else {
                        ui.label("Please join or create a group before sending files.");
                    }

                    ui.add_space(30.);
                    ui.heading("Create group");
                    ui.horizontal(|ui| {
                        ui.add_sized(
                            (200., 20.),
                            TextEdit::singleline(&mut self.create_group_input),
                        );

                        if ui.button("Create").clicked() {
                            //Create group
                            tokio::spawn(create_group(
                                self.create_group_input.clone(),
                                filename.clone(),
                                self.status.clone(),
                                self.groups.clone(),
                            ));
                            self.create_group_input.clear();
                        }
                    });

                    ui.heading("Join group");
                    ui.horizontal(|ui| {
                        ui.add_sized(
                            (200., 20.),
                            TextEdit::singleline(&mut self.join_group_input),
                        );
                        if ui.button("Join").clicked() {
                            //Join group
                            tokio::spawn(join_group(
                                self.join_group_input.clone(),
                                filename,
                                self.status.clone(),
                                self.groups.clone(),
                            ));
                            self.join_group_input.clear();
                        }
                    });
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
                        self.status.set(FileStatus::Selected(filename));
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

async fn upload_file(
    filename: String,
    board: Arc<Pinboard<FileStatus>>,
    group: Group,
    ctx: Context,
) {
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
    ctx.request_repaint();
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

async fn create_group(
    name: String,
    file: String,
    board: Arc<Pinboard<FileStatus>>,
    groups: Arc<Pinboard<Vec<Group>>>,
) {
    let form = CreateGroupForm {
        user_id: get_localdata().user_id,
        name,
    };

    board.set(FileStatus::LoadingMd);
    let res = reqwest::Client::new()
        .post("http://koebstoffer.info:3987/create")
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&form).unwrap())
        .send()
        .await;
    dbg!(&res);
    if let Err(err) = res {
        board.set(FileStatus::Failed(file, err.to_string()))
    } else {
        load_md(Some(file), board, groups).await;
    }
}

async fn join_group(
    raw_id: String,
    file: String,
    board: Arc<Pinboard<FileStatus>>,
    groups: Arc<Pinboard<Vec<Group>>>,
) {
    let Result::Ok(group_id) = Uuid::from_str(&raw_id) else {
        board.set(FileStatus::Failed(file, "Failed to parse group id".into()));
        return;
    };

    let form = JoinGroupForm {
        user_id: get_localdata().user_id,
        group_id,
    };
    board.set(FileStatus::LoadingMd);

    if let Err(err) = reqwest::Client::new()
        .post("http://koebstoffer.info:3987/join")
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&form).unwrap())
        .send()
        .await
    {
        board.set(FileStatus::Failed(file, err.to_string()))
    } else {
        load_md(Some(file), board, groups).await;
    }
}
