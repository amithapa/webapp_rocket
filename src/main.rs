#[macro_use]
extern crate rocket;

use rocket::{
    http::Status,
    response::{self, Responder},
    serde::{json::Json, Deserialize, Serialize},
    Request,
};
use rocket_db_pools::{Connection, Database};

#[derive(Deserialize, Serialize, sqlx::FromRow)]
#[serde(crate = "rocket::serde")]
struct Task {
    id: i64,
    item: String,
}

#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
struct TaskItem<'r> {
    item: &'r str,
}

#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
struct TaskId {
    id: i64,
}

struct DatabaseError(rocket_db_pools::sqlx::Error);

impl<'r> Responder<'r, 'r> for DatabaseError {
    fn respond_to(self, request: &Request) -> response::Result<'r> {
        Err(Status::InternalServerError)
    }
}

impl From<rocket_db_pools::sqlx::Error> for DatabaseError {
    fn from(error: rocket_db_pools::sqlx::Error) -> Self {
        DatabaseError(error)
    }
}

#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
struct TaskUpdate<'r> {
    id: i64,
    item: &'r str,
}

#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
struct TaskDelete {
    id: i64,
}

#[derive(Database)]
#[database("todo")]
struct TodoDatabase(sqlx::PgPool);

#[post("/addtask", data = "<task>")]
async fn add_task(
    task: Json<TaskItem<'_>>,
    mut db: Connection<TodoDatabase>,
) -> Result<Json<Task>, DatabaseError> {
    let added_task = sqlx::query_as::<_, Task>("INSERT INTO tasks (item) VALUES ($1) RETURNING *")
        .bind(task.item)
        .fetch_one(&mut *db)
        .await?;

    Ok(Json(added_task))
}

#[get("/readtasks")]
async fn read_tasks(mut db: Connection<TodoDatabase>) -> Result<Json<Vec<Task>>, DatabaseError> {
    let all_tasks = sqlx::query_as::<_, Task>("SELECT * FROM tasks").fetch_all(&mut *db).await?;
    Ok(Json(all_tasks))
}

#[put("/edittask", data = "<task_update>")]
async fn edit_task(task_update: Json<TaskUpdate<'_>>, mut db: Connection<TodoDatabase>) -> Result<Json<Task>, DatabaseError> {
    let updated_task = sqlx::query_as::<_, Task>("UPDATE tasks SET item = $1 WHERE id = $2 RETURNING *").bind(&task_update.item).bind(&task_update.id).fetch_one(&mut *db).await?;
    Ok(Json(updated_task))
}

#[delete("/deletetask", data = "<task_delete>")]
async fn delete_task(task_delete: Json<TaskDelete>, mut db: Connection<TodoDatabase>) -> Result<Json<Task>, DatabaseError> {
    let deleted_task = sqlx::query_as::<_, Task>("DELETE FROM tasks WHERE id = $1 RETURNING *").bind(&task_delete.id).fetch_one(&mut *db).await?;
    Ok(Json(deleted_task))
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[launch]
fn rocket() -> _ {
    rocket::build().attach(TodoDatabase::init()).mount(
        "/",
        routes![index, add_task, read_tasks, edit_task, delete_task],
    )
}
