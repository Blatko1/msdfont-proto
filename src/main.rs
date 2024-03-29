mod camera;
mod text;
mod util;

use std::time::{Duration, Instant};

use camera::Camera;
use pollster::block_on;
use text::Text;
use util::Requisites;
use winit::{
    dpi::PhysicalSize,
    event::{ElementState, KeyboardInput, VirtualKeyCode},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

const FPS_CAP: f64 = 60.0;

fn main() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "error");
    }
    env_logger::init();

    ///////////////////////////////// Create Window ///////////////////////////
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("SDF Text Renderer Prototype")
        .build(&event_loop)
        .unwrap();

    /////////// Initialize GPU, MSDF font and other preparations. ///////////
    let mut gfx = block_on(Graphics::new(&window)).unwrap();
    let arfont = artery_font::ArteryFont::read(
        &include_bytes!("../fonts/font.arfont")[..],
    )
    .unwrap();
    let mut camera = Camera::new(&gfx);
    let reqs = Requisites::init(&gfx, &arfont);

    let pipeline1 = util::pipeline1(&gfx, &reqs);
    let pipeline2 = util::pipeline2(&gfx, &reqs);
    let pipeline3 = util::pipeline3(&gfx, &reqs);
    let pipeline4 = util::pipeline4(&gfx, &reqs);

    let text1 = Text::new("TEST Aabcdefghijklmnoprstuvz", (0.0, 1.5, 0.0));
    let (vertex_buffer1, vertices1) = text1.create_buffer(&gfx, &reqs.glyphs);
    let text2 = Text::new("TEST Aabcdefghijklmnoprstuvz", (0.0, 0.0, 0.0));
    let (vertex_buffer2, vertices2) = text2.create_buffer(&gfx, &reqs.glyphs);
    let text3 = Text::new("TEST Aabcdefghijklmnoprstuvz", (0.0, -1.5, 0.0));
    let (vertex_buffer3, vertices3) = text3.create_buffer(&gfx, &reqs.glyphs);
    let text4 = Text::new("TEST Aabcdefghijklmnoprstuvz", (0.0, -3.0, 0.0));
    let (vertex_buffer4, vertices4) = text4.create_buffer(&gfx, &reqs.glyphs);

    /////////////////////////////// LOOP ///////////////////////////////////////
    let target_framerate = Duration::from_secs_f64(1.0 / FPS_CAP);
    let mut time = Instant::now();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            winit::event::Event::WindowEvent { event, .. } => match event {
                winit::event::WindowEvent::Resized(inner_size)
                | winit::event::WindowEvent::ScaleFactorChanged {
                    new_inner_size: &mut inner_size,
                    ..
                } => {
                    gfx.resize(inner_size);
                    camera.resize(&gfx);
                }
                winit::event::WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit
                }
                winit::event::WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        },
                    ..
                } => *control_flow = ControlFlow::Exit,
                _ => (),
            },
            winit::event::Event::DeviceEvent { event, .. } => {
                camera.input(&event)
            }
            winit::event::Event::MainEventsCleared => {
                let elapsed = time.elapsed();
                if elapsed >= target_framerate {
                    window.request_redraw();
                    time = Instant::now();
                } else {
                    *control_flow = ControlFlow::WaitUntil(
                        Instant::now() + target_framerate - elapsed,
                    );
                }
            }
            winit::event::Event::RedrawRequested(_) => {
                // UPDATE
                camera.update();

                let mat: &[[f32; 4]; 4] = &camera.update_global_matrix().into();
                gfx.queue.write_buffer(
                    &reqs.matrix_buffer,
                    0,
                    bytemuck::cast_slice(mat),
                );

                // RENDER
                let frame = gfx.surface.get_current_texture().unwrap();
                let view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                let mut encoder = gfx.device.create_command_encoder(
                    &wgpu::CommandEncoderDescriptor {
                        label: Some("Command Encoder"),
                    },
                );

                {
                    let mut rpass = encoder.begin_render_pass(
                        &wgpu::RenderPassDescriptor {
                            label: Some("Render Pass"),
                            color_attachments: &[Some(
                                wgpu::RenderPassColorAttachment {
                                    view: &view,
                                    resolve_target: None,
                                    ops: wgpu::Operations {
                                        load: wgpu::LoadOp::Clear(
                                            wgpu::Color {
                                                r: 0.1,
                                                g: 0.2,
                                                b: 0.3,
                                                a: 1.,
                                            },
                                        ),
                                        store: true,
                                    },
                                },
                            )],
                            depth_stencil_attachment: None,
                        },
                    );

                    // Testing method 2
                    rpass.set_pipeline(&pipeline1);

                    rpass.set_vertex_buffer(0, vertex_buffer1.slice(..));
                    rpass.set_bind_group(0, &reqs.bind_group, &[]);
                    
                    rpass.draw(0..4, 0..vertices1);

                    // Testing method 1
                    rpass.set_pipeline(&pipeline2);

                    rpass.set_vertex_buffer(0, vertex_buffer2.slice(..));
                    rpass.set_bind_group(0, &reqs.bind_group, &[]);
                    
                    rpass.draw(0..4, 0..vertices2);

                    // Cheap method
                    rpass.set_pipeline(&pipeline3);

                    rpass.set_vertex_buffer(0, vertex_buffer3.slice(..));
                    rpass.set_bind_group(0, &reqs.bind_group, &[]);

                    rpass.draw(0..4, 0..vertices3);

                    // Best method
                    rpass.set_pipeline(&pipeline4);

                    rpass.set_vertex_buffer(0, vertex_buffer4.slice(..));
                    rpass.set_bind_group(0, &reqs.bind_group, &[]);

                    rpass.draw(0..4, 0..vertices4);

                    // Lines
                    //rpass.set_pipeline(&line_pipeline);

                    //rpass.set_vertex_buffer(0, line_vertex_buffer.slice(..));
                    //rpass.set_bind_group(0, &reqs.bind_group, &[]);

                    //rpass.draw(0..line_vertices, 0..1);

                }

                gfx.queue.submit(Some(encoder.finish()));
                frame.present();
            }
            _ => (),
        }
    });
}
pub struct Graphics {
    pub device: wgpu::Device,
    pub surface: wgpu::Surface,
    pub adapter: wgpu::Adapter,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
}

impl Graphics {
    async fn new(
        window: &winit::window::Window,
    ) -> Result<Self, wgpu::RequestDeviceError> {
        let backends = wgpu::util::backend_bits_from_env()
            .unwrap_or_else(wgpu::Backends::all);
        let instance = wgpu::Instance::new(backends);
        let (size, surface) = unsafe {
            let size = window.inner_size();
            let surface = instance.create_surface(&window);
            (size, surface)
        };

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .expect("No adapters found!");

        let adapter_info = adapter.get_info();
        println!(
            "Adapter info: Name: {}, backend: {:?}, device: {:?}",
            adapter_info.name, adapter_info.backend, adapter_info.device_type
        );

        let required_features = wgpu::Features::empty();
        let adapter_features = adapter.features();
        let required_limits = wgpu::Limits::default();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Device"),
                    features: adapter_features & required_features,
                    limits: required_limits,
                },
                None,
            )
            .await?;

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
        };
        surface.configure(&device, &config);

        Ok(Self {
            device,
            surface,
            adapter,
            queue,
            config,
        })
    }

    fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.config.width = new_size.width.max(1);
        self.config.height = new_size.height.max(1);
        self.surface.configure(&self.device, &self.config);
    }
}
