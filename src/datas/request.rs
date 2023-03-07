use serde_with::skip_serializing_none;
use std::collections::HashMap;
use std::str;
use std::convert::From;
use serde::{Serialize, Deserialize};
use reqwest::header::{HeaderValue, InvalidHeaderValue};

use super::{AUTH_METHOD, MAX_N};

/**
 * ======
 * BASIC DATA STRUCTS
 * ======
 */

/// Not used in response
#[derive(Deserialize, Serialize, PartialEq, Eq, Debug)]
#[serde(untagged)]
enum StringOrArray<T> {
    Str(T),
    Arr(Vec<T>)
}

impl<T: AsRef<str>> PartialEq<StringOrArray<T>> for &StringOrArray<T> {
    fn eq(&self, other: &StringOrArray<T>) -> bool {
        match (self, other) {
            (StringOrArray::Str(x), StringOrArray::Str(y)) => x.as_ref() == y.as_ref(),
            (StringOrArray::Arr(x), StringOrArray::Arr(y)) => x.iter().zip(y.iter()).all(|(a, b)| a.as_ref() == b.as_ref()),
            _ => false
        }
    }
}

/**
 * ======
 * REQUEST BODY DATA
 * ======
 */

#[derive(Deserialize, Serialize, PartialEq, Eq, Debug)]
pub enum Models {
    /// GPT-3.5-Turbo
    #[serde(rename = "gpt-3.5-turbo")]
    GPT35Turbo,
    /// GPT-3.5-Turbo-0301
    #[serde(rename = "gpt-3.5-turbo-0301")]
    GPT35Turbo0301
}

impl PartialEq<Models> for &Models {
    fn eq(&self, other: &Models) -> bool {
        matches!((self, other), (Models::GPT35Turbo, Models::GPT35Turbo) | (Models::GPT35Turbo0301, Models::GPT35Turbo0301))
    }
}

#[derive(Deserialize, Serialize, PartialEq, Eq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Roles {
    System,
    User,
    Assistant
}

impl PartialEq<Roles> for &Roles {
    fn eq(&self, other: &Roles) -> bool {
        matches!((self, other), (Roles::System, Roles::System) | (Roles::User, Roles::User) | (Roles::Assistant, Roles::Assistant))
    }
}

pub struct ChatLogin<S> {
    /// Bearer <OpenAI_Key> \
    /// Its OpenAI_Key that should be filled in by users \
    /// Just store Str, use HeaderValue to check
    auth: S,
    organization: Option<S>
}

impl<S: AsRef<str>> ChatLogin<S> {
    /// auth must be "Bearer sk-[a-zA-Z0-9]{48}"
    pub fn new(auth: S, organization: Option<S>) -> Result<ChatLogin<S>, String> {
        let collects: Vec<&str> = auth.as_ref().split(' ').collect();
        if collects.len() != 2 { return Err(format!{"Auth format illegal"}); };
        let mut gets: Vec<&str> = collects[1].split('-').collect();
        gets.push(collects[0]);
        if gets[0].ne("sk") || (gets[1].len() != 48 || !gets[1].chars().all(|x| x.is_ascii_alphanumeric())) || gets[2].ne("Bearer") { 
            return Err(format!{"Auth format illegal"}); 
        };
        if let Err(_) = HeaderValue::from_str(auth.as_ref()) {
            return Err(format!("Auth is not legal header value"));
        };
        if let Some(Err(_)) = organization.as_ref().map(|org| HeaderValue::from_str(org.as_ref())) {
            return Err(format!("Invalid Organization format"))
        };
        Ok(ChatLogin::<S>{auth, organization})
    }

    pub fn set_auth(&mut self, auth: S) -> Result<(), String> {
        let mut collects: Vec<&str> = auth.as_ref().split(' ').collect();
        if collects.len() != 2 { return Err(format!{"Auth format illegal"}); };
        let mut gets: Vec<&str> = collects[1].split('-').collect();
        gets.push(collects[0]);
        if gets[0].ne("sk") || (gets[1].len() != 48 || !gets[1].chars().all(|x| x.is_ascii_alphanumeric())) || gets[2].ne("Bearer") { 
            return Err(format!{"Auth format illegal"}); 
        };
        if let Err(_) = HeaderValue::from_str(auth.as_ref()) {
            return Err(format!("Auth is not legal header value"));
        };
        self.auth = auth;
        Ok(())
    }

