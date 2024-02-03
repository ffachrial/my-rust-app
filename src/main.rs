use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use sqlx::sqlite::SqlitePool;
use serde::{Serialize, Deserialize};

#[derive(sqlx::FromRow, Serialize, Deserialize)]
struct Item {
    id:     i32,
    name:   String,
}

async fn get_items(db_pool: web::Data<SqlitePool>) -> impl Responder {
    let result = sqlx::query_as::<_, Item>("SELECT * from items")
        .fetch_all(&**db_pool)
        .await;

    match result {
        Ok(items) => HttpResponse::Ok().json(items),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

async fn get_item_by_id(db_pool: web::Data<SqlitePool>, path: web::Path<i32>) -> impl Responder {
    let result = sqlx::query_as::<_, Item>("SELECT * FROM items WHERE id = ?")
        .bind(path.into_inner())
        .fetch_one(&**db_pool)
        .await;

    match result {
        Ok(item) => HttpResponse::Ok().json(item),
        Err(_) => HttpResponse::NotFound().finish(),
    }
}

async fn create_item(db_pool: web::Data<SqlitePool>, item: web::Json<Item>) -> impl Responder {
    let result = sqlx::query("INSERT INTO items (name) VALUES (?)")
        .bind(&item.name)
        .execute(&**db_pool)
        .await;

    match result {
        Ok(_) => HttpResponse::Created().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

async fn update_item(db_pool: web::Data<SqlitePool>, path: web::Path<i32>, item: web::Json<Item>) -> impl Responder {
    let result = sqlx::query("UPDATE items SET name = ? WHERE id = ?")
        .bind(&item.name)
        .bind(path.into_inner())
        .execute(&**db_pool)
        .await;

    match result {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

async fn delete_item(db_pool: web::Data<SqlitePool>, path: web::Path<i32>) -> impl Responder {
    let result = sqlx::query("DELETE FROM items WHERE id = ?")
        .bind(path.into_inner())
        .execute(&**db_pool)
        .await;

    match result {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let db_pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS items (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL
        )
        "#,
    )
    .execute(&db_pool)
    .await
    .unwrap();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(db_pool.clone()))
            .route("/items", web::get().to(get_items))
            .route("/items/{id}", web::get().to(get_item_by_id))
            .route("/items", web::post().to(create_item))
            .route("/items/{id}", web::put().to(update_item))
            .route("/items/{id}", web::delete().to(delete_item))
    })
    .bind("127.0.0.1:8085")?
    .run()
    .await
}