use axum::{
    routing::{get, post},
    http::StatusCode,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use anyhow::{anyhow, Result};

use rust_kasa::{device, models, kasa_protocol};

#[tokio::main]
async fn main() {
    // initialize tracing
    //tracing_subscriber::fmt::init();

     

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        // `POST /users` goes to `create_user`
        .route("/plugs", get(get_plugs))
        .route("/toggle", post(toggle_plug));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Hello, World!"
}

async fn get_plugs(
    // this argument tells axum to parse the request body
    // as JSON into a `CreateUser` type
    //Json(payload): Json<CreateUser>,
) -> (StatusCode, Json<Plugs>) {

    if let Ok(dev) = device::determine_target("".to_string()){
        if let Some(plugs) = dev.get_children() {
            return (StatusCode::CREATED, Json(plugs));
        }
    }
    return (StatusCode::NOT_FOUND, Json(vec![]));
}

async fn toggle_plug(Json(payload): Json<Index>) -> StatusCode {
    println!("togglin"); 
    if let Ok(dev) = device::determine_target("".to_string()){
        println!("do we find em");
        //let children = dev.get_children();
        //if let Some(plugs) = children {
        dev.toggle_relay_by_id(payload.idx as usize);
        //return (StatusCode::CREATED, Json(plugs[payload as usize]));
        return StatusCode::CREATED;
        //}
    }
    println!("failed");
    return StatusCode::NOT_FOUND;
}

type Plugs = Vec<models::KasaChildren>;

#[derive(Deserialize)]
struct Index {
    idx: u32,
}

// the input to our `create_user` handler
#[derive(Deserialize)]
struct CreateUser {
    username: String,
}

// the output to our `create_user` handler
#[derive(Serialize)]
struct User {
    id: u64,
    username: String,
}