    pub fn set_organization(&mut self, organization: S) -> Result<(), String> {
        if let Err(_) = HeaderValue::from_str(organization.as_ref()) {
            return Err(format!("Invalid Organization format"))
        };
        self.organization = Some(organization);
        Ok(())
    }

    pub fn get_auth(&self) -> &S {
        &self.auth
    }

    pub fn get_organization(&self) -> Option<&S> {
        self.organization.as_ref()
    }

    // /// check if given token can access to api
    // async fn check_access(&self) -> bool {
    //     let client = Client::new();
    //     if let Ok(response) = client.get("https://api.openai.com/v1/models")
    //         .headers(self.gen_headers())
    //         .send()
    //         .await
    //     {
    //         match response.status() {
    //             reqwest::StatusCode::OK => true,
    //             _other => false
    //         }
    //     } else {
    //         false
    //     }
    // }
}

#[derive(Deserialize, Serialize, PartialEq, Eq, Debug)]
pub struct Message<T> {
    role: Roles,
    content: T
}

impl<T: AsRef<str>> Message<T> {
    pub fn new(role: Roles, content: T) -> Message<T> {
        Message{role, content}
    }

    pub fn set_role(&mut self, role: Roles) {
        self.role = role;
    }

    pub fn set_content(&mut self, content: T) {
        self.content = content;
    }

    pub fn get_role(&self) -> &Roles {
        &self.role
    }

    pub fn get_content(&self) -> &T {
        &self.content
    }
}

/// request body
/// * note: All Introductions are from OpenAI official website, copyright by OpenAI
#[skip_serializing_none]
#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct Body<Sentence> {
    model: Models,
    messages: Vec<Message<Sentence>>,
    /// What sampling temperature to use, between 0 and 2. Higher values like 0.8 will make the output more random, while lower values like 0.2 will make it more focused and deterministic. \
    /// recommend altering this or `top_p` but not both. \
    /// default to 1
    temperature: Option<f32>,
    /// An alternative to sampling with temperature, called nucleus sampling, where the model considers the results of the tokens with top_p probability mass. So 0.1 means only the tokens comprising the top 10% probability mass are considered. (0-1)\
    /// recommend altering this or `temperature` but not both. \
    /// default to 1
    top_p: Option<f32>,
    /// How many chat completion choices to generate for each input message. \
    /// default to 1
    n: Option<u32>,
    /// If set, partial message deltas will be sent, like in ChatGPT. Tokens will be sent as data-only [server-sent](https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events/Using_server-sent_events#Event_stream_format) events as they become available, with the stream terminated by a `data: [DONE]` message. \
    /// default to false
    stream: Option<bool>,
    /// Up to 4 sequences where the API will stop generating further tokens. \
    /// default to NULL
    /*
    What's this?
    #[serde(bound(serialize = "Sentence: AsRef<str>"))]
     */
    stop: Option<StringOrArray<Sentence>>,
    /// The maximum number of tokens allowed for the generated answer. By default, the number of tokens the model can return will be (4096 - prompt tokens). \
    /// default to inf
    max_tokens: Option<u32>,
    /// Number between -2.0 and 2.0. Positive values penalize new tokens based on whether they appear in the text so far, increasing the model's likelihood to talk about new topics. \
    /// [See more information about frequency and presence penalties.](https://platform.openai.com/docs/api-reference/parameter-details) \
    /// default to 0
    presence_penalty: Option<f32>,
    /// Number between -2.0 and 2.0. Positive values penalize new tokens based on their existing frequency in the text so far, decreasing the model's likelihood to repeat the same line verbatim. \
    /// [See more information about frequency and presence penalties.](https://platform.openai.com/docs/api-reference/parameter-details) \
    /// default to 0
    frequency_penalty: Option<f32>,
    /// Modify the likelihood of specified tokens appearing in the completion. \
    /// Accepts a json object that maps tokens (specified by their token ID in the tokenizer) to an associated bias value from -100 to 100. Mathematically, the bias is added to the logits generated by the model prior to sampling. The exact effect will vary per model, but values between -1 and 1 should decrease or increase likelihood of selection; values like -100 or 100 should result in a ban or exclusive selection of the relevant token. \
    /// default to null
    /*
    Need Further Tests:
    The HashMap doesn't guarantee the order, thus may making the unexpected request and cause error.
    If this param doesn't matter with the order, then it's fine.
     */
    logit_bias: Option<HashMap<u32, i32>>,
    /// A unique identifier representing your end-user, which can help OpenAI to monitor and detect abuse. Learn more. \
    /// no default, thus take null as default
    user: Option<Sentence>
}

