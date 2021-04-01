use super::Backend;
use log::debug;
use serde::Deserialize;
use std::process::Command;

#[derive(Debug, Deserialize)]
pub struct Shell {
    command: String,
}

impl Backend for Shell {
    fn send_message(&self, title: &str, message: &str) -> Result<String, String> {
        debug!("execute command sh -c {}", self.command);
        Ok(Command::new("sh")
            .arg("-c")
            .arg(self.command.as_str())
            .env("title", title)
            .env("message", message)
            .status()
            .map_err(|e| format!("run shell cmd:{} fail:{}", self.command, e))?
            .code()
            .map_or("None".to_string(), |v| v.to_string()))
    }
}
