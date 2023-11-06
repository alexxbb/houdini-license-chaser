// https://www.sidefx.com/docs/houdini/ref/utils/sesinetd.html

use crate::response::{HoudiniVersion, License, Product};
use anyhow::{Context, Result};
use iced::futures::channel::mpsc;
use iced::futures::{SinkExt, StreamExt};
use iced::subscription;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::ops::Add;
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
    Ready(mpsc::Receiver<()>),
}

pub fn subscribe() -> subscription::Subscription<crate::app::Message> {
    subscription::channel(
        std::any::TypeId::of::<Chaser>(),
        100,
        |mut sender| async move {
            let mut state = ChaserState::Starting;
            loop {
                match &mut state {
                    ChaserState::Starting => {
                        let (tx, rx) = mpsc::channel(100);
                        sender.send(crate::app::Message::ChaserStarted(tx)).await;
                        state = ChaserState::Ready(rx);
                    }
                    ChaserState::Ready(app_events_receiver) => {
                        use iced::futures::StreamExt;
                        tokio::time::sleep(Duration::from_secs(3)).await;
                        let r = app_events_receiver.try_next();
                        let g = reqwest::get("https://google.com").await;
                        dbg!(g.map(|_r| _r.status()));
                        // let input = receiver.select_next_some().await;
                        eprintln!("Received event from app: {r:?}");
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