impl<Sentence: AsRef<str>> Default for Body<Sentence> {
    fn default() -> Self {
        Body { model: Models::GPT35Turbo, messages: Vec::<Message<Sentence>>::new(), temperature: None, top_p: None, n: None, stream: None, stop: None, max_tokens: None, presence_penalty: None, frequency_penalty: None, logit_bias: None, user: None }
    }
}

type E = Result<(), String>;

impl<Sentence: AsRef<str>> Body<Sentence> {
    pub fn new(model: Models) -> Body<Sentence> {
        let default = Body::default();
        Body { model, ..default }
    }

    pub fn set_models(&mut self, model: Models) {
        self.model = model;
    }

    pub fn set_temperature(&mut self, temperature: f32) -> E {
        match (0.0..=2.0).contains(&temperature) {
            true => {
                self.temperature = Some(temperature);
                Ok(())
            },
            false => Err(String::from("temperature must be between 0 and 2"))
        }
    }

    pub fn set_top_p(&mut self, top_p: f32) -> E {
        match (0.0..=1.0).contains(&top_p) {
            true => {
                self.top_p = Some(top_p);
                Ok(())
            },
            false => Err(String::from("top_p must be between 0 and 1"))
        }
    }

    pub fn set_n(&mut self, n: u32) -> E {
        match (1..=MAX_N).contains(&n) {
            true => {
                self.n = Some(n);
                Ok(())
            },
            false => Err(format!("n must be between 0 and {}", MAX_N))
        }
    }

    pub fn set_stream(&mut self, stream: bool) -> E {
        self.stream = Some(stream);
        Ok(())
    }

    pub fn set_stop(&mut self, stop: StringOrArray<Sentence>) -> E {
        if let StringOrArray::Arr(soa) = &stop {
            match soa.len() > 4 {
                true => Err(String::from("stop can't have more than 4 elements")),
                false => {
                    self.stop = Some(stop);
                    Ok(())
                }
            }
        } else {
            self.stop = Some(stop);
            Ok(())
        }
    }

    pub fn set_max_tokens(&mut self, max_tokens: u32) -> E {
        match max_tokens < 1 {
            true => Err(String::from("max_tokens must be greater than 0")),
            false => {
                self.max_tokens = Some(max_tokens);
                Ok(())
            }
        }
    }

    pub fn set_presence_penalty(&mut self, presence_penalty: f32) -> E {
        match (-2.0..=2.0).contains(&presence_penalty) {
            true => {
                self.presence_penalty = Some(presence_penalty);
                Ok(())
            },
            false => Err(String::from("presence_penalty must be between -2 and 2"))
        }
    }

    pub fn set_frequency_penalty(&mut self, frequency_penalty: f32) -> E {
        match (-2.0..=2.0).contains(&frequency_penalty) {
            true => {
                self.frequency_penalty = Some(frequency_penalty);
                Ok(())
            },
            false => Err(String::from("frequency_penalty must be between -2 and 2"))
        }
    }

    pub fn set_logit_bias(&mut self, logit_bias: HashMap<u32, i32>) -> E {
        for v in logit_bias.values() {
            if let false = (-100..=100).contains(v) {
                return Err(String::from("logit_bias right param must be between -100 and 100"));
            };
        }
        self.logit_bias = Some(logit_bias);
        Ok(())
    }

    pub fn add_logit_bias(&mut self, token: u32, bias: i32) -> E {
        if let false = (-100..=100).contains(&bias) {
            return Err(String::from("logit_bias right param must be between -100 and 100"));
        };
        match &mut self.logit_bias {
            Some(lb) => {
                lb.insert(token, bias);
                Ok(())
            },
            None => {
                let mut lb = HashMap::new();
                lb.insert(token, bias);
                self.logit_bias = Some(lb);
                Ok(())
            }
        }
    }

