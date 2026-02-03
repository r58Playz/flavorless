use blitz_dom::{
	Document, DocumentMutator, EventDriver, EventHandler, Namespace, Node, QualName, ns,
};
use blitz_html::HtmlDocument;
use blitz_traits::{
	events::{
		BlitzKeyEvent, BlitzPointerEvent, BlitzPointerId, BlitzWheelEvent, DomEventKind,
		MouseEventButton, PointerCoords, UiEvent,
	},
	shell::Viewport,
};
use js_sys::Function;
use keyboard_types::{Code, Key, Location, Modifiers};
use std::{
	any::Any,
	collections::HashMap,
	mem::transmute,
	ops::{Deref, DerefMut},
	str::FromStr,
};
use wasm_bindgen::{JsError, JsValue, prelude::wasm_bindgen};
use web_sys::{Event as JsEvent, KeyboardEvent, PointerEvent, WheelEvent, console};

#[wasm_bindgen]
pub struct BlitzNode(pub usize);

#[wasm_bindgen]
impl BlitzNode {
	pub fn new(doc: &mut BlitzDocument, name: String) -> Self {
		Self(
			doc.mutator()
				.create_element(QualName::new(None, ns!(html), name.into()), vec![]),
		)
	}

	pub fn new_ns(doc: &mut BlitzDocument, name: String, ns: String) -> Self {
		Self(doc.mutator().create_element(
			QualName::new(None, Namespace::from(ns), name.into()),
			vec![],
		))
	}

	pub fn new_text(doc: &mut BlitzDocument, text: &str) -> Self {
		Self(doc.mutator().create_text_node(text))
	}

	pub fn new_comment(doc: &mut BlitzDocument) -> Self {
		Self(doc.mutator().create_comment_node())
	}

	pub fn append(&self, doc: &mut BlitzDocument, child: &BlitzNode) {
		doc.mutator().append_children(self.0, &[child.0]);
	}
	pub fn remove(&self, doc: &mut BlitzDocument, child: &BlitzNode) {
		doc.mutator().remove_and_drop_node(child.0);
	}
	pub fn insert(&self, doc: &mut BlitzDocument, child: &BlitzNode, anchor: &BlitzNode) {
		doc.mutator().insert_nodes_before(anchor.0, &[child.0]);
	}
	pub fn replace(&self, doc: &mut BlitzDocument, child: &BlitzNode) {
		let mut mutator = doc.mutator();
		mutator.replace_node_with(self.0, &[child.0]);
		mutator.remove_and_drop_node(self.0);
	}

	pub fn parent(&self, doc: &BlitzDocument) -> Result<Option<BlitzNode>, JsError> {
		Ok(doc.node(self)?.parent.map(BlitzNode))
	}
	pub fn children(&self, doc: &BlitzDocument) -> Result<Vec<BlitzNode>, JsError> {
		Ok(doc
			.node(self)?
			.children
			.iter()
			.copied()
			.map(BlitzNode)
			.collect())
	}
	pub fn next_sibling(&self, doc: &BlitzDocument) -> Result<Option<BlitzNode>, JsError> {
		Ok(doc.node(self)?.forward(1).map(|x| BlitzNode(x.id)))
	}

	pub fn get_attribute(
		&self,
		doc: &BlitzDocument,
		name: String,
	) -> Result<Option<String>, JsError> {
		Ok(doc.node(self)?.attr(name.into()).map(ToOwned::to_owned))
	}
	pub fn set_attribute(&self, doc: &mut BlitzDocument, name: String, value: &str) {
		doc.mutator()
			.set_attribute(self.0, QualName::new(None, ns!(html), name.into()), value);
	}
	pub fn remove_attribute(&self, doc: &mut BlitzDocument, name: String) {
		doc.mutator()
			.clear_attribute(self.0, QualName::new(None, ns!(html), name.into()));
	}

	pub fn add_event_listener(
		&self,
		events: &mut BlitzEventHandler,
		listener: &str,
		func: Function,
	) -> Result<(), JsError> {
		events.add_listener(self.0, listener, func)
	}
	pub fn remove_event_listener(
		&self,
		events: &mut BlitzEventHandler,
		listener: &str,
		func: Function,
	) -> Result<(), JsError> {
		events.remove_listener(self.0, listener, func)
	}

	pub fn get_data(&self, doc: &BlitzDocument) -> Result<Option<String>, JsError> {
		Ok(doc.node(self)?.text_data().map(|x| x.content.clone()))
	}
	pub fn set_data(&self, doc: &mut BlitzDocument, data: &str) {
		doc.mutator().set_node_text(self.0, data);
	}

	pub fn set_inner_text(&self, doc: &mut BlitzDocument, text: &str) {
		let mut mutator = doc.mutator();
		mutator.remove_and_drop_all_children(self.0);
		let node = mutator.create_text_node(text);
		mutator.append_children(self.0, &[node]);
	}

