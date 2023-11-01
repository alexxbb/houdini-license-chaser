#![allow(unused)]

mod request;
mod response;

use response::License;

use anyhow::{Context, Result};
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

// https://www.sidefx.com/docs/houdini/ref/utils/sesinetd.html

const API_ENDPOINT: &str = "http://vmlic-4:1715/api";

fn main() -> Result<()> {
    let _ = dotenv::dotenv()?;
    let hfs = PathBuf::from(std::env::var("HFS").context("HFS Variable not set")?);

    let client = reqwest::blocking::Client::builder().build()?;

    #[derive(Serialize)]
    struct Keys(String, Vec<()>, HashMap<String, bool>);
    let keys = Keys(
        "cmd_ls".to_string(),
        vec![],
        HashMap::from([("show_licenses".to_string(), true)]),
    );

    let mut params = HashMap::new();
    params.insert("json", serde_json::to_string(&keys)?);

    let response = client
        .post(API_ENDPOINT)
        .header(
            CONTENT_TYPE,
            HeaderValue::from_static("application/x-www-urlencoded"),
        )
        .form(&params)
        .send()?;
    let response: HashMap<String, Vec<License>> = response.json()?;
    dbg!(&response[&String::from("licenses")]);

    Ok(())
}
