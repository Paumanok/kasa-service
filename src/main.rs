use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use axum_macros::debug_handler;
use serde::Deserialize;

use rust_kasa::{
    device::{self, Device},
    models,
};

use std::sync::{Arc, Mutex};

//type Plugs = Vec<models::KasaChildren>;

type OutletDevices = Vec<models::KasaResp>;

#[derive(Deserialize)]
struct Index {
    idx: u32,
}

#[derive(Deserialize)]
struct Outlet {
    ip_addr: Option<String>,
    alias: Option<String>,
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

#[tokio::main]
async fn main() {
    println!("hello world");
    let state = Arc::new(Mutex::new(DeviceState::new()));

    if let Ok(dev) = device::discover() {
        if let Ok(mut this_state) = state.lock() {
            for d in dev {
                this_state.devs.push(d);
            }

        }
    }
    // build our application with a route
    let app = Router::new()
        .route("/", get(discover_devices))
        .route("/discover", get(discover_devices))
        //.route("/plugs", get(get_plugs))
        .route("/toggle", post(toggle_outlet))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:4000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn discover_devices(
) -> (StatusCode, Json<Vec<models::SysInfo>>) {
    if let Ok(dev_list) = device::discover() {
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

fn alias_exists_with_refresh(dev_state: &mut DeviceState, alias: &String) -> bool {
   if find_dev_with_alias(dev_state , alias).is_some() {
        return true;
    } else {

        dev_state.devs.clear();
        if let Ok(new_devs) = device::discover() {
            for d in new_devs {
                dev_state.devs.push(d);
            }
        }
        
        //try again with updated state
        if find_dev_with_alias(dev_state, alias).is_some() {
            return true;
        }
        
    }
    return false;
}


fn find_dev_with_alias(devs: &DeviceState, alias: &String ) -> Option<Device> {
    for dev in &devs.devs {
        if dev.sysinfo()?.alias == *alias {
            return Some(dev.clone());
        }
    }
    None
}

fn find_dev_with_ip(devs: &DeviceState, ip: &String) -> Option<Device> {
    for dev in &devs.devs {
        if dev.ip_addr == *ip {
            return Some(dev.clone());
        }
    }
    None
}
#[debug_handler]
async fn toggle_outlet(
    State(state): State<Arc<Mutex<DeviceState>>>,
    Json(payload): Json<Outlet>) -> StatusCode {

    let mut target: Option<Device> = None;
    
    if let Ok(mut this_state) = state.lock() {
        //user set alias field, see if it exists
        if let Some(alias) = payload.alias {
            if alias_exists_with_refresh(&mut this_state, &alias) {
                target = find_dev_with_alias(&mut this_state,  &alias);
            }
        }
        //if user set ip, or set ip and alias but alias lookup failed
        if let (Some(ip), None) = (payload.ip_addr, target.clone()) {
            // first check if ip in devices
            if let Some(new_t) = find_dev_with_ip(&mut this_state, &ip){
                target = Some(new_t);
            //if not, lets see if its on the network
            } else if let Ok(new_t) =  device::determine_target(ip) {
                target = Some(new_t);
            }

        }

        if let Some(t) = target {
            //we have a target
            // now, did user provide an idx? if so, does the device support idx'd outlets?
            if t.has_children() {
                if let Some(idx) = payload.idx {
                    t.toggle_relay_by_id(idx);
                    return StatusCode::OK;
                }
            } else {
                t.toggle_single_relay();
                return StatusCode::OK;
            }

        } else {
            //both attempts failed
            return StatusCode::NOT_FOUND;
        }

    } else {
        return StatusCode::NOT_FOUND;
    }

    return StatusCode::NOT_FOUND;
}

