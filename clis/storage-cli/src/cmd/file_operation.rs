use anyhow::Result;
use clap::Subcommand;
use futures::TryStreamExt;
use log::info;
use opendal::Operator;
use tokio::{fs::File, io::AsyncReadExt};

#[derive(Debug, Subcommand)]
pub enum FileOperation {
  Put { src: String, object_key: String },
  Get { object_key: String, dst: String },
  Stat { object_key: String },
}

impl FileOperation {
  pub async fn execute(&self, op: &Operator) -> Result<()> {
    match self {
      FileOperation::Put { src, object_key } => put_src_to_object_key(op, src, object_key).await?,
      FileOperation::Get { object_key, dst } => get_object_key_to_dst(op, object_key, dst).await?,
      FileOperation::Stat { object_key } => dump_stat(op, object_key).await?,
    }
    Ok(())
  }
}

/// 上传本地文件到对象存储
async fn put_src_to_object_key(op: &Operator, src: &str, object_key: &str) -> Result<()> {
  use futures::AsyncWriteExt;

  let mut f = File::open(src).await?;
  let mut writer = op.writer_with(object_key).await?.into_futures_async_write();
  let mut buf = [0_u8; 8192];
  let mut uploaded = 0;

  loop {
    let n = f.read(&mut buf[..]).await?;
    if n == 0 {
      break;
    }
    writer.write_all(&buf[..n]).await?;
    uploaded += n;
  }
  writer.close().await?;

  info!("Total file upload of {} bytes.", uploaded);
  Ok(())
}

/// 下载对象存储文件到本地
async fn get_object_key_to_dst(op: &Operator, object_key: &str, dst: &str) -> Result<()> {
  use tokio::io::AsyncWriteExt;

  let mut f = File::create_new(dst).await?;
  let reader = op.reader_with(object_key).await?;
  let mut readed = 0u64;

  let mut bs = reader.into_bytes_stream(..).await?;
  while let Ok(Some(item)) = bs.try_next().await {
    if item.is_empty() {
      break;
    }
    readed += item.len() as u64;
    f.write_all(&item).await?;
  }

  info!("Total file download of {} bytes.", readed);
  f.flush().await?;
  Ok(())
}

/// 输出对象存储文件元信息
async fn dump_stat(op: &Operator, object_key: &str) -> Result<()> {
  let md = op.stat(object_key).await?;
  println!(
    r#"metakey: {:?}
mode: {}
cache_control: {}
content_disposition: {}
content_length: {}
content_md5: {}
content_range: {}
content_type: {}
etag: {}
last_modified: {}
version: {}"#,
    md.metakey().into_iter().map(|k| format!("{:?}", k)).collect::<Vec<_>>(),
    md.mode(),
    md.cache_control().unwrap_or_default(),
    md.content_disposition().unwrap_or_default(),
    md.content_length(),
    md.content_md5().unwrap_or_default(),
    md.content_range().as_ref().map(|v| v.to_string()).unwrap_or_default(),
    md.content_type().unwrap_or_default(),
    md.etag().unwrap_or_default(),
    md.last_modified().as_ref().map(|d| d.to_rfc3339()).unwrap_or_default(),
    md.version().unwrap_or_default(),
  );
  Ok(())
}
