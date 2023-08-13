use uuid::Uuid;
pub mod localdata;
#[derive(Debug, serde::Deserialize, serde::Serialize, Clone, Default)]
pub struct EventData {
    pub filename: String,
    pub file_id: Uuid,
    pub groupname: String,
    pub group_id: Uuid,
    pub sender: String,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Group {
    pub name: String,
    pub id: Uuid,
}
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct CreateGroupForm {
    pub user_id: Uuid,
    pub name: String,
}
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct JoinGroupForm {
    pub user_id: Uuid,
    pub group_id: Uuid,
}
