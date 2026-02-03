use anyrender::{NormalizedCoord, Paint, PaintRef, PaintScene};
use vello::{
	kurbo::{Affine, Rect, Shape, Stroke},
	peniko::{BlendMode, BrushRef, Color, Fill, FontData, StyleRef},
};

pub struct VelloScenePainter<'s> {
	pub(crate) inner: &'s mut vello::Scene,
}

impl VelloScenePainter<'_> {
	pub fn new<'s>(scene: &'s mut vello::Scene) -> VelloScenePainter<'s> {
		VelloScenePainter { inner: scene }
	}
}

impl PaintScene for VelloScenePainter<'_> {
	fn reset(&mut self) {
		self.inner.reset();
	}

	fn push_layer(
		&mut self,
		blend: impl Into<BlendMode>,
		alpha: f32,
		transform: Affine,
		clip: &impl Shape,
	) {
		self.inner
			.push_layer(Fill::NonZero, blend, alpha, transform, clip);
	}

	fn push_clip_layer(&mut self, transform: Affine, clip: &impl Shape) {
		self.inner.push_clip_layer(Fill::NonZero, transform, clip);
	}

	fn pop_layer(&mut self) {
		self.inner.pop_layer();
	}

	fn stroke<'a>(
		&mut self,
		style: &Stroke,
		transform: Affine,
		paint_ref: impl Into<PaintRef<'a>>,
		brush_transform: Option<Affine>,
		shape: &impl Shape,
	) {
		let paint_ref: PaintRef<'_> = paint_ref.into();
		let brush_ref: BrushRef<'_> = paint_ref.into();
		self.inner
			.stroke(style, transform, brush_ref, brush_transform, shape);
	}

	fn fill<'a>(
		&mut self,
		style: Fill,
		transform: Affine,
		paint: impl Into<PaintRef<'a>>,
		brush_transform: Option<Affine>,
		shape: &impl Shape,
	) {
		let paint: PaintRef<'_> = paint.into();

		let brush_ref: BrushRef<'_> = match paint {
			Paint::Solid(color) => BrushRef::Solid(color),
			Paint::Gradient(gradient) => BrushRef::Gradient(gradient),
			Paint::Image(image) => BrushRef::Image(image),
			_ => return,
		};

		self.inner
			.fill(style, transform, brush_ref, brush_transform, shape);
	}

	fn draw_glyphs<'a, 's: 'a>(
		&'a mut self,
		font: &'a FontData,
		font_size: f32,
		hint: bool,
		normalized_coords: &'a [NormalizedCoord],
		style: impl Into<StyleRef<'a>>,
		paint: impl Into<PaintRef<'a>>,
		brush_alpha: f32,
		transform: Affine,
		glyph_transform: Option<Affine>,
		glyphs: impl Iterator<Item = anyrender::Glyph>,
	) {
		self.inner
			.draw_glyphs(font)
			.font_size(font_size)
			.hint(hint)
			.normalized_coords(normalized_coords)
			.brush(paint.into())
			.brush_alpha(brush_alpha)
			.transform(transform)
			.glyph_transform(glyph_transform)
			.draw(
				style,
				glyphs.map(|g: anyrender::Glyph| vello::Glyph {
					id: g.id,
					x: g.x,
					y: g.y,
				}),
			);
	}

	fn draw_box_shadow(
		&mut self,
		transform: Affine,
		rect: Rect,
		brush: Color,
		radius: f64,
		std_dev: f64,
	) {
		self.inner
			.draw_blurred_rounded_rect(transform, rect, brush, radius, std_dev);
	}
}
