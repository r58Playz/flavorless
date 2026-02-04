use blitz_traits::net::{
	Body, Bytes, Entry, EntryValue, HeaderMap, NetHandler, NetProvider, Request as BlitzRequest,
	http,
};
use data_url::DataUrl;
use js_sys::{Array, Function, JsString, Object, Promise, Uint8Array};
use thiserror::Error;
use wasm_bindgen::{JsCast, JsValue, prelude::wasm_bindgen};
use wasm_bindgen_futures::{JsFuture, spawn_local};
use web_sys::{FormData, Request, RequestInit, UrlSearchParams, console};

#[wasm_bindgen(typescript_custom_section)]
const BLITZ_FETCHER_FUNCTION: &'static str = r#"
type BlitzFetcherFunction = (req: Request) => Promise<[String, Uint8Aray]>;
"#;

#[wasm_bindgen]
extern "C" {
	#[wasm_bindgen(typescript_type = "BlitzFetcherFunction")]
	pub type BlitzFetcherFunction;
}

#[derive(Debug, Error)]
enum ProviderError {
	#[error("DataUrl: {0:?}")]
	DataUrl(data_url::DataUrlError),
	#[error("DataUrlBase64: {0:?}")]
	DataUrlBase64(data_url::forgiving_base64::InvalidBase64),
	#[error("HeaderToStr: {0:?}")]
	ToStrError(http::header::ToStrError),
	#[error("{0}")]
	Js(String),
}
impl From<http::header::ToStrError> for ProviderError {
	fn from(value: http::header::ToStrError) -> Self {
		Self::ToStrError(value)
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
impl From<JsValue> for ProviderError {
	fn from(value: JsValue) -> Self {
		Self::Js(format!("{value:?}"))
	}
}

pub struct Provider(BlitzFetcherFunction);
unsafe impl Send for Provider {}
unsafe impl Sync for Provider {}

impl Provider {
	pub fn new(fetcher: BlitzFetcherFunction) -> Self {
		Self(fetcher)
	}
}

impl Provider {
	fn get_headers(map: HeaderMap, content_ty: &str) -> Result<JsValue, ProviderError> {
		let array = Array::new();

		for key in map.keys() {
			let key_js = key.to_string().into();
			for val in map.get_all(key) {
				array.push(&Array::of2(&key_js, &val.to_str()?.into()));
			}
		}
		array.push(&Array::of2(&"Content-Type".into(), &content_ty.into()).into());

		Ok(Object::from_entries(&array.into())?.into())
	}

	fn get_body(body: Body, formdata: bool) -> Result<JsValue, ProviderError> {
		Ok(match body {
			Body::Form(mut form) if formdata => {
				let js = FormData::new()?;
				for Entry { name, value } in form.0.drain(..) {
					match value {
						EntryValue::String(value) => js.set_with_str(&name, &value)?,
						_ => {
							console::warn_1(&"invalid formdata type, skipping".into());
						}
					}
				}
				js.into()
			}
			Body::Form(mut form) => {
				let js = UrlSearchParams::new()?;
				for Entry { name, value } in form.0.drain(..) {
					match value {
						EntryValue::String(value) => js.set(&name, &value),
						_ => {
							console::warn_1(&"invalid formdata type, skipping".into());
						}
					}
				}
				js.into()
			}
			Body::Bytes(bytes) => Uint8Array::new_from_slice(&bytes).into(),
			Body::Empty => JsValue::UNDEFINED,
		})
	}

	async fn fetch_inner(
		fetcher: BlitzFetcherFunction,
		request: BlitzRequest,
	) -> Result<(String, Bytes), ProviderError> {
		Ok(match request.url.scheme() {
			"data" => {
				let data_url = DataUrl::process(request.url.as_str())?;
				let decoded = data_url.decode_to_vec()?;
				(request.url.to_string(), Bytes::from(decoded.0))
			}
			_ => {
				let func = fetcher.unchecked_into::<Function>();
				let init = RequestInit::new();
				init.set_method(&request.method.to_string());
				init.set_headers(&Self::get_headers(request.headers, &request.content_type)?);
				init.set_body(&Self::get_body(
					request.body,
					request.content_type == "multipart/form-data",
				)?);

				let req = Request::new_with_str_and_init(&request.url.to_string(), &init)?;

				let promise: Promise = func.call1(&JsValue::NULL, &req.into())?.unchecked_into();
				let res: Array = JsFuture::from(promise).await?.unchecked_into();
				let url: JsString = res.at(0).unchecked_into();
				let bytes: Uint8Array = res.at(1).unchecked_into();

				(url.into(), bytes.to_vec().into())
			}
		})
	}
}

impl NetProvider for Provider {
	fn fetch(&self, _doc_id: usize, request: BlitzRequest, handler: Box<dyn NetHandler>) {
		let func = self.0.clone();
		spawn_local(async move {
			let result = Self::fetch_inner(func.into(), request).await;

			match result {
				Ok((response_url, bytes)) => {
					handler.bytes(response_url, bytes);
				}
				Err(x) => {
					console::warn_2(&"fetch failed:".into(), &x.to_string().into());
				}
			};
		});
	}
}
