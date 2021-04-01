use super::Backend;
use log::debug;
use reqwest::blocking;
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct WorkWeixin {
    corpid: String,
    agentid: String,
    corpsecret: String,
}

#[derive(Debug, Deserialize)]
struct GetTokenResponse {
    errcode: i32,
    errmsg: String,
    access_token: Option<String>,
    expires_in: Option<i32>,
}

impl Backend for WorkWeixin {
    fn send_message(&self, title: &str, message: &str) -> Result<String, String> {
        let access_token = match self.get_token() {
            Ok(json) => {
                if json.errcode != 0 {
                    return Err(format!("get_token failed:{}", json.errmsg));
                }
                json.access_token.unwrap()
            }
            Err(error) => {
                return Err(format!("get_token failed:{:?}", error));
            }
        };
        debug!("work_weixin access_token:{}", access_token);
        let content = format!("{}\n{}", title, message);
        debug!("work_weixin content:{}", content);
        match self.send_weixin(access_token, content) {
            Ok(s) => {
                return Ok(s);
            }
            Err(error) => {
                return Err(format!("send message:{:?}", error));
            }
        }
    }
}

impl WorkWeixin {
    fn get_token(&self) -> Result<GetTokenResponse, reqwest::Error> {
        let secret_url = format!(
            "https://qyapi.weixin.qq.com/cgi-bin/gettoken?corpid={}&corpsecret={}",
            self.corpid, self.corpsecret
        );
        Ok(blocking::get(secret_url)?.json::<GetTokenResponse>()?)
    }

    fn send_weixin(&self, access_token: String, content: String) -> Result<String, reqwest::Error> {
        let send_url = format!(
            "https://qyapi.weixin.qq.com/cgi-bin/message/send?access_token={}",
            access_token
        );
        let json_str = r#"
        {
            "touser": "@all",
            "msgtype": "text",
            "agentid": "",
            "text": {"content": ""}
        }"#;
        let mut json: Value = serde_json::from_str(json_str).unwrap();
        json["agentid"] = Value::from(self.agentid.as_str());
        json["text"]["content"] = Value::from(content.as_str());
        let client = blocking::Client::new();
        Ok(client.post(send_url).json(&json).send()?.text()?)
    }
}
