use serde::Deserialize;
use toml::Value;

pub mod http;
use super::http::Http;
pub mod shell;
use super::shell::Shell;
pub mod work_weixin;
use super::work_weixin::WorkWeixin;

pub trait Backend {
    fn send_message(&self, title: &str, message: &str) -> Result<String, String>;
}

impl dyn Backend {
    pub fn new(backend_type: Option<&str>, value: &Value) -> Result<Box<dyn Backend>, String> {
        match backend_type {
            Some("http") => Ok(Box::new(Backend::try_from::<Http>(value)?)),
            Some("shell") => Ok(Box::new(Backend::try_from::<Shell>(value)?)),
            Some("work_weixin") => Ok(Box::new(Backend::try_from::<WorkWeixin>(value)?)),
            _ => Err(format!("no such backend:{:?}", backend_type)),
        }
    }
}

impl<'de> dyn Backend {
    fn try_from<T: Deserialize<'de>>(value: &Value) -> Result<T, String> {
        value
            .clone()
            .try_into::<T>()
            .map_err(|e| format!("deserialize failed:{:?}", e))
    }
}
