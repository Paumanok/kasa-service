use std::process::Output;

use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;

use rust_kasa::{
    device::{self, Device},
    models,
};

#[tokio::main]
async fn main() {
    println!("hello world");
    let mut state = DeviceState::new();

    if let Ok(dev) = device::discover_multiple() {
        for d in dev {
            state.devs.push(d);
        }
    }
    // initialize tracing
    //tracing_subscriber::fmt::init();
    // build our application with a route
    let app = Router::new()
        .route("/", get(root))
        .route("/discover", get(discover_devices))
        .route("/plugs", get(get_plugs))
        .route("/toggle", post(toggle_outlet))
        .with_state(state);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:4000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Hello, World!"
}

async fn get_plugs(
    State(DeviceState { devs }): State<DeviceState>,
) -> (StatusCode, Json<Plugs>) {

    let dev = match devs.len() {
        len if len > 0 => devs[0].clone(),
        _ => match device::determine_target("".to_string()) {
            Ok(dev) => dev,
            Err(_err) => {
                    return (StatusCode::NOT_FOUND, Json(vec![]));
                },
            },

    };

    if let  Some(plugs) = dev.get_children() {
        return (StatusCode::OK, Json(plugs))
    }
       
    return (StatusCode::NOT_FOUND, Json(vec![]));
}

async fn discover_devices(
State(DeviceState { devs }): State<DeviceState>,
) ->(StatusCode, Json<Vec<models::SysInfo>>) {
    if let Ok(dev_list) = device::discover_multiple() {
         if dev_list.len() > 0 {
            let mut discovered: Vec<models::SysInfo> = vec![];
            for d in dev_list {
                match d.sysinfo() {
                    Some(disc) => discovered.push(disc),
                    _ => (),
                }
            }
            return (StatusCode::OK, Json(discovered));
            
        }
    } 
    
    return (StatusCode::NOT_FOUND, Json(vec![]));
}

async fn toggle_outlet(Json(payload): Json<Outlet>) -> StatusCode {
    
    if let Ok(dev) = device::determine_target(payload.ip_addr) {
        if let Some(idx) = payload.idx {
            println!("toggle idx");
            dev.toggle_relay_by_id(idx);
        } else {
            println!("toggle single");
            dev.toggle_single_relay();
        }
        return StatusCode::OK;
    }


    return StatusCode::NOT_FOUND
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

type OutletDevices = Vec<models::KasaResp>;

#[derive(Deserialize)]
struct Index {
    idx: u32,
}

#[derive(Deserialize)]
struct Outlet {
    ip_addr: String,
    idx: Option<usize>,
}

#[derive(Clone)]
struct DeviceState {
    devs: Vec<Device>,
}

impl DeviceState {
    pub fn new() -> Self {
        Self { devs: vec![] }
    }
}
