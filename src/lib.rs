mod res_format;
use res_format::*;

use serde::ser::SerializeSeq;
use std::collections::HashMap;
use std::str;
use serde::{Serialize, Deserialize};
use reqwest::header::AUTHORIZATION;
use reqwest::{Client, header::{HeaderMap, CONTENT_TYPE, HeaderValue, InvalidHeaderValue}};

enum Apis {
    ListModels(ListModels),
    RetrieveModels(RetrieveModels),
    Completions(Completions),
    Edits(Edits)
}

#[derive(Debug, Deserialize)]
enum StringOrArray {
    Str(String),
    Arr(Vec<String>)
}

impl Serialize for StringOrArray {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            StringOrArray::Str(x) => serializer.serialize_str(x),
            StringOrArray::Arr(x) => {
                let mut seq = serializer.serialize_seq(Some(x.len()))?;
                for element in x.iter() {
                    seq.serialize_element(element)?;
                }
                seq.end()
            }
        }
    }
}

#[derive(Debug)]
struct Token {
    content_type: Option<HeaderValue>,
    // auth required
    auth: HeaderValue,
    organization: Option<HeaderValue>
}

impl Token {
    fn new(auth: &str) -> Result<Token, InvalidHeaderValue> {
        match HeaderValue::from_str(auth) {
            Ok(auth) => Ok(Token {content_type: None, auth, organization: None}),
            Err(invalid) => Err(invalid) 
        }
    }

    fn set_content_type(&mut self, content_type: &str) -> Result<(), InvalidHeaderValue> {
        match HeaderValue::from_str(content_type) {
            Ok(content_type) => {
                self.content_type = Some(content_type);
                Ok(())
            },
            Err(invalid) => Err(invalid)
        }
    }

    fn set_organization(&mut self, organization: String) -> Result<(), InvalidHeaderValue> {
        match HeaderValue::from_str(&organization) {
            Ok(organization) => {
                self.organization = Some(organization);
                Ok(())
            },
            Err(invalid) => Err(invalid)
        }
    }

    fn gen_headers(&self) -> HeaderMap {
        let mut tmp = HeaderMap::new();
        if let Some(x) = &self.content_type {
            tmp.insert(CONTENT_TYPE, x.clone());
        }
        if let Some(y) = &self.organization {
            tmp.insert("OpenAI-Organization", y.clone());
        }
        tmp.insert(AUTHORIZATION, self.auth.clone());
        tmp
    }

    /// check if given token can access to api
    async fn check_access(&self) -> bool {
        let client = Client::new();
        if let Ok(response) = client.get("https://api.openai.com/v1/models")
            .headers(self.gen_headers())
            .send()
            .await
        {
            match response.status() {
                reqwest::StatusCode::OK => true,
                _other => false
            }
        } else {
            false
        }
    }
}

struct ListModels {
    /// required
    auth: Token
}

struct RetrieveModels {
    /// required
    auth: Token,
    /// required
    model: String
}

struct Completions {
    /// required
    auth: Token,
    /// required
    post_data: CompletionsPostData
}

#[derive(Debug, Deserialize, Serialize)]
struct CompletionsPostData {
    /// required
    model: String,
    /// Optional, Server take Defaults as <|endoftext|>
    #[serde(skip_serializing_if="Option::is_none")]
    prompt: Option::<StringOrArray>,
    /// Optional, Server take Defaults as null
    #[serde(skip_serializing_if="Option::is_none")]
    suffix: Option::<String>,
    /// Optional, Server take Defaults as 16 \
    /// newest models support up to 4096 tokens
    #[serde(skip_serializing_if="Option::is_none")]
    max_tokens: Option::<u16>,
    /// Optional, Server take Defaults as 1.0 \
    /// 0-1
    #[serde(skip_serializing_if="Option::is_none")]
    temperature: Option::<f32>,
    /// Optional, Server take Defaults as 1.0 \
    /// 0-1
    #[serde(skip_serializing_if="Option::is_none")]
    top_p: Option::<f32>,
    /// Optional, Server take Defaults as 1 \
    /// how many completions to generate
    #[serde(skip_serializing_if="Option::is_none")]
    n: Option::<u16>,
    /// Optional, Server take Defaults as false
    #[serde(skip_serializing_if="Option::is_none")]
    stream: Option::<bool>,
    /// Optional, Server take Defaults as null
    #[serde(skip_serializing_if="Option::is_none")]
    logprobs: Option::<u16>,
    /// Optional, Server take Defaults as false
    #[serde(skip_serializing_if="Option::is_none")]
    echo: Option::<bool>,
    /// Optional, Server take Defaults as null
    #[serde(skip_serializing_if="Option::is_none")]
    stop: Option::<StringOrArray>,
    /// Optional, Server take Defaults as 0.0 \
    /// -2.0 - 2.0
    #[serde(skip_serializing_if="Option::is_none")]
    presence_penalty: Option::<f32>,
    /// Optional, Server take Defaults as 0.0 \
    /// -2.0 - 2.0
    #[serde(skip_serializing_if="Option::is_none")]
    frequency_penalty: Option::<f32>,
    /// Optional, Server take Defaults as 1 \
    /// Generate <best_of> completions server-side and return the best, 
    /// when used with <n>, <best_of> must be greater than n
    #[serde(skip_serializing_if="Option::is_none")]
    best_of: Option::<u16>,
    /// Optional, Server take Defaults as null \
    /// it may need some tests
    #[serde(skip_serializing_if="Option::is_none")]
    logit_bias: Option::<HashMap<String, u16>>,
    /// Optional
    #[serde(skip_serializing_if="Option::is_none")]
    user: Option::<String>
}

