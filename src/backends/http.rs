use super::Backend;
use log::debug;
use reqwest::{blocking, Method};
use serde::Deserialize;
use std::collections::HashMap;
use std::convert::TryInto;

#[derive(Debug, Deserialize)]
pub struct Http {
    url: String,
    method: Option<String>,
    headers: Option<HashMap<String, String>>,
    form: Option<HashMap<String, String>>,
    json: Option<HashMap<String, String>>,
    body: Option<String>,
}

impl Backend for Http {
    fn send_message(&self, title: &str, message: &str) -> Result<String, String> {
        let client = blocking::Client::new();
        let url = self
            .url
            .replace("{title}", title)
            .replace("{message}", message);
        debug!("http url:{}", url);

        let headers: reqwest::header::HeaderMap = self
            .headers
            .as_ref()
            .unwrap_or(&HashMap::new())
            .try_into()
            .map_err(|e| format!("invalid headers:{:?}", e))?;
        debug!("http headers:{:?}", headers);

        let response = match Method::from_bytes(
            self.method
                .as_ref()
                .unwrap_or(&String::from("GET"))
                .as_bytes(),
        )
        .ok()
        {
            Some(Method::GET) => client.get(url.as_str()).headers(headers).send(),
            Some(Method::POST) => {
                let mut request_builder = client.post(url.as_str()).headers(headers);
                if let Some(form) = self.form.as_ref() {
                    let mut new_form = HashMap::new();
                    for (key, val) in form.iter() {
                        new_form.insert(
                            key,
                            val.replace("{title}", title).replace("{message}", message),
                        );
                    }
                    debug!("send form data:{:?}", new_form);
                    request_builder = request_builder.form(&new_form);
                } else if let Some(json) = self.json.as_ref() {
                    let mut new_json = HashMap::new();
                    for (key, val) in json.iter() {
                        new_json.insert(
                            key,
                            val.replace("{title}", title).replace("{message}", message),
                        );
                    }
                    debug!("send json data:{:?}", new_json);
                    request_builder = request_builder.json(&new_json);
                } else if let Some(body) = self.body.as_ref() {
                    let new_body = body.replace("{title}", title).replace("{message}", message);
                    debug!("send body data:{:?}", new_body);
                    request_builder = request_builder.body(new_body);
                } else {
                    return Err(String::from(
                        "post method must have form | json | body fields",
                    ));
                }
                request_builder.send()
            }
            _ => {
                return Err(format!(
                    "method {} not support",
                    self.method.as_ref().unwrap()
                ))
            }
        };

        response
            .map_err(|e| format!("request:{} failed:{}", url, e))?
            .text()
            .map_err(|e| format!("read response failed:{}", e))
    }
}
