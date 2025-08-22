use anyhow::Result;
use config::{ConfigBuilder, FileFormat, builder::DefaultState};
use serde::Deserialize;

use crate::{
  cmd::{DevopsCmd, StorageSource},
  utils::set_env,
};

#[derive(Debug, Deserialize)]
pub struct DevopsConf {
  service: StorageSource,
  storage: Option<StorageConf>,
}
impl DevopsConf {
  pub fn service(&self) -> &StorageSource {
    &self.service
  }

  pub fn storage(&self) -> Option<&StorageConf> {
    self.storage.as_ref()
  }
}

#[derive(Debug, Deserialize)]
pub struct StorageConf {
  pub endpoint: String,
  pub bucket: String,
  pub ak: String,
  pub sk: String,
}

impl DevopsConf {
  pub fn from_devops_cmd(cmd: &DevopsCmd) -> Result<Self> {
    let mut cb =
      config::Config::builder().add_source(config::File::from_str(include_str!("default.toml"), FileFormat::Toml));
    if let Some(config_file) = cmd.config_file.as_deref() {
      cb = cb.add_source(config::File::with_name(config_file));
    }

    if std::env::var("SERVICE").iter().any(|s| s.is_empty()) {
      // 当环境变量 SERVICE 未设置时
      if let Some(value) = cmd.service.as_ref() {
        // 当命令行参数 service 设置时
        set_env("SERVICE", value.to_string());
      }
    }
    let service = match std::env::var("SERVICE") {
      Ok(service) if !service.is_empty() => service,
      _ => cmd.service.clone().unwrap().to_string(),
    };
    set_env("SERVICE", service);
    if let Some(ak) = cmd.ak.as_deref() {
      set_env("STORAGE__AK", ak);
    }
    if let Some(sk) = cmd.sk.as_deref() {
      set_env("STORAGE__SK", sk);
    }
    if let Some(bucket) = cmd.bucket.as_deref() {
      set_env("STORAGE__BUCKET", bucket);
    }
    Self::from_config_builder(cb)
  }

  pub fn from_config_builder(cb: ConfigBuilder<DefaultState>) -> Result<Self> {
    let v = cb.add_source(config::Environment::default().separator("__")).build()?.try_deserialize()?;
    Ok(v)
  }

  pub fn from_file(file: &str) -> Result<Self> {
    let cb = config::Config::builder().add_source(config::File::with_name(file));
    Self::from_config_builder(cb)
  }
}

#[cfg(test)]
mod tests {
  use std::path::Path;

  use crate::cmd::StorageSource;

  use super::*;
  use anyhow::Result;

  #[test]
  fn test_devops_conf() -> Result<()> {
    let file = std::env::var("CARGO_MANIFEST_DIR")? + "/examples/app-template.toml";
    let file = Path::new(&file);
    let config_file = if file.exists() { Some(format!("{}", file.to_string_lossy())) } else { None };

    let cmd = DevopsCmd {
      ak: Some("<ak>".to_string()),
      config_file,
      file_op: Some(crate::cmd::FileOperation::Stat { object_key: "rust/demo/qinling-cli/devops-cli".into() }),
      ..Default::default()
    };

    let conf = DevopsConf::from_devops_cmd(&cmd)?;
    assert_eq!(conf.service(), &StorageSource::Obs);
    assert!(conf.storage().is_some());

    let sc = conf.storage().unwrap();
    assert_eq!(sc.ak, "<ak>");
    assert_eq!(sc.sk, "<ephooKohTh1iechapia0aem0bi2We7eeka9di3>");
    assert_eq!(sc.endpoint, "obs.cn-southwest-2.myhuaweicloud.com");
    assert_eq!(sc.bucket, "<bucket>");

    Ok(())
  }
}