impl CompletionsPostData {
    fn new(model: String) -> CompletionsPostData {
        let default = CompletionsPostData::default();
        CompletionsPostData{ model, ..default }
    }

    /// generate default legal data
    fn default() -> CompletionsPostData {
        CompletionsPostData{ model: "babbage".to_string(), prompt: None, suffix: None, max_tokens: None, temperature: None, top_p: None, n: None, stream: None, logprobs: None, echo: None, stop: None, presence_penalty: None, frequency_penalty: None, best_of: None, logit_bias: None, user: None }
    }

    /// check if post data illegal \
    /// return true if legal \
    /// return false if illegal \
    /// ! the check rule is incomplete
    fn check_data(&self) -> bool {
        // check max_tokens
        (self.max_tokens.is_none() || self.max_tokens.unwrap().le(&4096)) &&
        // check temperature
        (self.temperature.is_none() || (self.temperature.unwrap().ge(&0.0) && self.temperature.unwrap().le(&1.0))) && 
        // check top_p
        (self.top_p.is_none() || (self.top_p.unwrap().ge(&0.0) && self.top_p.unwrap().le(&1.0))) &&
        // check presence_penalty
        (self.presence_penalty.is_none() || (self.presence_penalty.unwrap().ge(&-2.0) && self.presence_penalty.unwrap().le(&2.0))) &&
        // check frequency_penalty
        (self.frequency_penalty.is_none() || (self.frequency_penalty.unwrap().ge(&-2.0) && self.frequency_penalty.unwrap().le(&2.0))) &&
        // check best_of
        (self.best_of.is_none() || self.n.is_none() || self.best_of.unwrap().gt(&self.n.unwrap()))
    }
}

struct Edits {
    auth: Token,
}

impl ListModels {
    /// auth required
    fn new(auth: Token) -> ListModels {
        ListModels {auth}
    }

    /// get and return full json info
    async fn perform(&self) -> Result<ListModelsResponse, String> {
        let headers = self.auth.gen_headers();
        let client = Client::new();
        if let Ok(response) = client.get("https://api.openai.com/v1/models")
            .headers(headers)
            .send()
            .await
        {
            match response.status() {
                reqwest::StatusCode::OK => {
                    match response.json::<ListModelsResponse>().await {
                        Ok(gets) => Ok(gets),
                        Err(x) => Err(x.to_string())
                    }
                },
                reqwest::StatusCode::UNAUTHORIZED => Err("unauthorized".to_string()),
                _other => Err(format!("error code: {}", _other))
            }
        } else {
            Err("request error".to_string())
        }
    }
}

impl RetrieveModels {
    /// auth required
    /// model required
    fn new(auth: Token, model: String) -> RetrieveModels {
        RetrieveModels {auth, model}
    }

    /// get and return full json info
    async fn perform(&self) -> Result<ModelInfo, String> {
        let headers = self.auth.gen_headers();
        let client = Client::new();
        let url = format!("https://api.openai.com/v1/models/{}", self.model);
        if let Ok(response) = client.get(url)
            .headers(headers)
            .send()
            .await
        {
            match response.status() {
                reqwest::StatusCode::OK => {
                    match response.json::<ModelInfo>().await {
                        Ok(gets) => Ok(gets),
                        Err(x) => Err(x.to_string())
                    }
                },
                reqwest::StatusCode::UNAUTHORIZED => Err("unauthorized".to_string()),
                _other => Err(format!("error code: {}", _other))
            }
        } else {
            Err("request error".to_string())
        }
    }
}

impl Completions {
    /// auth required
    /// model required
    fn new(auth: Token, post_data: CompletionsPostData) -> Completions {
        Completions { auth, post_data }
    }

    fn set_model(&mut self, model: String) {
        self.post_data.model = model;
    }

    fn set_prompt(&mut self, prompt: StringOrArray) {
        self.post_data.prompt = Some(prompt);
    }

    fn set_suffix(&mut self, suffix: String) {
        self.post_data.suffix = Some(suffix);
    }

