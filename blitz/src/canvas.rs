use anyhow::{Context, Result};
use vello::{
	AaConfig, AaSupport, RenderParams, Renderer, RendererOptions, Scene, peniko::color::palette,
};
use web_sys::OffscreenCanvas;
use wgpu::{
	Device, DeviceDescriptor, InstanceDescriptor, PowerPreference, Queue, RequestAdapterOptions,
	Surface, SurfaceConfiguration, SurfaceTarget, TextureFormat, TextureUsages,
	TextureViewDescriptor,
};

pub struct CanvasVelloScene {
	device: Device,
	queue: Queue,
	renderer: Renderer,
	surface: Surface<'static>,
	scene: Scene,
	width: u32,
	height: u32,
	scale: f32,
}
impl CanvasVelloScene {
	pub async fn new(canvas: OffscreenCanvas, scale: f32) -> Result<CanvasVelloScene> {
		let width = canvas.width();
		let height = canvas.height();

		let instance = wgpu::Instance::new(&InstanceDescriptor::default());
		let surface = instance
			.create_surface(SurfaceTarget::OffscreenCanvas(canvas))
			.context("failed to create surface")?;
		let adapter = instance
			.request_adapter(&RequestAdapterOptions {
				power_preference: PowerPreference::None,
				compatible_surface: Some(&surface),
				force_fallback_adapter: false,
			})
			.await
			.context("failed to request adapter")?;

		let (device, queue) = adapter
			.request_device(&DeviceDescriptor::default())
			.await
			.context("failed to request device")?;

		let surface_config = SurfaceConfiguration {
			usage: TextureUsages::STORAGE_BINDING,
			width,
			height,
			format: TextureFormat::Rgba8Unorm,
			view_formats: vec![TextureFormat::Rgba8UnormSrgb],
			alpha_mode: wgpu::CompositeAlphaMode::Auto,
			desired_maximum_frame_latency: 2,
			present_mode: wgpu::PresentMode::AutoVsync,
		};
		surface.configure(&device, &surface_config);

		let renderer = Renderer::new(
			&device,
			RendererOptions {
				use_cpu: false,
				antialiasing_support: AaSupport::all(),
				num_init_threads: None,
				pipeline_cache: None,
			},
		)
		.context("failed to create renderer")?;

		Ok(Self {
			device,
			queue,
			surface,
			renderer,
			scene: Scene::new(),
			width,
			height,
			scale,
		})
	}

	pub fn render(&mut self, func: impl Fn(&mut Scene, u32, u32, f32)) -> anyhow::Result<()> {
		let texture = self
			.surface
			.get_current_texture()
			.context("failed to get canvas texture")?;
		let view = texture
			.texture
			.create_view(&TextureViewDescriptor::default());

		let params = RenderParams {
			antialiasing_method: AaConfig::Msaa16,
			base_color: palette::css::WHITE,
			width: self.width,
			height: self.height,
		};

		func(&mut self.scene, self.width, self.height, self.scale);

		self.renderer
			.render_to_texture(&self.device, &self.queue, &self.scene, &view, &params)
			.context("failed to render")?;

		self.scene.reset();

		texture.present();

		Ok(())
	}
}
