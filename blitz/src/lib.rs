use std::str::FromStr;
use std::sync::Arc;

use anyhow::Context;
use blitz_dom::{Document, DocumentConfig, FontContext};
use blitz_html::HtmlDocument;
use blitz_traits::{
    events::{UiEvent, BlitzPointerEvent, BlitzWheelEvent, BlitzKeyEvent, PointerCoords, BlitzPointerId, MouseEventButton},
    shell::{ColorScheme, Viewport},
};
use fontique::Blob;
use keyboard_types::{Code, Key, Location, Modifiers};
use wasm_bindgen::{JsError, JsValue, prelude::wasm_bindgen};
use web_sys::{OffscreenCanvas, PointerEvent, WheelEvent, KeyboardEvent};

use crate::{anyrender::VelloScenePainter, canvas::CanvasVelloScene};

pub mod anyrender;
pub mod blitz_net;
pub mod canvas;

pub const CANTARELL_FONT: &[u8] = include_bytes!("../assets/cantarell.otf");

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
pub struct BlitzRendererEvent(UiEvent);

#[wasm_bindgen]
pub struct BlitzRenderer {
    scene: CanvasVelloScene,
    doc: HtmlDocument,
}
#[wasm_bindgen]
impl BlitzRenderer {
    async fn _new(html: String, canvas: OffscreenCanvas, scale: f32) -> anyhow::Result<BlitzRenderer> {
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

        Ok(BlitzRenderer {
            scene: CanvasVelloScene::new(canvas, scale)
                .await
                .context("failed to create vello scene")?,
            doc: HtmlDocument::from_html(&html, config),
        })
    }

    #[wasm_bindgen]
    pub async fn new(html: String, canvas: OffscreenCanvas, scale: f32) -> Result<BlitzRenderer, JsError> {
        Self::_new(html, canvas, scale).await.map_err(anyhow_to_obj)
    }

    async fn _resize(&mut self, canvas: OffscreenCanvas, scale: f32) -> anyhow::Result<()> {
        let mut viewport = self.doc.viewport_mut();
        viewport.window_size = (canvas.width(), canvas.height());
        viewport.set_hidpi_scale(scale);

        self.scene = CanvasVelloScene::new(canvas, scale)
            .await
            .context("failed to create vello scene")?;
        Ok(())
    }
    #[wasm_bindgen]
    pub async fn resize(&mut self, canvas: OffscreenCanvas, scale: f32) -> Result<(), JsError> {
        self._resize(canvas, scale).await.map_err(anyhow_to_obj)
    }

    #[wasm_bindgen]
    pub fn render(&mut self, time: f64) -> Result<(), JsError> {
        self.doc.resolve(time);
        self.scene
            .render(|scene, width, height, scale| {
                blitz_paint::paint_scene(
                    &mut VelloScenePainter::new(scene),
                    &self.doc,
                    scale as f64,
                    width,
                    height,
                    0,
                    0,
                );
            })
            .map_err(anyhow_to_obj)
    }

    #[wasm_bindgen]
    pub fn event(&mut self, event: BlitzRendererEvent) {
        self.doc.handle_ui_event(event.0);
    }

    #[wasm_bindgen]
    pub fn event_pointer(
        &self,
        web_event: &PointerEvent,
        canvas_x: f32,
        canvas_y: f32,
    ) -> Result<BlitzRendererEvent, JsError> {
        let id = match web_event.pointer_type().as_str() {
            "mouse" => BlitzPointerId::Mouse,
            "pen" => BlitzPointerId::Pen,
            "touch" => BlitzPointerId::Finger(web_event.pointer_id() as u64),
            _ => BlitzPointerId::Mouse,
        };

        let button = match web_event.button() {
            0 => MouseEventButton::Main,
            1 => MouseEventButton::Auxiliary,
            2 => MouseEventButton::Secondary,
            3 => MouseEventButton::Fourth,
            4 => MouseEventButton::Fifth,
            _ => MouseEventButton::Main,
        };

        let mut mods = Modifiers::empty();
        if web_event.alt_key() {
            mods |= Modifiers::ALT;
        }
        if web_event.ctrl_key() {
            mods |= Modifiers::CONTROL;
        }
        if web_event.shift_key() {
            mods |= Modifiers::SHIFT;
        }
        if web_event.meta_key() {
            mods |= Modifiers::META;
        }

        let mut buttons = 0u8;
        if web_event.buttons() & 0x01 != 0 {
            buttons |= 0x01;
        }
        if web_event.buttons() & 0x02 != 0 {
            buttons |= 0x02;
        }
        if web_event.buttons() & 0x04 != 0 {
            buttons |= 0x04;
        }
        if web_event.buttons() & 0x08 != 0 {
            buttons |= 0x08;
        }
        if web_event.buttons() & 0x10 != 0 {
            buttons |= 0x10;
        }

        let event = BlitzPointerEvent {
            id,
            is_primary: web_event.is_primary(),
            coords: PointerCoords {
                page_x: (web_event.page_x() as f32) - canvas_x,
                page_y: (web_event.page_y() as f32) - canvas_y,
                screen_x: web_event.screen_x() as f32,
                screen_y: web_event.screen_y() as f32,
                client_x: (web_event.client_x() as f32) - canvas_x,
                client_y: (web_event.client_y() as f32) - canvas_y,
            },
            button,
            buttons: blitz_traits::events::MouseEventButtons::from_bits_truncate(buttons),
            mods,
            details: Default::default(),
        };

        let ui_event = match web_event.type_().as_str() {
            "pointerdown" => UiEvent::PointerDown(event),
            "pointerup" => UiEvent::PointerUp(event),
            _ => UiEvent::PointerMove(event),
        };

        Ok(BlitzRendererEvent(ui_event))
    }