	pub fn get_inner_html(&self, doc: &BlitzDocument) -> Result<String, JsError> {
		Ok(doc.node(self)?.inner_html())
	}
	pub fn set_inner_html(&self, doc: &mut BlitzDocument, html: &str) {
		doc.mutator().set_inner_html(self.0, html);
	}
	pub fn get_outer_html(&self, doc: &BlitzDocument) -> Result<String, JsError> {
		Ok(doc.node(self)?.outer_html())
	}
}

impl From<&Node> for BlitzNode {
	fn from(value: &Node) -> Self {
		BlitzNode(value.id)
	}
}

#[wasm_bindgen]
pub struct BlitzEventHandler {
	listeners: HashMap<(usize, u8), Vec<Function>>,
	temp_override: Option<Function>,
}
#[wasm_bindgen]
impl BlitzEventHandler {
	pub fn set_doc_overrider(&mut self, func: Function) {
		self.temp_override = Some(func);
	}
}
impl BlitzEventHandler {
	pub fn new() -> Self {
		Self {
			listeners: HashMap::new(),
			temp_override: None,
		}
	}

	pub fn add_listener(
		&mut self,
		node: usize,
		event_kind: &str,
		func: Function,
	) -> Result<(), JsError> {
		let kind =
			Self::str_to_kind(event_kind).ok_or_else(|| JsError::new("Invalid event kind"))?;
		self.listeners
			.entry((node, kind.discriminant()))
			.or_default()
			.push(func);
		Ok(())
	}

	pub fn remove_listener(
		&mut self,
		node: usize,
		event_kind: &str,
		func: Function,
	) -> Result<(), JsError> {
		let kind =
			Self::str_to_kind(event_kind).ok_or_else(|| JsError::new("Invalid event kind"))?;
		self.listeners
			.entry((node, kind.discriminant()))
			.and_modify(|x| x.retain(|x| *x != func));
		Ok(())
	}

	fn str_to_kind(s: &str) -> Option<DomEventKind> {
		match s {
			"pointermove" => Some(DomEventKind::PointerMove),
			"pointerdown" => Some(DomEventKind::PointerDown),
			"pointerup" => Some(DomEventKind::PointerUp),
			"pointerenter" => Some(DomEventKind::PointerEnter),
			"pointerleave" => Some(DomEventKind::PointerLeave),
			"pointerover" => Some(DomEventKind::PointerOver),
			"pointerout" => Some(DomEventKind::PointerOut),

			"mousemove" => Some(DomEventKind::MouseMove),
			"mousedown" => Some(DomEventKind::MouseDown),
			"mouseup" => Some(DomEventKind::MouseUp),
			"mouseenter" => Some(DomEventKind::MouseEnter),
			"mouseleave" => Some(DomEventKind::MouseLeave),
			"mouseover" => Some(DomEventKind::MouseOver),
			"mouseout" => Some(DomEventKind::MouseOut),

			"scroll" => Some(DomEventKind::Scroll),
			"wheel" => Some(DomEventKind::Wheel),

			"click" => Some(DomEventKind::Click),
			"contextmenu" => Some(DomEventKind::ContextMenu),
			"dblclick" => Some(DomEventKind::DoubleClick),

			"keypress" => Some(DomEventKind::KeyPress),
			"keydown" => Some(DomEventKind::KeyDown),
			"keyup" => Some(DomEventKind::KeyUp),
			"input" => Some(DomEventKind::Input),
			"composition" => Some(DomEventKind::Ime),

			"focus" => Some(DomEventKind::Focus),
			"blur" => Some(DomEventKind::Blur),
			"focusin" => Some(DomEventKind::FocusIn),
			"focusout" => Some(DomEventKind::FocusOut),
			_ => None,
		}
	}
	fn kind_to_str(k: DomEventKind) -> &'static str {
		match k {
			DomEventKind::PointerMove => "pointermove",
			DomEventKind::PointerDown => "pointerdown",
			DomEventKind::PointerUp => "pointerup",
			DomEventKind::PointerEnter => "pointerenter",
			DomEventKind::PointerLeave => "pointerleave",
			DomEventKind::PointerOver => "pointerover",
			DomEventKind::PointerOut => "pointerout",

			DomEventKind::MouseMove => "mousemove",
			DomEventKind::MouseDown => "mousedown",
			DomEventKind::MouseUp => "mouseup",
			DomEventKind::MouseEnter => "mouseenter",
			DomEventKind::MouseLeave => "mouseleave",
			DomEventKind::MouseOver => "mouseover",
			DomEventKind::MouseOut => "mouseout",

			DomEventKind::Scroll => "scroll",
			DomEventKind::Wheel => "wheel",

			DomEventKind::Click => "click",
			DomEventKind::ContextMenu => "contextmenu",
			DomEventKind::DoubleClick => "dblclick",

			DomEventKind::KeyPress => "keypress",
			DomEventKind::KeyDown => "keydown",
			DomEventKind::KeyUp => "keyup",
			DomEventKind::Input => "input",
			DomEventKind::Ime => "composition",

			DomEventKind::Focus => "focus",
			DomEventKind::Blur => "blur",
			DomEventKind::FocusIn => "focusin",
			DomEventKind::FocusOut => "focusout",
		}
	}
}
impl EventHandler for &mut BlitzEventHandler {
	fn handle_event(
		&mut self,
		chain: &[usize],
		event: &mut blitz_traits::events::DomEvent,
		doc: &mut dyn Document,
		event_state: &mut blitz_traits::events::EventState,
	) {
		let doc: &mut HtmlDocument =
			unsafe { (doc as &mut dyn Any).downcast_mut().unwrap_unchecked() };

		let temp_override_ret = self.temp_override.as_ref().map(|func| {
			(
				func,
				func.call1(&JsValue::NULL, &BlitzDocument::unsafe_with_ref(doc).into())
					.unwrap(),
			)
		});

		'a: for node_id in chain {
			if let Some(listeners) = self
				.listeners
				.get(&(*node_id, event.data.kind().discriminant()))
			{
				for listener in listeners {
					let kind = BlitzEventHandler::kind_to_str(event.data.kind());
					match JsEvent::new(kind) {
						Ok(event) => {
							if let Err(err) = listener.call1(&JsValue::NULL, &event.clone().into())
							{
								console::warn_3(
									&"error while calling event listener for ".into(),
									&kind.into(),
									&err,
								);
								continue;
							}

							if event.cancel_bubble() {
								event_state.stop_propagation();
							}
							if event.default_prevented() {
								event_state.prevent_default();
								break 'a;
							}
						}
						Err(err) => {
							console::warn_3(
								&"failed to instantiate js event for".into(),
								&kind.into(),
								&err,
							);
						}
					}
				}
			}
		}

