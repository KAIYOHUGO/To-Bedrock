use anyhow::Result;
use serde::{de::DeserializeOwned, Deserialize};
use std::{
    collections::HashMap,
    io::{Error, ErrorKind},
    marker::PhantomData,
};
use tokio::io::AsyncBufRead;
use tokio_stream::StreamExt;
use tokio_util::io::StreamReader;

pub async fn get_version_manifest() -> Result<VersionManifest> {
    let resp =
        reqwest::get("https://piston-meta.mojang.com/mc/game/version_manifest_v2.json").await?;
    Ok(resp.json().await?)
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct VersionManifest {
    pub latest: Latest,
    pub versions: Vec<Version>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Latest {
    pub release: Id,
    pub snapshot: Id,
}

/// version id
///
/// `1.19.4` `23w12a` `1.19.4-pre4`
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Id(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Version {
    pub id: Id,
    #[serde(rename = "type")]
    pub version_type: Type,
    pub url: JsonUrl<VersionPackage>,
    // pub time: String,
    // #[serde(rename = "releaseTime")]
    // pub release_time: SystemTime,
    // pub sha1: String,
    // #[serde(rename = "complianceLevel")]
    // pub compliance_level: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(transparent)]
pub struct JsonUrl<T>(String, PhantomData<T>);

impl<T> JsonUrl<T>
where
    T: DeserializeOwned,
{
    pub async fn get(self) -> Result<T> {
        let resp = reqwest::get(self.0).await?;
        Ok(resp.json().await?)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub enum Type {
    #[serde(rename = "old_alpha")]
    OldAlpha,
    #[serde(rename = "old_beta")]
    OldBeta,
    #[serde(rename = "release")]
    Release,
    #[serde(rename = "snapshot")]
    Snapshot,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct VersionPackage {
    // pub arguments: Arguments,
    #[serde(rename = "assetIndex")]
    pub asset_index: AssetIndex,
    // pub assets: String,
    // #[serde(rename = "complianceLevel")]
    // pub compliance_level: i64,
    pub downloads: Downloads,
    pub id: Id,
    // #[serde(rename = "javaVersion")]
    // pub java_version: JavaVersion,
    // pub libraries: Vec<Library>,
    // pub logging: Logging,
    // #[serde(rename = "mainClass")]
    // pub main_class: String,
    // #[serde(rename = "minimumLauncherVersion")]
    // pub minimum_launcher_version: i64,
    // #[serde(rename = "releaseTime")]
    // pub release_time: SystemTime,
    // pub time: String,
    #[serde(rename = "type")]
    pub version_type: Type,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct AssetIndex {
    // pub id: String,
    // pub sha1: String,
    pub size: usize,
    // #[serde(rename = "totalSize")]
    // pub total_size: Option<usize>,
    pub url: JsonUrl<PackageAsset>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Downloads {
    pub client: ClientMappingsClass,
    pub client_mappings: ClientMappingsClass,
    pub server: ClientMappingsClass,
    pub server_mappings: ClientMappingsClass,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ClientMappingsClass {
    // pub sha1: String,
    pub size: usize,
    pub url: RawUrl,
    // pub path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(transparent)]
pub struct RawUrl(String);

impl RawUrl {
    pub async fn get(self) -> Result<impl AsyncBufRead> {
        get(self.0).await
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct PackageAsset {
    pub objects: HashMap<String, Object>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Object {
    pub hash: String,
    pub size: usize,
}

impl Object {
    pub fn url(&self) -> RawUrl {
        RawUrl(format!(
            "https://resources.download.minecraft.net/{}/{}",
            &self.hash[..2],
            &self.hash
        ))
    }
}

pub async fn get(url: String) -> Result<impl AsyncBufRead> {
    let resp = reqwest::get(url).await?;
    let reader = StreamReader::new(
        resp.bytes_stream()
            .map(|r| r.map_err(|e| Error::new(ErrorKind::Other, e))),
    );
    Ok(reader)
}
