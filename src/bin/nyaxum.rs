use axum::{Json, Router, extract::{Path, Query}, http::StatusCode, routing::{get, post}};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;

#[derive(Deserialize)]
struct PageAuthor {
    nama : String,
    author : String,
}

#[derive(Serialize)]
struct IdUser{
    status : String,
    id : String,
}

async fn get_comment_byid (Path(id): Path<String>) -> (StatusCode, Json<IdUser>) {
    println!("Succes get id : {}", id);
    return (StatusCode::OK, Json(IdUser { status: "Succes".to_string(), id }));
}

async fn see_page_author (Query(pageauthor): Query<PageAuthor>)-> String{
    format!("Nama : {} author : {}", pageauthor.nama , pageauthor.author)
}

async fn make_page_author (Json(pageauthor): Json<PageAuthor>) -> String{
    format!("Nama : {} author : {}", pageauthor.nama , pageauthor.author)
}


//192.168.0.26
#[tokio::main]
async fn main() {
    let result_listener = TcpListener::bind("0.0.0.0:2736").await;

    let listener = match result_listener {
        Ok(listenar) => listenar,
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    };

    //route nesting
    let service_user = Router::new()
        .route("/{id}", get(get_comment_byid));
    let app_service = Router::new()
        //nesting
        .nest("/user/", service_user)
        .route("/comment/{id}", get(get_comment_byid))
        // penggunaan http://192.168.0.26:2736/page/?nama=Dadang&author=Dydy pakai parameter
        // untuk post pakai json
        .route("/page/", get(see_page_author).post(make_page_author));

    let axum_serv = axum::serve(listener, app_service).await;
    match axum_serv {
        Ok(_) => println!("Program Jalan"),
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    }
}
