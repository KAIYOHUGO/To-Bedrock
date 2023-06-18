use anyhow::{anyhow, Result};
use std::{
    collections::HashMap,
    io::{Cursor, Read, Write},
};
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncReadExt};
use zip::ZipArchive;

pub type TranslateKV = HashMap<String, String>;

// TODO: add support for namespace
// pub type TranslateKV = HashMap<Namespace, String>;
// #[derive(Debug, Clone, Hash)]
// pub struct Namespace(Vec<String>);

pub fn des_java(reader: impl Read) -> Result<TranslateKV> {
    Ok(serde_json::from_reader(reader)?)
}

pub async fn des_bedrock(reader: impl AsyncBufReadExt + Unpin) -> Result<TranslateKV> {
    let mut kv = TranslateKV::new();
    let mut lines = reader.lines();

    while let Some(line) = lines.next_line().await? {
        let text = if let Some((text, _comment)) = line.split_once('#') {
            text
        } else {
            &line
        };
        // ignore BOM
        let text = text
            .trim_start_matches('\u{feff}')
            .trim_start()
            .trim_end_matches('\t');
        if text.is_empty() {
            continue;
        }

        let Some((k,v)) = text.split_once('=') else {
            dbg!(line);
            return Err(anyhow!("malformed bedrock lang file, expect 'key=value'"))
        };
        kv.insert(k.to_owned(), v.to_owned());
    }
    Ok(kv)
}

pub fn sync_ser_bedrock(writer: &mut impl Write, kv: TranslateKV) -> Result<()> {
    for (k, v) in kv {
        writeln!(writer, "{k}={v}")?;
    }
    Ok(())
}

pub async fn des_en_us_from_java(
    mut reader: impl AsyncBufRead + Unpin,
    size: Option<usize>,
) -> Result<TranslateKV> {
    let mut tmp_jar = if let Some(size) = size {
        Vec::with_capacity(size)
    } else {
        vec![]
    };
    reader.read_to_end(&mut tmp_jar).await?;

    let mut jar = ZipArchive::new(Cursor::new(tmp_jar))?;
    let en_us = jar.by_name("assets/minecraft/lang/en_us.json")?;
    let kv = des_java(en_us)?;
    Ok(kv)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_des_bedrock() {
        let bytes = "\u{feff}
aaa=bbb\t\t## ignore me
## line comment

##
ccc=ddd\t#";
        let result = HashMap::from_iter([
            ("aaa".to_owned(), "bbb".to_owned()),
            ("ccc".to_owned(), "ddd".to_owned()),
        ]);
        let kv = des_bedrock(bytes.as_bytes()).await.unwrap();
        assert_eq!(kv, result)
    }
}
