use std::sync::Arc;

use anyhow::Context;
use blitz_dom::{DocumentConfig, FontContext};
use blitz_html::HtmlDocument;
use blitz_traits::shell::{ColorScheme, Viewport};
use fontique::Blob;
use js_sys::Array;
use wasm_bindgen::{JsError, JsValue, prelude::wasm_bindgen};
use web_sys::OffscreenCanvas;

use crate::{
	anyrender::VelloScenePainter,
	blitz_net::{BlitzFetcherFunction, Provider as NetProvider},
	canvas::CanvasVelloScene,
	document::{BlitzDocument, BlitzEventHandler},
};

pub mod anyrender;
pub mod blitz_net;
pub mod canvas;
pub mod document;

#[wasm_bindgen(typescript_custom_section)]
const BLITZ_RENDERER_RESULT: &'static str = r#"
type BlitzRendererResult = [BlitzRenderer, BlitzDocument, BlitzEventHandler];
"#;

#[wasm_bindgen]
extern "C" {
	#[wasm_bindgen(typescript_type = "BlitzRendererResult")]
	pub type BlitzRendererResult;
}

#[wasm_bindgen(start)]
pub fn start() {
	console_error_panic_hook::set_once();
}

fn anyhow_to_obj(val: anyhow::Error) -> JsError {
	JsError::new(&format!("{:?}", val))
}

#[wasm_bindgen]
pub struct BlitzRenderer {
	scene: CanvasVelloScene,
}
#[wasm_bindgen]
impl BlitzRenderer {
	async fn _new(
		html: String,
		base: String,
		fetcher: BlitzFetcherFunction,
		canvas: OffscreenCanvas,
		scale: f32,
	) -> anyhow::Result<(BlitzRenderer, BlitzDocument, BlitzEventHandler)> {
		let mut font_ctx = FontContext::default();
		font_ctx.collection.register_fonts(
			Blob::new(Arc::new(include_bytes!("./AdwaitaSans-Regular.ttf"))),
			None,
		);

		let config = DocumentConfig {
			font_ctx: Some(font_ctx),
			viewport: Some(Viewport::new(
				canvas.width(),
				canvas.height(),
				scale as f32,
				ColorScheme::Dark,
			)),
			base_url: Some(base),
			net_provider: Some(Arc::new(NetProvider::new(fetcher))),
			..Default::default()
		};

		let mut doc = HtmlDocument::from_html(&html, config);
		doc.add_user_agent_stylesheet(":root { font-family: Adwaita Sans; }");

		Ok((
			BlitzRenderer {
				scene: CanvasVelloScene::new(canvas, scale)
					.await
					.context("failed to create vello scene")?,
			},
			BlitzDocument::new(doc),
			BlitzEventHandler::new(),
		))
	}

	#[wasm_bindgen]
	pub async fn new(
		html: String,
		base: String,
		fetcher: BlitzFetcherFunction,
		canvas: OffscreenCanvas,
		scale: f32,
	) -> Result<BlitzRendererResult, JsError> {
		Self::_new(html, base, fetcher, canvas, scale)
			.await
			.map(|x| JsValue::from(Array::of3(&x.0.into(), &x.1.into(), &x.2.into())).into())
			.map_err(anyhow_to_obj)
	}

	async fn _resize(
		&mut self,
		doc: &mut BlitzDocument,
		canvas: OffscreenCanvas,
		scale: f32,
	) -> anyhow::Result<()> {
		let mut viewport = doc.viewport();
		viewport.window_size = (canvas.width(), canvas.height());
		viewport.set_hidpi_scale(scale);

		self.scene = CanvasVelloScene::new(canvas, scale)
			.await
			.context("failed to create vello scene")?;
		Ok(())
	}
	#[wasm_bindgen]
	pub async fn resize(
		&mut self,
		doc: &mut BlitzDocument,
		canvas: OffscreenCanvas,
		scale: f32,
	) -> Result<(), JsError> {
		self._resize(doc, canvas, scale)
			.await
			.map_err(anyhow_to_obj)
	}

	#[wasm_bindgen]
	pub fn render(&mut self, doc: &mut BlitzDocument, time: f64) -> Result<(), JsError> {
		doc.resolve(time);
		self.scene
			.render(|scene, width, height, scale| {
				blitz_paint::paint_scene(
					&mut VelloScenePainter::new(scene),
					doc.doc(),
					scale as f64,
					width,
					height,
					0,
					0,
				);
			})
			.map_err(anyhow_to_obj)
	}
}