    fn set_max_tokens(&mut self, max_tokens: u16) {
        self.post_data.max_tokens = Some(max_tokens);
    }

    fn set_temperature(&mut self, temperature: f32) {
        self.post_data.temperature = Some(temperature);
    }

    fn set_top_p(&mut self, top_p: f32) {
        self.post_data.top_p = Some(top_p);
    }

    fn set_n(&mut self, n: u16) {
        self.post_data.n = Some(n);
    }

    fn set_stream(&mut self, stream: bool) {
        self.post_data.stream = Some(stream);
    }

    fn set_logprobs(&mut self, logprobs: u16) {
        self.post_data.logprobs = Some(logprobs);
    }

    fn set_echo(&mut self, echo: bool) {
        self.post_data.echo = Some(echo);
    }
    
    fn set_stop(&mut self, stop: StringOrArray) {
        self.post_data.stop = Some(stop);
    }
    
    fn set_presence_penalty(&mut self, presence_penalty: f32) {
        self.post_data.presence_penalty = Some(presence_penalty);
    }

    fn set_frequency_penalty(&mut self, frequency_penalty: f32) {
        self.post_data.frequency_penalty = Some(frequency_penalty);
    }

    fn set_best_of(&mut self, best_of: u16) {
        self.post_data.best_of = Some(best_of);
    }

    fn set_logit_bias(&mut self, logit_bias: HashMap<String, u16>) {
        self.post_data.logit_bias = Some(logit_bias);
    }

    /// post and return full json info
    async fn perform(&self) -> Result<CompletionsResponse, String> {
        if self.post_data.check_data() {
            let headers = self.auth.gen_headers();
            let client = Client::new();
            let url = "https://api.openai.com/v1/completions";
            match client.post(url)
                .headers(headers)
                .json(&self.post_data)
                .send()
                .await
            {
                Ok(response) => {
                    match response.status() {
                        reqwest::StatusCode::OK => {
                            match response.json::<CompletionsResponse>().await {
                                Ok(gets) => Ok(gets),
                                Err(x) => Err(x.to_string())
                            }
                        },
                        reqwest::StatusCode::UNAUTHORIZED => Err("unauthorized".to_string()),
                        _other => Err(format!("error code: {}", _other))
                    }
                },
                Err(x) => Err(x.to_string())
            }
        } else {
            Err("post data illegal".to_string())
        }
    }
}

#[cfg(test)]
mod basic_tests {

    use std::env;

    use crate::{Token, ListModels, RetrieveModels, Completions, CompletionsPostData, StringOrArray};

    macro_rules! aw {
        ($e:expr) => {
            tokio_test::block_on($e)
        };
    }

    #[test]
    fn test_listmodels() {
        let key = "OPENAI_KEY";
        let key_value = match env::var(key) {
            Ok(val) => val,
            Err(e) => panic!("couldn't find env {}: {}", key, e),
        };
        let mut token = Token::new(&key_value).unwrap();
        token.set_content_type("applications/json").unwrap();
        let api = ListModels::new(token);
        match aw!(api.perform()) {
            Ok(models) => {
                println!("{:?}", models);
            },
            Err(e) => {
                println!("{:?}", e);
            }
        }
    }

    #[test]
    fn test_retrievemodels() {
        let key = "OPENAI_KEY";
        let key_value = match env::var(key) {
            Ok(val) => val,
            Err(e) => panic!("couldn't find env {}: {}", key, e),
        };
        let mut token = Token::new(&key_value).unwrap();
        token.set_content_type("applications/json").unwrap();
        let api = RetrieveModels::new(token, "text-davinci-003".to_string());
        match aw!(api.perform()) {
            Ok(models) => {
                println!("{:?}", models);
            },
            Err(e) => {
                println!("{:?}", e);
            }
        }
    }

    #[test]
    fn test_completions() {
        let key = "OPENAI_KEY";
        let key_value = match env::var(key) {
            Ok(val) => val,
            Err(e) => panic!("couldn't find env {}: {}", key, e),
        };
        let token = Token::new(&key_value).unwrap();
        let mut api = Completions::new(token, CompletionsPostData::default());
        api.set_prompt(StringOrArray::Str(String::from("Say this is a test")));
        api.set_max_tokens(7);
        api.set_temperature(0.0);
        println!("{:?}", &api.post_data);
        println!("{}", serde_json::to_string(&api.post_data).unwrap());
        match aw!(api.perform()) {
            Ok(models) => {
                println!("{:?}", models);
            },
            Err(e) => {
                println!("{:?}", e);
            }
        }
    }

    #[test]
    fn test_check_access() {
        let mut false_token = Token::new("Bearer API_KEY").unwrap();
        false_token.set_content_type("applications/json").unwrap();
        assert!(!aw!(false_token.check_access()));
    }
}
