use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub enum Product {
    #[serde(rename = "Houdini-Master")]
    HoudiniFx,
    #[serde(rename = "Houdini-Escape")]
    HoudiniCore,
    #[serde(rename = "Karma-Render")]
    KarmaRenderer,
    #[serde(rename = "Render")]
    Render,
    #[serde(rename = "Houdini-Engine")]
    HoudiniEngine,
}

#[derive(Deserialize, Debug)]
pub struct User {
    id: i32,
    machine: String,
    time: String,
}

#[derive(Deserialize, Debug)]
pub struct License {
    id: String,
    platform: String,
    product: String,
    product_id: Product,
    version: String,
    available: i32,
    total_tokens: i32,
    expires: String,
    ip_mask: String,
    ipmatch: bool,
    servers: String,
    signature: String,
    license_access_mode: String,
}
