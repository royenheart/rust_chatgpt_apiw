use serde::Deserialize;
use crate::datas::request::{Message};

#[derive(Deserialize, PartialEq, Eq, Debug)]
pub struct Resp<Sentence> {
    id: Sentence,
    object: Sentence,
    created: u64,
    choices: Vec<Choice<Sentence>>,
    usage: Usage
}

#[derive(Deserialize, PartialEq, Eq, Debug)]
struct Usage {
    prompt_tokens: u16,
    completion_tokens: u16,
    total_tokens: u16
}

#[derive(Deserialize, PartialEq, Eq, Debug)]
struct Choice<Sentence> {
    index: u64,
    message: Message<Sentence>,
    finish_reason: Sentence
}

#[cfg(test)]
mod response_test {
    use serde_test::{assert_de_tokens, Token};

    use crate::datas::request::Roles;

    use super::*;

    #[test]
    fn test_de_choice() {
        let choice = Choice::<String> {
            index: 0,
            message: Message::new(Roles::Assistant, "\nTest".to_string()),
            finish_reason: "stop".to_string()
        };

        assert_de_tokens(&choice, &[
            Token::Struct { name: "Choice", len: 3 },
            Token::Str("index"),
            Token::U64(0),
            Token::Str("message"),
            Token::Struct { name: "Message", len: 2 },
            Token::Str("role"),
            Token::Enum { name: "Roles" },
            Token::Str("assistant"),
            Token::Unit,
            Token::Str("content"),
            Token::String("\nTest"),
            Token::StructEnd,
            Token::Str("finish_reason"),
            Token::String("stop"),
            Token::StructEnd
        ]);
    }

    #[test]
    fn test_de_usage() {
        let usage = Usage {
            prompt_tokens: 1,
            completion_tokens: 1,
            total_tokens: 2
        };
        
        assert_de_tokens(&usage, &[
            Token::Struct { name: "Usage", len: 3 },
            Token::Str("prompt_tokens"),
            Token::U16(1),
            Token::Str("completion_tokens"),
            Token::U16(1),
            Token::Str("total_tokens"),
            Token::U16(2),
            Token::StructEnd
        ])
    }

    #[test]
    fn test_deserialize_resp() {
        let resp: Resp<&str> = Resp {
            id: "chatcmpl-123",
            object: "chat.completion",
            created: 161444444,
            choices: vec![
                Choice {
                    index: 0,
                    message: Message::new(Roles::Assistant, "\nEarth is"),
                    finish_reason: "stop"
                }
            ],
            usage: Usage {
                prompt_tokens: 1,
                completion_tokens: 1,
                total_tokens: 2
            }
        };

        assert_de_tokens(&resp, &[
            Token::Struct { name: "Resp", len: 5 },
            Token::Str("id"),
            Token::BorrowedStr("chatcmpl-123"),
            Token::Str("object"),
            Token::BorrowedStr("chat.completion"),
            Token::Str("created"),
            Token::U64(161444444),
            Token::Str("choices"),
            Token::Seq { len: Some(1) },
            Token::Struct { name: "Choice", len: 3 },
            Token::Str("index"),
            Token::U64(0),
            Token::Str("message"),
            Token::Struct { name: "Message", len: 2 },
            Token::Str("role"),
            Token::Enum { name: "Roles" },
            Token::Str("assistant"),
            Token::Unit,
            Token::Str("content"),
            Token::BorrowedStr("\nEarth is"),
            Token::StructEnd,
            Token::Str("finish_reason"),
            Token::BorrowedStr("stop"),
            Token::StructEnd,
            Token::SeqEnd,
            Token::Str("usage"),
            Token::Struct { name: "Usage", len: 3 },
            Token::Str("prompt_tokens"),
            Token::U16(1),
            Token::Str("completion_tokens"),
            Token::U16(1),
            Token::Str("total_tokens"),
            Token::U16(2),
            Token::StructEnd,
            Token::StructEnd
        ])
    }

}