use std::fmt::Display;

use clap::{Parser, ValueEnum};
use serde::Deserialize;

use super::FileOperation;

#[derive(Debug, Default, Parser)]
#[command(name = "devops-cli")]
#[command(version, about = "DevOps command tool")]
pub struct DevopsCmd {
  #[arg(short, long, help = "The storage service to use, default is 'obs'")]
  pub service: Option<StorageSource>,

  #[arg(short, long)]
  pub bucket: Option<String>,

  #[arg(long)]
  pub ak: Option<String>,

  #[arg(long)]
  pub sk: Option<String>,

  #[arg(short('f'), long)]
  pub config_file: Option<String>,

  #[command(subcommand)]
  pub file_op: Option<FileOperation>,
}

#[derive(Debug, Clone, PartialEq, Eq, ValueEnum, Deserialize)]
pub enum StorageSource {
  /// 华为云 OBS
  Obs,
  /// 阿里云 OSS
  Oss,
}

impl Display for StorageSource {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      StorageSource::Obs => f.write_str("obs"),
      StorageSource::Oss => f.write_str("oss"),
    }
  }
}
