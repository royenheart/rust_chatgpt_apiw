use async_trait::async_trait;
use reqwest::header::HeaderMap;

pub trait GenHeaders {
    fn gen_headers(&self) -> HeaderMap;
}

#[async_trait]
pub trait AsyncPerform<Auth: GenHeaders> {
    type Respr;
    async fn perform(&self, auth: &Auth) -> Result<Self::Respr, String>; 
}