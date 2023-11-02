use serde::{de::Error, Deserialize, Deserializer};

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
    pub id: i32,
    pub machine: String,
    pub time: String,
}

#[derive(Deserialize, Debug)]
pub struct HoudiniVersion {
    pub major: u8,
    pub minor: u8,
}

fn parse_version<'de, D: Deserializer<'de>>(des: D) -> Result<HoudiniVersion, D::Error> {
    let buf = String::deserialize(des)?;

    buf.split_once(".")
        .ok_or(Error::custom("Could not parse version"))
        .map(|(major, minor)| {
            let major = major
                .parse::<u8>()
                .map_err(|_| Error::custom("Could not parse major version"))?;
            let minor = minor
                .parse::<u8>()
                .map_err(|_| Error::custom("Could not parse minor version"))?;
            Ok(HoudiniVersion { major, minor })
        })?
}

#[derive(Deserialize, Debug)]
pub struct License {
    pub id: String,
    pub platform: String,
    pub product: String,
    pub product_id: Product,
    #[serde(deserialize_with = "parse_version")]
    pub version: HoudiniVersion,
    pub available: i32,
    pub total_tokens: i32,
    pub expires: String,
    pub ip_mask: String,
    pub ipmatch: bool,
    pub servers: String,
    pub signature: String,
    pub license_access_mode: String,
}