		if let Some((func, ret)) = temp_override_ret {
			func.call1(&JsValue::NULL, &ret).unwrap();
		}
	}
}

#[wasm_bindgen]
pub struct BlitzRendererEvent(UiEvent);

enum BlitzDocumentInner {
	Owned(HtmlDocument),
	Ref(&'static mut HtmlDocument),
}
impl Deref for BlitzDocumentInner {
	type Target = HtmlDocument;
	fn deref(&self) -> &Self::Target {
		match self {
			Self::Owned(x) => x,
			Self::Ref(x) => x,
		}
	}
}
impl DerefMut for BlitzDocumentInner {
	fn deref_mut(&mut self) -> &mut Self::Target {
		match self {
			Self::Owned(x) => x,
			Self::Ref(x) => x,
		}
	}
}

#[wasm_bindgen]
pub struct BlitzDocument(BlitzDocumentInner);

impl BlitzDocument {
	pub fn new(doc: HtmlDocument) -> Self {
		Self(BlitzDocumentInner::Owned(doc))
	}
	pub fn unsafe_with_ref(doc: &mut HtmlDocument) -> Self {
		Self(BlitzDocumentInner::Ref(unsafe { transmute(doc) }))
	}

	pub fn doc(&self) -> &HtmlDocument {
		&self.0
	}
	pub fn node(&self, node: &BlitzNode) -> Result<&Node, JsError> {
		self.0
			.get_node(node.0)
			.ok_or_else(|| JsError::new("invalid node"))
	}

	pub fn viewport(&mut self) -> impl DerefMut<Target = Viewport> {
		self.0.viewport_mut()
	}

	pub fn resolve(&mut self, time: f64) {
		self.0.resolve(time);
	}

	pub fn mutator(&mut self) -> DocumentMutator<'_> {
		DocumentMutator::new(&mut self.0)
	}
}

#[wasm_bindgen]
impl BlitzDocument {
	pub fn root(&self) -> BlitzNode {
		self.0.root_node().into()
	}

	pub fn query_selector(&self, selector: &str) -> Result<Option<BlitzNode>, JsError> {
		self.0
			.query_selector(selector)
			.map(|x| x.map(BlitzNode))
			.map_err(|_| JsError::new("selector failed to parse"))
	}

	pub fn event(&mut self, events: &mut BlitzEventHandler, event: BlitzRendererEvent) {
		let mut handler = EventDriver::new(self.0.deref_mut(), events);
		handler.handle_ui_event(event.0);
	}

	pub fn event_pointer(
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

	pub fn event_wheel(
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
			0 => blitz_traits::events::BlitzWheelDelta::Pixels(
				-web_event.delta_x(),
				-web_event.delta_y(),
			),
			1 => blitz_traits::events::BlitzWheelDelta::Lines(
				-web_event.delta_x(),
				-web_event.delta_y(),
			),
			_ => blitz_traits::events::BlitzWheelDelta::Pixels(
				-web_event.delta_x(),
				-web_event.delta_y(),
			),
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

	pub fn event_keyboard(web_event: &KeyboardEvent) -> Result<BlitzRendererEvent, JsError> {
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
