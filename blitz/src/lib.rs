use std::sync::Arc;

use anyhow::Context;
use blitz_dom::{DocumentConfig, FontContext};
use blitz_html::{HtmlDocument, HtmlProvider};
use blitz_traits::shell::{ClipboardError, ColorScheme, ShellProvider, Viewport};
use fontique::Blob;
use js_sys::{Array, Function};
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
pub struct BlitzShellProvider {
	set_clipboard: Function,
}
unsafe impl Send for BlitzShellProvider {}
unsafe impl Sync for BlitzShellProvider {}
#[wasm_bindgen]
impl BlitzShellProvider {
	#[wasm_bindgen(constructor)]
	pub fn new(set_clipboard: Function) -> Self {
		Self { set_clipboard }
	}
}
impl ShellProvider for BlitzShellProvider {
	fn set_clipboard_text(&self, text: String) -> Result<(), ClipboardError> {
		self.set_clipboard
			.call1(&JsValue::NULL, &text.into())
			.map(|_| ())
			.map_err(|_| ClipboardError)
	}
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
		shell: BlitzShellProvider,
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
			shell_provider: Some(Arc::new(shell)),
			html_parser_provider: Some(Arc::new(HtmlProvider)),
			..Default::default()
		};

		let mut doc = HtmlDocument::from_html(&html, config);
		doc.add_user_agent_stylesheet(":root, input, textarea { font-family: Adwaita Sans; }");

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
		shell: BlitzShellProvider,
		canvas: OffscreenCanvas,
		scale: f32,
	) -> Result<BlitzRendererResult, JsError> {
		Self::_new(html, base, fetcher, shell, canvas, scale)
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

	fn loader(width: u32, scale: f32, time: f64, scene: &mut vello::Scene) -> u32 {
		use vello::{
			kurbo::{Affine, Rect},
			peniko::{Color, Fill},
		};

		let bar_height = 8.0 * scale as f64;
		let cycle_time = 1.75;
		let progress = (time % cycle_time) / cycle_time;

		// Helper to interpolate keyframe values
		let keyframe = |t: f64, keyframes: &[(f64, f64)]| -> f64 {
			for i in 0..keyframes.len() - 1 {
				let (t1, v1) = keyframes[i];
				let (t2, v2) = keyframes[i + 1];
				if t >= t1 && t < t2 {
					let local = (t - t1) / (t2 - t1);
					return v1 + (v2 - v1) * local;
				}
			}
			keyframes[keyframes.len() - 1].1
		};

		// Bar 1 animations
		let bar1_right = keyframe(
			progress,
			&[(0.0, 1.0), (0.5714, 0.0), (0.7142, 0.0), (1.0, 1.0)],
		);
		let bar1_left = keyframe(
			progress,
			&[(0.0, 0.0), (0.1429, 0.0), (0.7142, 1.0), (1.0, 0.0)],
		);

		// Bar 2 animations
		let bar2_right = keyframe(
			progress,
			&[(0.0, 1.0), (0.3714, 1.0), (0.8571, 0.0), (1.0, 0.0)],
		);
		let bar2_left = keyframe(progress, &[(0.0, 0.0), (0.5143, 0.0), (1.0, 1.0)]);

		// Track 1 animations
		let track1_left = keyframe(progress, &[(0.0, 0.0), (0.5714, 1.0), (1.0, 1.0)]);
		let track1_right = 0.0;

		// Track 2 animations
		let track2_left = keyframe(
			progress,
			&[(0.0, 0.0), (0.3714, 0.0), (0.8571, 1.0), (1.0, 1.0)],
		);
		let track2_right = keyframe(
			progress,
			&[
				(0.0, 1.0),
				(0.1429, 1.0),
				(0.7142, 0.0),
				(0.8571, 0.0),
				(1.0, 1.0),
			],
		);

		// Track 3 animations
		let track3_left = 0.0;
		let track3_right = keyframe(progress, &[(0.0, 1.0), (0.5143, 1.0), (1.0, 0.0)]);

		let margin = 4.0 * scale as f64;
		let w = width as f64;

		// Colors (approximating M3 secondary-container and primary)
		let track_color = Color::from_rgba8(0, 0, 0, 0);
		let bar_color = Color::from_rgb8(100, 150, 255);

		// Draw tracks (background)
		let track1_x = w * track1_left + margin;
		let track1_w = w * track1_right - track1_x;
		if track1_w > 0.0 {
			let rect = Rect::new(track1_x, 0.0, track1_x + track1_w, bar_height);
			scene.fill(Fill::NonZero, Affine::IDENTITY, track_color, None, &rect);
		}

		let track2_x = w * track2_left + margin;
		let track2_w = w - w * track2_right - track2_x - margin;
		if track2_w > 0.0 {
			let rect = Rect::new(track2_x, 0.0, track2_x + track2_w, bar_height);
			scene.fill(Fill::NonZero, Affine::IDENTITY, track_color, None, &rect);
		}

		let track3_x = w * track3_left;
		let track3_w = w * (1.0 - track3_right) - track3_x - margin;
		if track3_w > 0.0 {
			let rect = Rect::new(track3_x, 0.0, track3_x + track3_w, bar_height);
			scene.fill(Fill::NonZero, Affine::IDENTITY, track_color, None, &rect);
		}

		// Draw bars (foreground)
		let bar1_x = w * bar1_left;
		let bar1_w = w - w * bar1_right - bar1_x;
		if bar1_w > 0.0 {
			let rect = Rect::new(bar1_x, 0.0, bar1_x + bar1_w, bar_height);
			scene.fill(Fill::NonZero, Affine::IDENTITY, bar_color, None, &rect);
		}

		let bar2_x = w * bar2_left;
		let bar2_w = w - w * bar2_right - bar2_x;
		if bar2_w > 0.0 {
			let rect = Rect::new(bar2_x, 0.0, bar2_x + bar2_w, bar_height);
			scene.fill(Fill::NonZero, Affine::IDENTITY, bar_color, None, &rect);
		}

		bar_height as u32
	}

	#[wasm_bindgen]
	pub fn render(
		&mut self,
		doc: &mut BlitzDocument,
		loading: bool,
		time: f64,
	) -> Result<(), JsError> {
		self.scene
			.render(|scene, width, height, scale| {
				let offset = if loading {
					Self::loader(width, scale, time, scene)
				} else {
					0
				};

				blitz_paint::paint_scene(
					&mut VelloScenePainter::new(scene),
					doc.doc(),
					scale as f64,
					width,
					height - offset,
					0,
					offset,
				);
			})
			.map_err(anyhow_to_obj)
	}
}
