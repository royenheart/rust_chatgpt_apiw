pub mod response;
pub mod request;

/**
 * ======
 * GLOBAL VAR
 * ======
 */

pub const POST_URL: &str = "https://api.openai.com/v1/chat/completions";
pub const AUTH_METHOD: &str = "Bearer";
pub const AUTH_ORG: &str = "OpenAI-Organization";
pub const AUTH_CONTENT_TYPE: &str = "application/json";
/// Max number of chat completion choices generated for each input message 
const MAX_N: u32 = 1024;