    #[wasm_bindgen]
    pub fn event_wheel(
        &self,
        web_event: &WheelEvent,
        canvas_x: f32,
        canvas_y: f32,
    ) -> Result<BlitzRendererEvent, JsError> {
        let mut mods = Modifiers::empty();
        if web_event.alt_key() {
            mods |= Modifiers::ALT;
        }
        if web_event.ctrl_key() {
            mods |= Modifiers::CONTROL;
        }
        if web_event.shift_key() {
            mods |= Modifiers::SHIFT;
        }
        if web_event.meta_key() {
            mods |= Modifiers::META;
        }

        let mut buttons = 0u8;
        if web_event.buttons() & 0x01 != 0 {
            buttons |= 0x01;
        }
        if web_event.buttons() & 0x02 != 0 {
            buttons |= 0x02;
        }
        if web_event.buttons() & 0x04 != 0 {
            buttons |= 0x04;
        }
        if web_event.buttons() & 0x08 != 0 {
            buttons |= 0x08;
        }
        if web_event.buttons() & 0x10 != 0 {
            buttons |= 0x10;
        }

        let delta = match web_event.delta_mode() {
            0 => blitz_traits::events::BlitzWheelDelta::Pixels(-web_event.delta_x(), -web_event.delta_y()),
            1 => blitz_traits::events::BlitzWheelDelta::Lines(-web_event.delta_x(), -web_event.delta_y()),
            _ => blitz_traits::events::BlitzWheelDelta::Pixels(-web_event.delta_x(), -web_event.delta_y()),
        };

        let event = BlitzWheelEvent {
            delta,
            coords: PointerCoords {
                page_x: (web_event.page_x() as f32) - canvas_x,
                page_y: (web_event.page_y() as f32) - canvas_y,
                screen_x: web_event.screen_x() as f32,
                screen_y: web_event.screen_y() as f32,
                client_x: (web_event.client_x() as f32) - canvas_x,
                client_y: (web_event.client_y() as f32) - canvas_y,
            },
            buttons: blitz_traits::events::MouseEventButtons::from_bits_truncate(buttons),
            mods,
        };

        Ok(BlitzRendererEvent(UiEvent::Wheel(event)))
    }

    #[wasm_bindgen]
    pub fn event_keyboard(&self, web_event: &KeyboardEvent) -> Result<BlitzRendererEvent, JsError> {
        let mut mods = Modifiers::empty();
        if web_event.alt_key() {
            mods |= Modifiers::ALT;
        }
        if web_event.ctrl_key() {
            mods |= Modifiers::CONTROL;
        }
        if web_event.shift_key() {
            mods |= Modifiers::SHIFT;
        }
        if web_event.meta_key() {
            mods |= Modifiers::META;
        }

        let code = Code::from_str(&web_event.code()).unwrap_or(Code::Unidentified);
        let key = Key::from_str(&web_event.key()).unwrap_or(Key::Unidentified);
        let location = match web_event.location() {
            0 => Location::Standard,
            1 => Location::Left,
            2 => Location::Right,
            3 => Location::Numpad,
            _ => Location::Standard,
        };

        let state = if web_event.type_() == "keydown" {
            blitz_traits::events::KeyState::Pressed
        } else {
            blitz_traits::events::KeyState::Released
        };

        let event = BlitzKeyEvent {
            key,
            code,
            modifiers: mods,
            location,
            is_auto_repeating: web_event.repeat(),
            is_composing: web_event.is_composing(),
            state,
            text: None,
        };

        let ui_event = match web_event.type_().as_str() {
            "keydown" => UiEvent::KeyDown(event),
            _ => UiEvent::KeyUp(event),
        };

        Ok(BlitzRendererEvent(ui_event))
    }
}
