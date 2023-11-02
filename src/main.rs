#![allow(unused)]

mod request;
mod response;

use response::License;

use crate::response::{HoudiniVersion, Product};
use anyhow::{Context, Result};
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

// https://www.sidefx.com/docs/houdini/ref/utils/sesinetd.html

const API_ENDPOINT: &str = "http://vmlic-4:1715/api";

#[derive(Serialize)]
struct Keys(String, Vec<()>, HashMap<String, bool>);

fn main() -> Result<()> {
    let _ = dotenv::dotenv()?;
    let hfs = PathBuf::from(std::env::var("HFS").context("HFS Variable not set")?);

    let (ctrc_tx, ctrlc_rx) = std::sync::mpsc::sync_channel(100);
    ctrlc::set_handler(move || {
        eprintln!("Sending Ctrl-C");
        let _ = ctrc_tx.send(());
    });

    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let client = reqwest::blocking::Client::builder().build().unwrap();

        let keys = Keys(
            "cmd_ls".to_string(),
            vec![],
            HashMap::from([("show_licenses".to_string(), true)]),
        );
        let mut params = HashMap::new();
        params.insert("json", serde_json::to_string(&keys).unwrap());
        loop {
            if let Ok(_) = ctrlc_rx.try_recv() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_secs(3));
            eprintln!("Sending server request");
            let response = client
                .post(API_ENDPOINT)
                .header(
                    CONTENT_TYPE,
                    HeaderValue::from_static("application/x-www-urlencoded"),
                )
                .form(&params)
                .send()
                .unwrap();
            let response: HashMap<String, Vec<License>> = response.json().unwrap();
            tx.send(response).expect("data sent");
        }
    });

    while let Ok(data) = rx.recv() {
        let licenses = &data[&String::from("licenses")];
        let available_core_lic = licenses
            .iter()
            .filter_map(|lic| match lic.product_id {
                Product::HoudiniCore if lic.version.major == 20 => Some(lic.available),
                _ => None,
            })
            .sum::<i32>();
        dbg!(available_core_lic);
    }

    Ok(())
}