    pub fn add_logit_biass(&mut self, logit_biass: HashMap<u32, i32>) -> E {
        for v in logit_biass.values() {
            if let false = (-100..=100).contains(v) {
                return Err(String::from("logit_bias right param must be between -100 and 100"));
            };
        }
        match &mut self.logit_bias {
            Some(lb) => {
                lb.extend(logit_biass);
                Ok(())
            },
            None => {
                self.logit_bias = Some(logit_biass);
                Ok(())
            }
        }
    }

    pub fn set_user(&mut self, user: Sentence) {
        self.user = Some(user);
    }

    pub fn add_message(&mut self, message: Message<Sentence>) {
        self.messages.push(message);
    }

    pub fn add_messages(&mut self, messages: Vec<Message<Sentence>>) {
        self.messages.extend(messages);
    }

    pub fn clear_messages(&mut self) {
        self.messages.clear();
    }

    pub fn get_messages(&self) -> &Vec<Message<Sentence>> {
        &self.messages
    }

    pub fn get_model(&self) -> &Models {
        &self.model
    }

    pub fn get_temperature(&self) -> Option<f32> {
        self.temperature
    }

    pub fn get_top_p(&self) -> Option<f32> {
        self.top_p
    }

    pub fn get_n(&self) -> Option<u32> {
        self.n
    }

    pub fn get_stream(&self) -> Option<bool> {
        self.stream
    }

    pub fn get_stop(&self) -> Option<&StringOrArray<Sentence>> {
        self.stop.as_ref()
    }

    pub fn get_max_tokens(&self) -> Option<u32> {
        self.max_tokens
    }

    pub fn get_presence_penalty(&self) -> Option<f32> {
        self.presence_penalty
    }

    pub fn get_frequency_penalty(&self) -> Option<f32> {
        self.frequency_penalty
    }

    pub fn get_logit_bias(&self) -> Option<&HashMap<u32, i32>> {
        self.logit_bias.as_ref()
    }

    pub fn get_user(&self) -> Option<&Sentence> {
        self.user.as_ref()
    }
}

#[cfg(test)]
mod request_tests {
    use serde_test::{assert_tokens, Token};

    use super::*;

    #[test]
    fn test_se_de_string_or_array() {
        let a = StringOrArray::Str("test1");
        let b = StringOrArray::Arr(vec!["test2", "test3"]);
        let c = StringOrArray::Str(String::from("test4"));
        let d = StringOrArray::Arr(vec![String::from("test5")]);

        assert_tokens(&a, &[Token::BorrowedStr("test1")]);
        assert_tokens(&b, &[
            Token::Seq { len: Some(2) },
            Token::BorrowedStr("test2"),
            Token::BorrowedStr("test3"),
            Token::SeqEnd
        ]);
        assert_tokens(&c, &[Token::String("test4")]);
        assert_tokens(&d, &[
            Token::Seq { len: Some(1) },
            Token::String("test5"),
            Token::SeqEnd
        ]);
    }

    #[test]
    fn test_se_de_models() {
        let a = Models::GPT35Turbo;
        let b = Models::GPT35Turbo0301;

        assert_tokens(&a, &[
            Token::Enum { name: "Models" },
            Token::Str("gpt-3.5-turbo"),
            Token::Unit
        ]);
        assert_tokens(&b, &[
            Token::Enum { name: "Models" },
            Token::Str("gpt-3.5-turbo-0301"),
            Token::Unit
        ]);
    }

    #[test]
    fn test_se_de_roles() {
        let a = Roles::User;
        let b = Roles::System;
        let c = Roles::Assistant;

        assert_tokens(&a, &[
            Token::Enum { name: "Roles" },
            Token::Str("user"),
            Token::Unit
        ]);
        assert_tokens(&b, &[
            Token::Enum { name: "Roles" },
            Token::Str("system"),
            Token::Unit
        ]);
        assert_tokens(&c, &[
            Token::Enum { name: "Roles" },
            Token::Str("assistant"),
            Token::Unit
        ]);
    }

