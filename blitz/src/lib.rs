use std::sync::Arc;

use anyhow::Context;
use blitz_dom::{DocumentConfig, FontContext};
use blitz_html::HtmlDocument;
use blitz_traits::shell::{ColorScheme, Viewport};
use fontique::Blob;
use js_sys::Array;
use wasm_bindgen::{JsError, JsValue, prelude::wasm_bindgen};
use web_sys::OffscreenCanvas;

use crate::{anyrender::VelloScenePainter, canvas::CanvasVelloScene, document::{BlitzDocument, BlitzEventHandler}};

pub mod anyrender;
pub mod blitz_net;
pub mod canvas;
pub mod document;

pub const CANTARELL_FONT: &[u8] = include_bytes!("../assets/cantarell.otf");

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

fn obj_to_anyhow(val: impl Into<JsValue>) -> anyhow::Error {
    anyhow::Error::msg(format!("{:?}", val.into()))
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
        canvas: OffscreenCanvas,
        scale: f32,
    ) -> anyhow::Result<(BlitzRenderer, BlitzDocument, BlitzEventHandler)> {
        let mut font_ctx = FontContext::default();
        font_ctx
            .collection
            .register_fonts(Blob::new(Arc::new(CANTARELL_FONT)), None);

        let config = DocumentConfig {
            font_ctx: Some(font_ctx),
            viewport: Some(Viewport::new(
                canvas.width(),
                canvas.height(),
                scale as f32,
                ColorScheme::Dark,
            )),
            net_provider: Some(crate::blitz_net::Provider::shared(None)),
            ..Default::default()
        };

        Ok((
            BlitzRenderer {
                scene: CanvasVelloScene::new(canvas, scale)
                    .await
                    .context("failed to create vello scene")?,
            },
            BlitzDocument::new(HtmlDocument::from_html(&html, config)),
			BlitzEventHandler::new(),
        ))
    }

    #[wasm_bindgen]
    pub async fn new(
        html: String,
        canvas: OffscreenCanvas,
        scale: f32,
    ) -> Result<BlitzRendererResult, JsError> {
        Self::_new(html, canvas, scale)
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
