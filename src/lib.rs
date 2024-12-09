use std::hash::Hash;

use winit::{
    event::*,
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::WindowBuilder,
};

use winit::window::Window;

struct State<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,

    render_pipeline: wgpu::RenderPipeline,
    clear_color: wgpu::Color,
    // last
    window: &'a Window,
}

impl<'a> State<'a> {
    async fn new(window: &'a Window) -> State<'a> {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        // where we draw to.
        let surface = instance.create_surface(window).unwrap();

        // adapter = a handle to the actual graphics card
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    label: None,
                    memory_hints: Default::default(),
                },
                None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT, // describes gow SurfaceTexture`s will be used
            format: surface_format,                        // how it is stored in gpu memory
            width: size.width,
            height: size.height,
            // PresentMode::Fifo -> essentialy VSync
            present_mode: surface_caps.present_modes[0], // determines how to sync the surface with the display
            alpha_mode: surface_caps.alpha_modes[0], // something to do with transparent windows (dunno)
            view_formats: vec![], // list of TextureFormat`s that u can use when creating TextureViews`s
            desired_maximum_frame_latency: 2,
        };

        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));
        // - long version
        // let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        //     label: Some("Shader"),
        //     source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        // });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main", // 1.these are the function we marked as @vertex and @fragment.
                buffers: &[], // 2.The buffers field tells wgpu what type of vertices we want to pass to the vertex shader.
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            // 3.The fragment is technically optional, so you have to wrap it in Some(). We need it if we want to store color data to the surface.
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                // 4. The targets field tells wgpu what color outputs it should set up
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            // The primitive field describes how to interpret our vertices when converting them into triangles.
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList, // 1. Using PrimitiveTopology::TriangleList means that every three vertices will correspond to one triangle
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw, // 2. FrontFace::Ccw means that a triangle is facing forward if the vertices are arranged in a counter-clockwise direction
                cull_mode: Some(wgpu::Face::Back), // Triangles that are not considered facing forward are culled (not included in the render).
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None, // 1. We're not using a depth/stencil buffer currently, so we leave depth_stencil as None.
            multisample: wgpu::MultisampleState {
                count: 1,                         // 2. determines how many samples the pipeline will use
                mask: !0, // 3. specifies which samples should be active. In this case, we are using all of them.
                alpha_to_coverage_enabled: false, // 4. has to do with anti-aliasing
            },
            multiview: None, // 5. indicates how many array layers the render attachments can have
            cache: None, // 6. allows wgpu to cache shader compilation data. Only really useful for Android build targets.
        });

        Self {
            window,
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            clear_color: wgpu::Color::BLACK,
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.clear_color = wgpu::Color {
                    r: position.x as f64 / self.size.width as f64,
                    g: position.y as f64 / self.size.height as f64,
                    b: 1.0,
                    a: 1.0,
                }
            }
            _ => {
                // print!("something")
            }
        }
        false
    }

    fn update(&mut self) {
        // todo!()
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        // get_current_texture function will wait for the surface to provide a new SurfaceTexture that we will render to.
        let output = self.surface.get_current_texture()?;

        // This line creates a TextureView with default settings.
        // We need to do this because we want to control how the render code interacts with the texture.
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Command Encoder -> create the actual command to send to the GPU.
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        // we create a new Block because we need to borrow mut encoder
        // and we cannot call encode.finish() untill we release that mutable borrow.
        // can also use drop(render_pass) to achieve the same effect
        {
            //  We need to use the encoder to create a RenderPass -> to clear the screen
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[
                    // This is what @location(0) in the fragment shader targets//
                    Some(wgpu::RenderPassColorAttachment {
                        view: &view,          // Informs what texture to save the colors to
                        resolve_target: None, // is the texture that will receive the resolved output -> same as view(dont need) unless multisampling
                        ops: wgpu::Operations {
                            // wgpu::Operations -> tells whgpu what to do with ther colors on the screen (specified by view)
                            load: wgpu::LoadOp::Clear(self.clear_color),
                            store: wgpu::StoreOp::Store,
                        },
                    }),
                ],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            render_pass.set_pipeline(&self.render_pipeline); // 2.
            render_pass.draw(0..3, 0..1); // 3.
        }

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(()) //
    }
}

pub async fn run() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut state = State::new(&window).await;

    event_loop
        .run(move |event, control_flow| match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.window().id() => {
                if !state.input(event) {
                    match event {
                        WindowEvent::CloseRequested
                        | WindowEvent::KeyboardInput {
                            event:
                                KeyEvent {
                                    state: ElementState::Pressed,
                                    physical_key: PhysicalKey::Code(KeyCode::Escape),
                                    ..
                                },
                            ..
                        } => control_flow.exit(),
                        WindowEvent::Resized(physical_size) => {
                            print!("resize");
                            state.resize(*physical_size);
                        }
                        WindowEvent::RedrawRequested => {
                            state.window().request_redraw();

                            // if !surface_configured {
                            //     return;
                            // }

                            state.update();
                            match state.render() {
                                Ok(_) => {}
                                // Reconfigure the surface if it's lost or outdated
                                Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                                    state.resize(state.size)
                                }
                                // The system is out of memory, we should probably quit
                                Err(wgpu::SurfaceError::OutOfMemory) => {
                                    log::error!("OutOfMemory");
                                    control_flow.exit();
                                }

                                // This happens when the a frame takes too long to present
                                Err(wgpu::SurfaceError::Timeout) => {
                                    log::warn!("Surface timeout")
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        })
        .ok();
}
