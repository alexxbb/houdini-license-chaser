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

#[derive(Serialize)]
struct Keys(String, Vec<()>, HashMap<String, bool>);

#[derive(Hash)]
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

pub fn subscribe(server_url: Arc<str>) -> subscription::Subscription<Message> {
    subscription::unfold(
        std::any::TypeId::of::<ChaserState>(),
        ChaserState::Starting,
        move |state| {
            let server_url = Arc::clone(&server_url);
            async move {
                match state {
                    ChaserState::Starting => (
                        Message::ChaserEvent(ChaserEvent::ServerStarted),
                        ChaserState::Working,
                    ),
                    ChaserState::Working => {
                        tokio::time::sleep(Duration::from_secs(2)).await;

                        let client = reqwest::Client::builder().build().unwrap();

                        let params = PARAMS.get_or_init(|| {
                            let keys = Keys(
                                "cmd_ls".to_string(),
                                vec![],
                                HashMap::from([("show_licenses".to_string(), true)]),
                            );
                            let mut params = HashMap::new();
                            params.insert(
                                String::from("json"),
                                serde_json::to_string(&keys).expect("Keys should be valid JSON"),
                            );
                            params
                        });
                        let request = client
                            .post(server_url.as_ref())
                            .header(
                                CONTENT_TYPE,
                                HeaderValue::from_static("application/x-www-urlencoded"),
                            )
                            .form(&params);

                        match request.send().await {
                            Ok(response) => {
                                match response.json::<HashMap<String, Vec<License>>>().await {
                                    Ok(resp) => (
                                        Message::ChaserEvent(ChaserEvent::ServerResponse(
                                            Arc::new(resp),
                                        )),
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
            }
        },
    )
}
