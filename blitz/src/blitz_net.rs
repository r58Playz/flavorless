//! Networking (HTTP, filesystem, Data URIs) for Blitz with WASM support
//!
//! Provides an implementation of the [`blitz_traits::net::NetProvider`] trait
//! with wasm-bindgen-futures integration and always-enabled caching.

use blitz_traits::net::{Body, Bytes, NetHandler, NetProvider, NetWaker, Request};
use data_url::DataUrl;
use std::sync::Arc;
use wasm_bindgen_futures::spawn_local;

const USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64; rv:60.0) Gecko/20100101 Firefox/81.0";

pub struct Provider {
    client: reqwest::Client,
    waker: Arc<dyn NetWaker>,
}

impl Provider {
    pub fn new(waker: Option<Arc<dyn NetWaker>>) -> Self {
        let builder = reqwest::Client::builder();
        let client = builder.build().unwrap();

        let waker = waker.unwrap_or(Arc::new(DummyNetWaker));
        Self { client, waker }
    }

    pub fn shared(waker: Option<Arc<dyn NetWaker>>) -> Arc<dyn NetProvider> {
        Arc::new(Self::new(waker))
    }

    pub fn is_empty(&self) -> bool {
        Arc::strong_count(&self.waker) == 1
    }

    pub fn count(&self) -> usize {
        Arc::strong_count(&self.waker) - 1
    }
}

impl Provider {
    async fn fetch_inner(
        client: reqwest::Client,
        request: Request,
    ) -> Result<(String, Bytes), ProviderError> {
        Ok(match request.url.scheme() {
            "data" => {
                let data_url = DataUrl::process(request.url.as_str())?;
                let decoded = data_url.decode_to_vec()?;
                (request.url.to_string(), Bytes::from(decoded.0))
            }
            "file" => {
                let file_content = std::fs::read(request.url.path())?;
                (request.url.to_string(), Bytes::from(file_content))
            }
            _ => {
                let response = client
                    .request(request.method, request.url)
                    .headers(request.headers)
                    .header("Content-Type", request.content_type.as_str())
                    .header("User-Agent", USER_AGENT)
                    .apply_body(request.body, request.content_type.as_str())
                    .await
                    .send()
                    .await?;

                (response.url().to_string(), response.bytes().await?)
            }
        })
    }

    #[allow(clippy::type_complexity)]
    pub fn fetch_with_callback(
        &self,
        request: Request,
        callback: Box<dyn FnOnce(Result<(String, Bytes), ProviderError>) + Send + Sync + 'static>,
    ) {
        let client = self.client.clone();
        spawn_local(async move {
            let result = Self::fetch_inner(client, request).await;
            callback(result);
        });
    }

    pub async fn fetch_async(&self, request: Request) -> Result<(String, Bytes), ProviderError> {
        let client = self.client.clone();
        Self::fetch_inner(client, request).await
    }
}

impl NetProvider for Provider {
    fn fetch(&self, doc_id: usize, request: Request, handler: Box<dyn NetHandler>) {
        let client = self.client.clone();
        let waker = self.waker.clone();
        
        spawn_local(async move {
            let result = Self::fetch_inner(client, request).await;

            // Call the waker to notify of completed network request
            waker.wake(doc_id);

            match result {
                Ok((response_url, bytes)) => {
                    handler.bytes(response_url, bytes);
                }
                Err(_) => {
                    // Error handling
                }
            };
        });
    }
}

#[derive(Debug)]
pub enum ProviderError {
    Io(std::io::Error),
    DataUrl(data_url::DataUrlError),
    DataUrlBase64(data_url::forgiving_base64::InvalidBase64),
    ReqwestError(reqwest::Error),
}

impl From<std::io::Error> for ProviderError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<data_url::DataUrlError> for ProviderError {
    fn from(value: data_url::DataUrlError) -> Self {
        Self::DataUrl(value)
    }
}

impl From<data_url::forgiving_base64::InvalidBase64> for ProviderError {
    fn from(value: data_url::forgiving_base64::InvalidBase64) -> Self {
        Self::DataUrlBase64(value)
    }
}

impl From<reqwest::Error> for ProviderError {
    fn from(value: reqwest::Error) -> Self {
        Self::ReqwestError(value)
    }
}

trait ReqwestExt {
    async fn apply_body(self, body: Body, content_type: &str) -> Self;
}

impl ReqwestExt for reqwest::RequestBuilder {
    async fn apply_body(self, body: Body, content_type: &str) -> Self {
        match body {
            Body::Bytes(bytes) => self.body(bytes),
            Body::Form(form_data) => match content_type {
                "application/x-www-form-urlencoded" => self.form(&form_data),
                "multipart/form-data" => {
                    use blitz_traits::net::Entry;
                    use blitz_traits::net::EntryValue;
                    let mut form_data = form_data;
                    let mut form = reqwest::multipart::Form::new();
                    for Entry { name, value } in form_data.0.drain(..) {
                        form = match value {
                            EntryValue::String(value) => form.text(name, value),
                            EntryValue::File(_) => {
                                // File uploads not supported in WASM environment
                                form
                            }
                            EntryValue::EmptyFile => form.part(
                                name,
                                reqwest::multipart::Part::bytes(&[])
                                    .mime_str("application/octet-stream")
                                    .unwrap(),
                            ),
                        };
                    }
                    self.multipart(form)
                }
                _ => self,
            },
            Body::Empty => self,
        }
    }
}

struct DummyNetWaker;
impl NetWaker for DummyNetWaker {
    fn wake(&self, _client_id: usize) {}
}
