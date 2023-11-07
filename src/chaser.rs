// https://www.sidefx.com/docs/houdini/ref/utils/sesinetd.html

use crate::app::Message;
use crate::response::{HoudiniVersion, License, Product};
use anyhow::{Context, Result};
use iced::futures::channel::mpsc;
use iced::futures::{SinkExt, StreamExt};
use iced::subscription;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::fmt::Display;
use std::ops::Add;
use std::sync::Arc;
use std::sync::OnceLock;
use std::time::Duration;

const API_ENDPOINT: &str = "http://vmlic-4:1715/api";

#[derive(Serialize)]
struct Keys(String, Vec<()>, HashMap<String, bool>);

pub struct Chaser {}

impl Chaser {
    pub fn new() -> Self {
        Self {}
    }

    pub fn start() {}

    pub fn stop() {}
}

enum ChaserState {
    Starting,
    Working,
}

#[derive(Debug, Clone)]
pub enum ChaserEvent {
    ServerStarted,
    ServerResponse(Arc<HashMap<String, Vec<License>>>),
    ServerErrored,
}

static PARAMS: OnceLock<HashMap<String, String>> = OnceLock::new();

pub fn subscribe() -> subscription::Subscription<Message> {
    subscription::unfold(
        std::any::TypeId::of::<Chaser>(),
        ChaserState::Starting,
        |state| async move {
            match state {
                ChaserState::Starting => (
                    Message::ChaserEvent(ChaserEvent::ServerStarted),
                    ChaserState::Working,
                ),
                ChaserState::Working => {
                    tokio::time::sleep(Duration::from_secs(3)).await;

                    let client = reqwest::Client::builder().build().unwrap();

                    let params = PARAMS.get_or_init(|| {
                        let keys = Keys(
                            "cmd_ls".to_string(),
                            vec![],
                            HashMap::from([("show_licenses".to_string(), true)]),
                        );
                        let mut params = HashMap::new();
                        params.insert(String::from("json"), serde_json::to_string(&keys).unwrap());
                        params
                    });
                    let request = client
                        .post(API_ENDPOINT)
                        .header(
                            CONTENT_TYPE,
                            HeaderValue::from_static("application/x-www-urlencoded"),
                        )
                        .form(&params);

                    match request.send().await {
                        Ok(response) => {
                            match response.json::<HashMap<String, Vec<License>>>().await {
                                Ok(resp) => (
                                    Message::ChaserEvent(ChaserEvent::ServerResponse(Arc::new(
                                        resp,
                                    ))),
                                    ChaserState::Working,
                                ),
                                Err(e) => {
                                    eprintln!("Deserialize error: {e:?}");
                                    // TODO :Better error reporting
                                    (
                                        Message::ChaserEvent(ChaserEvent::ServerErrored),
                                        ChaserState::Working,
                                    )
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Server response error: {e:?}");
                            // TODO :Better error reporting
                            (
                                Message::ChaserEvent(ChaserEvent::ServerErrored),
                                ChaserState::Working,
                            )
                        }
                    }
                }
            }
        },
    )
}

// fn main2() -> Result<()> {
//     let (mut ctrc_tx, ctrlc_rx) = mpsc::channel(100);
//     ctrlc::set_handler(move || {
//         eprintln!("Sending Ctrl-C");
//         let _ = ctrc_tx.send(());
//     });
//
//     let (mut tx, rx) = mpsc::channel(100);
//     std::thread::spawn(move || {
//         let client = reqwest::blocking::Client::builder().build().unwrap();
//
//         let keys = Keys(
//             "cmd_ls".to_string(),
//             vec![],
//             HashMap::from([("show_licenses".to_string(), true)]),
//         );
//         let mut params = HashMap::new();
//         params.insert("json", serde_json::to_string(&keys).unwrap());
//         'outer: loop {
//             let _now = std::time::Instant::now();
//             let then = _now.add(std::time::Duration::from_secs(3));
//             while std::time::Instant::now() < then {
//                 if let Ok(_) = ctrlc_rx.try_recv() {
//                     break 'outer;
//                 }
//             }
//             eprintln!("Sending server request");
//             let response = client
//                 .post(API_ENDPOINT)
//                 .header(
//                     CONTENT_TYPE,
//                     HeaderValue::from_static("application/x-www-urlencoded"),
//                 )
//                 .form(&params);
//             match response.send() {
//                 Ok(response) => {
//                     let response: HashMap<String, Vec<License>> = response.json().unwrap();
//                     tx.send(response).expect("data sent");
//                 }
//                 Err(e) => {
//                     eprintln!("{}", e);
//                 }
//             }
//         }
//     });
//
//     while let Ok(data) = rx.recv() {
//         let licenses = &data[&String::from("licenses")];
//         let available_core_lic = licenses
//             .iter()
//             .filter_map(|lic| match lic.product_id {
//                 Product::HoudiniCore if lic.version.major == 20 => Some(lic.available),
//                 _ => None,
//             })
//             .sum::<i32>();
//         dbg!(available_core_lic);
//     }
//     println!("Quitting");
//     Ok(())
// }
