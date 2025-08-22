use anyhow::{Result, anyhow};
use opendal::{
  Operator,
  services::{Obs, Oss},
};

use crate::{
  cmd::StorageSource,
  conf::{DevopsConf, StorageConf},
};

pub async fn get_operator(cc: &DevopsConf) -> Result<Operator> {
  let sc = cc.storage().ok_or_else(|| anyhow!("The storage config is not found"))?;
  match cc.service() {
    StorageSource::Obs => builder_obs(sc),
    StorageSource::Oss => builder_oss(sc),
  }
}

fn builder_oss(sc: &StorageConf) -> Result<Operator> {
  let mut b = Oss::default();
  b.bucket(&sc.bucket).endpoint(&sc.endpoint).access_key_id(&sc.ak).access_key_secret(&sc.sk);
  let op = Operator::new(b)?;
  Ok(op.finish())
}

fn builder_obs(sc: &StorageConf) -> Result<Operator> {
  let mut b: Obs = Obs::default();
  b.bucket(&sc.bucket).endpoint(&sc.endpoint).access_key_id(&sc.ak).secret_access_key(&sc.sk);
  let op = Operator::new(b)?;
  Ok(op.finish())
}