    #[test]
    fn test_se_de_message() {
        let a = Message::new(Roles::User, "test1");
        let b = Message::new(Roles::Assistant, String::from("test2"));

        assert_tokens(&a, &[
            Token::Struct { name: "Message", len: 2 },
            Token::Str("role"),
            Token::Enum { name: "Roles" },
            Token::Str("user"),
            Token::Unit,
            Token::Str("content"),
            Token::BorrowedStr("test1"),
            Token::StructEnd
        ]);
        assert_tokens(&b, &[
            Token::Struct { name: "Message", len: 2 },
            Token::Str("role"),
            Token::Enum { name: "Roles" },
            Token::Str("assistant"),
            Token::Unit,
            Token::Str("content"),
            Token::BorrowedStr("test2"),
            Token::StructEnd
        ]);
    }

    #[test]
    fn test_se_de_body() {
        let a = Body::<String>::default();

        assert_tokens(&a, &[
            Token::Struct { name: "Body", len: 2 },
            Token::Str("model"),
            Token::Enum { name: "Models" },
            Token::Str("gpt-3.5-turbo"),
            Token::Unit,
            Token::Str("messages"),
            Token::Seq { len: Some(0) },
            Token::SeqEnd,
            Token::StructEnd
        ]);
    }

    #[test]
    fn test_body_data_method() {
        let mut body = Body::default();
        assert!(body.set_temperature(0.1).is_ok());
        assert!(body.set_temperature(3.0).is_err());
        assert!(body.set_top_p(0.3).is_ok());
        assert!(body.set_top_p(3.0).is_err());
        assert!(body.set_n(4).is_ok());
        assert!(body.set_n(MAX_N + 1).is_err());
        assert!(body.set_n(0).is_err());
        assert!(body.set_stream(true).is_ok());
        assert!(body.set_stream(false).is_ok());
        assert!(body.set_stop(StringOrArray::Str("dd")).is_ok());
        assert!(body.set_stop(StringOrArray::Arr(vec!["a", "b", "c", "d"])).is_ok());
        assert!(body.set_max_tokens(100).is_ok());
        assert!(body.set_max_tokens(0).is_err());
        assert!(body.set_presence_penalty(0.1).is_ok());
        assert!(body.set_presence_penalty(3.0).is_err());
        assert!(body.set_presence_penalty(-2.0).is_ok());
        assert!(body.set_presence_penalty(-3.0).is_err());
        assert!(body.set_frequency_penalty(0.1).is_ok());
        assert!(body.set_frequency_penalty(3.0).is_err());
        assert!(body.set_frequency_penalty(-2.0).is_ok());
        assert!(body.set_frequency_penalty(-3.0).is_err());
        assert!(body.add_logit_bias(5044, -33).is_ok());
        assert!(body.add_logit_bias(4033, -193).is_err());
        body.add_message(Message::new(Roles::System, "Earth is be like"));
        body.add_message(Message::new(Roles::User, "What is Earth"));
        let serbody = serde_json::to_string(&body).unwrap();
        assert_eq!(serbody, "{\"model\":\"gpt-3.5-turbo\",\"messages\":[{\"role\":\"system\",\"content\":\"Earth is be like\"},{\"role\":\"user\",\"content\":\"What is Earth\"}],\"temperature\":0.1,\"top_p\":0.3,\"n\":4,\"stream\":false,\"stop\":[\"a\",\"b\",\"c\",\"d\"],\"max_tokens\":100,\"presence_penalty\":-2.0,\"frequency_penalty\":-2.0,\"logit_bias\":{\"5044\":-33}}");
    }

    #[test]
    fn test_chat_login() {
        let mut token = ChatLogin::<&str>::new("Bearer sk-XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX", Some("test")).unwrap();
        assert!(token.set_auth("sk dd ss").is_err());
        assert!(token.set_auth("beard sk-XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX").is_err());
        assert!(token.set_auth("Bearer sk/XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX").is_err());
        assert!(token.set_auth("Bearer sk-XXdsf").is_err());
        assert!(token.set_auth("Bearer sk-XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX;X").is_err());
        assert!(token.set_auth("Bearer sk-XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXaXXXX6XX3XxX").is_ok());
    }
}