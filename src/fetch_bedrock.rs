use std::{collections::HashMap, num::NonZeroUsize};

use anyhow::Result;
use tokio::task;

use crate::{
    api::get,
    parse::{des_bedrock, TranslateKV},
};

pub async fn fetch(
    bedrock_version: String,
    max: NonZeroUsize,
) -> Result<HashMap<String, TranslateKV>> {
    let mut ret: HashMap<String, TranslateKV> = Default::default();
    let mut set = task::JoinSet::new();
    let octocrab = octocrab::instance();
    let page = octocrab
        .repos("Mojang", "bedrock-samples")
        .get_content()
        .path("resource_pack/texts")
        .r#ref(format!("v{}", bedrock_version))
        .send()
        .await?;
    for i in page.items {
        let Some((lang_id,ext))=i.name.split_once('.') else {
                continue;
            };
        if ext != "lang" {
            continue;
        }
        let Some(url) = i.download_url else {
                continue;
            };
        let lang_id = lang_id.to_lowercase();
        set.spawn(async move {
            let content = get(url).await?;
            let kv = des_bedrock(content).await?;
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
