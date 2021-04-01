use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::process::Command;
use std::time::Instant;
use toml::Value;

use clap::clap_app;
use log::*;

mod backend;
use crate::backend::*;

fn main() {
    let matches = clap_app!(ntf =>
     (version: "0.1.0")
     (author: "ccoder64. <ccoder64@gmail.com>")
     (about: "Run program and notify")
     (@setting SubcommandRequired)
     (@arg config: -c --config +takes_value "Sets a custom config file")
     (@arg v: -v +multiple "Sets the level of verbosity")
     (@arg backend: -b --backend +takes_value "Notify backend service")
     (@arg title: -t --title +takes_value "Message title sent")
     (@arg message: -m --message +takes_value "Message body sent")
     (@subcommand test =>
      (@setting TrailingVarArg)
      (about: "Test Configuration")
     )
     (@subcommand send =>
      (@setting TrailingVarArg)
      (about: "Test Send Message")
     )
     (@subcommand done =>
      (@setting TrailingVarArg)
      (about: "Execute the command and notify the message")
      (@arg COMMAND: +required +multiple "")
     )
    )
    .get_matches();

    // init log
    stderrlog::new()
        .module(module_path!())
        .verbosity(matches.occurrences_of("v") as usize + 2)
        .timestamp(stderrlog::Timestamp::Second)
        .init()
        .unwrap();
    debug!("init stderrlog ok");

    // load config
    let config_file = match matches.value_of("config") {
        Some(c) => {
            if !PathBuf::from(c).is_file() {
                error!("config file:{} not exists", c);
                exit(1);
            }
            c.to_string()
        }
        None => match get_default_config_file() {
            Ok(s) => s,
            Err(error) => {
                warn!("find configuration file failed:{}", error);
                exit(2);
            }
        },
    };
    info!("found configuration file:{}", config_file);

    let value = match load_config(&config_file) {
        Ok(v) => v,
        Err(error) => {
            error!("{}", error);
            exit(3);
        }
    };
    debug!("load config ok");

    let backend = match matches.value_of("backend") {
        Some(b) => b,
        None => match value["global"].get("backend") {
            Some(b) => match b.as_str() {
                Some(s) => s,
                None => {
                    error!("config {} backend field type error", config_file);
                    exit(4);
                }
            },
            None => {
                error!("backend not set");
                exit(5);
            }
        },
    };
    info!("use backend:{}", backend);

    let backend_type = value[backend]["type"].as_str();
    let backend = match Backend::new(backend_type, &value[backend]) {
        Ok(b) => b,
        Err(error) => {
            error!("create backend:{:?} failed:{}", backend_type, error);
            exit(6);
        }
    };

    let title = matches.value_of("title").unwrap_or("");
    let message = matches.value_of("message").unwrap_or("");

    debug!("configuration title:{} message:{}", title, message);

    if let Some(ref _sub_matches) = matches.subcommand_matches("test") {
        debug!("test configuration ...");
        info!("configuration check ok");
    }
    else if let Some(ref _sub_matches) = matches.subcommand_matches("send") {
        debug!("send message ...");
        send(backend, title, message);
    }
    else if let Some(ref sub_matches) = matches.subcommand_matches("done") {
        debug!("run command ...");
        let command: Vec<_> = sub_matches.values_of("COMMAND").unwrap().collect();
        done(command, backend, title, message);
    }
}

fn get_default_config_file() -> Result<String, String> {
    if let Ok(s) = env::var("HOME") {
        let pathbuf = Path::new(&s).join(".ntf.toml");
        if pathbuf.is_file() {
            return Ok(String::from(pathbuf.to_str().unwrap()));
        }
    }
    if PathBuf::from("/etc/ntf.toml").is_file() {
        return Ok(String::from("/etc/ntf.toml"));
    }
    Err(String::from("not found"))
}

fn load_config(config_file: &str) -> Result<Value, String> {
    String::from_utf8(
        fs::read(&config_file).map_err(|e| format!("read file:{} failed:{}", config_file, e))?,
    )
    .map_err(|e| format!("decode utf8 error:{}", e))?
    .parse::<Value>()
    .map_err(|e| format!("parse config:{} failed:{}", config_file, e))
}

fn send(backend: Box<dyn Backend>, title: &str, message: &str) {
    let send_result;
    if !title.is_empty() && !message.is_empty() {
        send_result = backend.send_message(title, message);
    } else {
        send_result = backend.send_message("hello", "this is test message");
    }
    match send_result {
        Ok(s) => info!("send message ok:{}", s),
        Err(e) => error!("send failed:{}", e),
    }
}

fn done(command: Vec<&str>, backend: Box<dyn Backend>, title: &str, message: &str) {
    info!("start command:{:?}", command);
    let result = run_command(&command[0], &command[1..]);
    let send_result;
    if !title.is_empty() && !message.is_empty() {
        send_result = backend.send_message(title, message);
    } else {
        match result {
            Ok(message) => {
                info!("execute ok:{}", message);
                let title = format!("命令 `{}` 执行完成", command.join(" "));
                send_result = backend.send_message(title.as_str(), message.as_str());
            }
            Err(message) => {
                warn!("execute failed:{}", message);
                let title = format!("命令 `{}` 执行失败", command.join(" "));
                send_result = backend.send_message(title.as_str(), message.as_str());
            }
        }
    }
    match send_result {
        Ok(s) => info!("send message ok:{}", s),
        Err(e) => error!("send failed:{}", e),
    }
}

fn run_command(cmd: &str, args: &[&str]) -> Result<String, String> {
    let start = Instant::now();
    let status = match Command::new(cmd).args(args).status() {
        Ok(s) => s,
        Err(error) => {
            return Ok(format!(
                "执行用时:{}秒,失败原因:{}",
                start.elapsed().as_secs(),
                error
            ));
        }
    };

    match status.code() {
        Some(0) => {
            return Ok(format!("执行用时:{}秒", start.elapsed().as_secs()));
        }
        Some(code) => {
            return Err(format!(
                "执行用时:{}秒, 退出码:{}",
                start.elapsed().as_secs(),
                code
            ));
        }
        None => {
            return Err(format!(
                "执行用时:{}秒, 退出原因:被杀死",
                start.elapsed().as_secs()
            ))
        }
    }
}
