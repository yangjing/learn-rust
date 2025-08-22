use std::ffi::OsStr;

pub fn set_env<K, V>(key: K, value: V)
where
  K: AsRef<OsStr>,
  V: AsRef<OsStr>,
{
  unsafe {
    std::env::set_var(key, value);
  }
}
