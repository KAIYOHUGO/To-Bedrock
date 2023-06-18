use anyhow::Result;
use std::{collections::HashMap, num::NonZeroUsize};
use tokio::task;
use tokio_util::io::SyncIoBridge;

use crate::{
    api::VersionPackage,
    parse::{des_en_us_from_java, des_java, TranslateKV},
};

pub async fn fetch(
    java_package: VersionPackage,
    max: NonZeroUsize,
) -> Result<HashMap<String, TranslateKV>> {
    let mut ret: HashMap<String, TranslateKV> = Default::default();
    let mut set = task::JoinSet::new();

    set.spawn(async move {
        let java = java_package.downloads.client.url.get().await?;
        let kv = des_en_us_from_java(java, Some(java_package.downloads.client.size)).await?;
        dbg!("en_us");
        anyhow::Ok(("en_us".into(), kv))
    });

    let assets = java_package.asset_index.url.get().await?;
    let iter = assets
        .objects
        .into_iter()
        .filter(|(path, _)| path.starts_with("minecraft/lang/"))
        .map(|(path, obj)| {
            (
                path.trim_start_matches("minecraft/lang/")
                    .trim_end_matches(".json")
                    .into(),
                obj,
            )
        });

    for (lang_id, obj) in iter {
        set.spawn(async move {
            let reader = obj.url().get().await?;
            let kv = task::spawn_blocking(move || des_java(SyncIoBridge::new(reader))).await??;
            dbg!(&lang_id);
            anyhow::Ok((lang_id, kv))
        });

        if set.len() > max.get() {
            let (k, v) = set.join_next().await.expect("never fail")??;
            ret.insert(k, v);
        }
    }

    while let Some(value) = set.join_next().await {
        let (k, v) = value??;
        ret.insert(k, v);
    }
    Ok(ret)
}
