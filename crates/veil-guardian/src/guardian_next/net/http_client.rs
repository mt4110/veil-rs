use crate::guardian_next::error::GuardianNextError;
use reqwest::header::{ETAG, IF_NONE_MATCH};

#[derive(Debug)]
pub enum FetchJsonResult<T> {
    Fetched { payload: T, etag: Option<String> },
    NotModified,
}

#[async_trait::async_trait]
pub trait HttpClient: Send + Sync {
    async fn fetch_json<T: serde::de::DeserializeOwned + Send>(
        &self,
        url: &str,
        etag: Option<&str>,
        timeout_ms: u64,
    ) -> Result<FetchJsonResult<T>, GuardianNextError>;
}

pub struct ReqwestHttpClient {
    client: reqwest::Client,
}

impl ReqwestHttpClient {
    pub fn new() -> Result<Self, GuardianNextError> {
        let client = reqwest::Client::builder()
            .user_agent("veil-guardian-next")
            .build()?;
        Ok(Self { client })
    }
}

#[async_trait::async_trait]
impl HttpClient for ReqwestHttpClient {
    async fn fetch_json<T: serde::de::DeserializeOwned + Send>(
        &self,
        url: &str,
        etag: Option<&str>,
        timeout_ms: u64,
    ) -> Result<FetchJsonResult<T>, GuardianNextError> {
        let mut req = self
            .client
            .get(url)
            .timeout(std::time::Duration::from_millis(timeout_ms));
        if let Some(et) = etag {
            req = req.header(IF_NONE_MATCH, et);
        }

        let resp = req.send().await?;
        if resp.status() == reqwest::StatusCode::NOT_MODIFIED {
            return Ok(FetchJsonResult::NotModified);
        }

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(GuardianNextError::http_status(status, body));
        }

        let etag = resp
            .headers()
            .get(ETAG)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());
        let payload = resp.json::<T>().await?;
        Ok(FetchJsonResult::Fetched { payload, etag })
    }
}
