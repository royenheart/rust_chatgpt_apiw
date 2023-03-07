//! # TODO
//! 1. ChatGPT api wrapper(chat model, use gpt-3.5-turbo)
//! 2. A CLI interface, which can ask some questions and get answers ONE BY ONE
//! 3. Store questions in a file with special formats and be able to use these questions to call the API
//! 4. Using methods 3, can insert an answer into a specified location in an article with a specific format to produce a complete article
//!     It should seem like this:
//!     - Article
//! 
//!     ```
//!     As we all know, Earth is a {{earth-be-like}}
//!     ```
//! 
//!     - Answers
//! 
//!     ```
//!     earth-be-like = "Answers from questions: 'What is earth'"
//!     ```
//! 5. log system(record raw questions, curl format api call, raw response and so on)
//! 
//! # Constructions
//! - CLI(main.rs)
//! - API Wrapper(lib.rs)
//! - Network Requests, request data should be related to a response data(using trait and type) - netreq
//! - Data formats(Display trait(display), Default trait, option trait(Just use Option), required trait(Not Option), support correct serialize and deserialize methods(Generate right output for request body and read data), API Callers can just use create and edit(**use provided methods**) funcs without worring about incorrect attributes in request(limited and auto check). Users just ask questions and get answers. - formats

mod datas;
mod netreq;



#[cfg(test)]
mod basic_tests {
    // use std::env;

    // use crate::{Token, Completions, CompletionsPostData, StringOrArray};

    // macro_rules! aw {
    //     ($e:expr) => {
    //         tokio_test::block_on($e)
    //     };
    // }

    // #[test]
    // fn test_completions() {
    //     let key = "OPENAI_KEY";
    //     let key_value = match env::var(key) {
    //         Ok(val) => val,
    //         Err(e) => panic!("couldn't find env {}: {}", key, e),
    //     };
    //     let token = Token::new(&key_value).unwrap();
    //     let mut api = Completions::new(token, CompletionsPostData::default());
    //     api.set_prompt(StringOrArray::Str(String::from("Say this is a test")));
    //     api.set_max_tokens(7);
    //     api.set_temperature(0.0);
    //     println!("{:?}", &api.post_data);
    //     println!("{}", serde_json::to_string(&api.post_data).unwrap());
    //     match aw!(api.perform()) {
    //         Ok(models) => {
    //             println!("{:?}", models);
    //         },
    //         Err(e) => {
    //             println!("{:?}", e);
    //         }
    //     }
    // }

    // #[test]
    // fn test_check_access() {
    //     let mut false_token = Token::new("Bearer API_KEY").unwrap();
    //     false_token.set_content_type("applications/json").unwrap();
    //     assert!(!aw!(false_token.check_access()));
    // }
}
