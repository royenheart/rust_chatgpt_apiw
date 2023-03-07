use async_trait::async_trait;
use reqwest::Client;
use reqwest::header::AUTHORIZATION;
use reqwest::header::CONTENT_TYPE;
use reqwest::header::HeaderMap;
use reqwest::header::HeaderValue;
use serde::Deserialize;
use serde::Serialize;

use self::perform::AsyncPerform;
use self::perform::GenHeaders;

mod perform;

use crate::datas;
use crate::datas::AUTH_CONTENT_TYPE;
use crate::datas::AUTH_ORG;
use crate::datas::POST_URL;
use crate::datas::response::Resp;
use crate::datas::request::Message;
use crate::datas::request::Models;
use crate::datas::request::Roles;
use crate::datas::request::ChatLogin;
use crate::datas::request::Body;

impl<S: AsRef<str>> GenHeaders for ChatLogin<S> {
    fn gen_headers(&self) -> HeaderMap {
        let mut tmp: HeaderMap = HeaderMap::with_capacity(3);
        tmp.insert(CONTENT_TYPE, HeaderValue::from_str(AUTH_CONTENT_TYPE).unwrap());
        tmp.insert(AUTHORIZATION, HeaderValue::from_str(self.get_auth().as_ref()).unwrap());
        if let Some(org) = self.get_organization() {
            tmp.insert(AUTH_ORG, HeaderValue::from_str(org.as_ref()).unwrap());
        }
        tmp
    }
}

/*
Why? It may need more tests...
 */
#[async_trait]
impl<Auth: GenHeaders + std::marker::Sync> AsyncPerform<Auth> for Body<String> {
    type Respr = Resp<String>;
    async fn perform(&self, auth: &Auth) -> Result<Self::Respr, String> {
        let headers = auth.gen_headers();
        let client = Client::new();
        match client.post(POST_URL)
            .headers(headers)
            .json(&self)
            .send()
            .await
        {
            Ok(response) => {
                match response.status() {
                    reqwest::StatusCode::OK => {
                        match response.json::<Self::Respr>().await {
                            Ok(gets) => Ok(gets),
                            Err(x) => Err(format!{"Resp Parse Error: {}", x})
                        }
                    },
                    reqwest::StatusCode::UNAUTHORIZED => Err(format!("unauthorized")),
                    _other => Err(format!("error code: {}", _other))
                }
            },
            Err(x) => Err(format!{"Server not response: {}", x})
        }
    }
}

#[cfg(test)]
mod netreq_tests {
    use std::{env, fmt::format};

    use super::*;

    macro_rules! aw {
        ($e:expr) => {
            tokio_test::block_on($e)
        };
    }

    #[test]
    fn test_ask() {
        let key = "OPENAI_KEY";
        let key_value = match env::var(key) {
            Ok(val) => val,
            Err(e) => panic!("couldn't find env {}: {}", key, e),
        };
        let token = ChatLogin::new(&key_value, None).unwrap();
        let mut chat = Body::<String>::default();
        chat.add_message(Message::new(Roles::User, format!("Today is?")));
        match aw!(chat.perform(&token)) {
            Ok(models) => {
                println!("{:?}", models);
            },
            Err(e) => {
                println!("{:?}", e);
            }
        }
    }
}