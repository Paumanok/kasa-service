use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use rust_kasa::{
    device::{self, Device},
    kasa_protocol, models,
};

#[tokio::main]
async fn main() {
    let mut state = DeviceState::new();

    if let Ok(dev) = device::determine_target("".to_string()) {
        state.devs.push(dev);
    }
    // initialize tracing
    //tracing_subscriber::fmt::init();
    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        // `POST /users` goes to `create_user`
        .route("/plugs", get(get_plugs))
        .route("/toggle", post(toggle_plug))
        .with_state(state);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Hello, World!"
}

async fn get_plugs(// this argument tells axum to parse the request body
    // as JSON into a `CreateUser` type
    //Json(payload): Json<CreateUser>,
) -> (StatusCode, Json<Plugs>) {
    if let Ok(dev) = device::determine_target("".to_string()) {
        if let Some(plugs) = dev.get_children() {
            return (StatusCode::CREATED, Json(plugs));
        }
    }
    return (StatusCode::NOT_FOUND, Json(vec![]));
}

async fn toggle_plug(
    State(DeviceState { devs }): State<DeviceState>,
    Json(payload): Json<Index>,
) -> StatusCode {
    println!("togglin");

    if devs.len() > 0 {
        //just take first for now if it exists
        println!("theres a device");
        devs[0].clone().toggle_relay_by_id(payload.idx as usize);
        return StatusCode::OK;

    } else if devs.len()== 0 {

        if let Ok( dev) = device::determine_target("".to_string()) {
            dev.toggle_relay_by_id(payload.idx as usize);
            return StatusCode::OK;
        }
    }

    println!("failed");
    return StatusCode::NOT_FOUND;
}

type Plugs = Vec<models::KasaChildren>;

#[derive(Deserialize)]
struct Index {
    idx: u32,
}

#[derive(Clone)]
struct DeviceState {
    devs: Vec<device::Device>,
}

impl DeviceState {
    pub fn new() -> Self {
        Self { devs: vec![] }
    }
}
