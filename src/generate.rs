use std::{collections::HashMap, io::Write, path::PathBuf, sync::Arc};

use crate::{
    pack::{pack_addon, Header, Manifest, Module},
    parse::{ser_bedrock, TranslateKV},
};
use anyhow::{anyhow, Result};
use tokio::{fs, task};
use tokio_util::io::SyncIoBridge;

/// output is `Map<bedrock_id,java_id>`
pub fn gen_bedrock_java_id_map(mut bedrock: TranslateKV, java: TranslateKV) -> TranslateKV {
    let mut java_value_key = TranslateKV::new();
    java_value_key.reserve(java.capacity());
    for (k, v) in java {
        java_value_key.insert(v, k);
    }

    for (_, v) in bedrock.iter_mut() {
        if let Some(java_key) = java_value_key.remove(v) {
            *v = java_key;
        }
    }
    bedrock
}

pub fn gen_translate(
    bedrock_java_id_map: &TranslateKV,
    mut java: TranslateKV,
    bedrock: TranslateKV,
) -> TranslateKV {
    let mut translate = TranslateKV::new();
    for (bedrock_id, bedrock_text) in bedrock {
        let Some(java_id) = bedrock_java_id_map.get(&bedrock_id) else {
            continue;
        };
        let Some(java_text) = java.remove(java_id) else {
            continue;
        };
        if bedrock_text == java_text {
            continue;
        }
        translate.insert(bedrock_id, java_text);
    }
    translate
}

pub async fn gen_output(
    mut java_texts: HashMap<String, TranslateKV>,
    mut bedrock_texts: HashMap<String, TranslateKV>,
    version: [u8; 3],
    output: PathBuf,
) -> Result<()> {
    let (Some(java_en_us),Some(bedrock_en_us))=(java_texts.remove("en_us"),bedrock_texts.remove("en_us")) else {
        return Err(anyhow!("en_us.json and en_us.lang file are required"));
    };

    let mut tasks = task::JoinSet::new();
    let id_map = Arc::new(gen_bedrock_java_id_map(bedrock_en_us, java_en_us));

    let manifest = Arc::new(Manifest {
        format_version: 2,
        header: Header {
            description: "Java Translation Pack Generate by kaiyo hugo".into(),
            name: "To Bedrock Translate Resource Pack".into(),
            uuid: "66c6e9a8-3093-462a-9c36-dbb052165623".into(),
            version,
            min_engine_version: [1, 16, 0],
        },
        modules: vec![Module {
            module_type: "resources".into(),
            uuid: "743f6949-53be-44b6-b326-398005028623".into(),
            version,
        }],
    });

    for (id, java_text) in java_texts {
        let Some(bedrock_text) = bedrock_texts.remove(&id) else {
            continue;
        };
        let lang_id = id
            .split_once('_')
            .map(|(begin, end)| format!("{}_{}", begin, end.to_uppercase()))
            .unwrap_or(id);
        let mut path = output.join(format!("To_Bedrock_{}", lang_id));
        path.set_extension("mcpack");

        let id_map = id_map.clone();
        let manifest = manifest.clone();

        tasks.spawn(async move {
            let text = gen_translate(&id_map, java_text, bedrock_text);
            task::spawn_blocking(|| pack_addon(path, lang_id, text, manifest)).await??;
            anyhow::Ok(())
        });
    }
    while let Some(next) = tasks.join_next().await {
        next??;
    }
    Ok(())
}
