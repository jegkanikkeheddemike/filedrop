use std::str::FromStr;

use axum::{extract::Path, http::StatusCode, Json};
use filedrop_lib::{CreateGroupForm, Group, JoinGroupForm};
use uuid::Uuid;

use crate::db;

pub async fn get_user(Path(user_id): Path<String>) -> Json<Vec<Group>> {
    let groups = sqlx::query!(
        "
        select name, id from group_members 
            join groups on group_id = groups.id 
            where user_id = $1",
        &user_id
    )
    .fetch_all(db::get())
    .await
    .unwrap();

    let groups = groups
        .into_iter()
        .filter_map(|g| {
            Some(Group {
                name: g.name?,
                id: Uuid::from_str(&g.id).ok()?,
            })
        })
        .collect();
    Json(groups)
}

pub async fn join_group(Json(form): Json<JoinGroupForm>) -> StatusCode {
    //Check if already exists
    if sqlx::query!(
        "select user_id from group_members where group_id = $1 and user_id = $2",
        &form.group_id.to_string(),
        &form.user_id.to_string()
    )
    .fetch_optional(db::get())
    .await
    .unwrap()
    .is_some()
    {
        //The member already exists. Dont make a duplicate
        return StatusCode::CONFLICT;
    }

    sqlx::query!(
        "insert into group_members (group_id, user_id) values ($1,$2)",
        &form.group_id.to_string(),
        &form.user_id.to_string()
    )
    .execute(db::get())
    .await
    .unwrap();

    StatusCode::OK
}

pub async fn create_group(Json(form): Json<CreateGroupForm>) -> StatusCode {
    let group_id = Uuid::new_v4();
    sqlx::query!(
        "insert into groups (name, id) values ($1,$2)",
        &form.name,
        &group_id.to_string()
    )
    .execute(db::get())
    .await
    .unwrap();

    sqlx::query!(
        "insert into group_members (group_id, user_id) values ($1,$2)",
        &group_id.to_string(),
        &form.user_id.to_string()
    )
    .execute(db::get())
    .await
    .unwrap();

    StatusCode::OK
}
