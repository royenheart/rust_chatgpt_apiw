use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
struct Permission {
    id: String,
    object: String,
    created: u64,
    allow_create_engine: bool,
    allow_sampling: bool,
    allow_logprobs: bool,
    allow_search_indices: bool,
    allow_view: bool,
    allow_fine_tuning: bool,
    organization: String,
    group: Option<String>,
    is_blocking: bool
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ModelInfo {
    id: String,
    object: String,
    created: u64,
    owned_by: String,
    permission: Vec<Permission>,
    root: String,
    parent: Option<String>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ListModelsResponse {
    object: String,
    data: Vec<ModelInfo>
}

#[derive(Serialize, Deserialize, Debug)]
struct Usage {
    prompt_tokens: u16,
    completion_tokens: u16,
    total_tokens: u16
}

#[derive(Serialize, Deserialize, Debug)]
struct Choice {
    text: String,
    index: u64,
    logprobs: Option<String>,
    finish_reason: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CompletionsResponse {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<Choice>,
    usage: Usage
}