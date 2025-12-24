use super::*;

macro_rules! label {
  () => {
    Some(concat!(file!(), ":", line!(), ":", column!()))
  };
}

pub(crate) struct Renderer {
  capture_thread: CaptureThread,
  clamp_to_border_sampler: Sampler,
  composite_pipeline: Pipeline,
  device: wgpu::Device,
  error_channel: mpsc::Receiver<wgpu::Error>,
  field_texture_bind_group_layout: BindGroupLayout,
  filter_pipeline: Pipeline,
  filtering_sampler: Sampler,
  font_context: FontContext,
  format: ImageFormat,
  frame: u64,
  frequencies: TextureView,
  layout_context: LayoutContext,
  limits: Limits,
  mirroring_sampler: Sampler,
  non_filtering_sampler: Sampler,
  queue: Queue,
  resolution: NonZeroU32,
  resources: Option<Resources>,
  samples: TextureView,
  size: Size,
  surface: Option<(Surface<'static>, SurfaceConfiguration)>,
  vello_renderer: vello::Renderer,
  vello_scene: vello::Scene,
}

impl Renderer {
  const COMPOSITE_UNIFORMS: usize = 3;

  const FONT_STACK: FontStack<'static> = FontStack::List(Cow::Borrowed(&[
    FontFamily::Named(Cow::Borrowed("Helvetica Neue")),
    FontFamily::Generic(GenericFamily::SansSerif),
    FontFamily::Named(Cow::Borrowed("Apple Symbols")),
    FontFamily::Named(Cow::Borrowed("Zapf Dingbats")),
    FontFamily::Named(Cow::Borrowed("Last Resort")),
  ]));

  const IMAGE_SUBRESOURCE_RANGE_FULL: ImageSubresourceRange = ImageSubresourceRange {
    array_layer_count: None,
    aspect: TextureAspect::All,
    base_array_layer: 0,
    base_mip_level: 0,
    mip_level_count: None,
  };

  fn begin_render_pass<'a>(encoder: &'a mut CommandEncoder, view: &TextureView) -> RenderPass<'a> {
    encoder.begin_render_pass(&RenderPassDescriptor {
      color_attachments: &[Some(RenderPassColorAttachment {
        depth_slice: None,
        ops: Operations {
          load: LoadOp::Load,
          store: StoreOp::Store,
        },
        resolve_target: None,
        view,
      })],
      depth_stencil_attachment: None,
      label: label!(),
      occlusion_query_set: None,
      timestamp_writes: None,
    })
  }

  fn bytes_per_row_with_padding(&self) -> u32 {
    const MASK: u32 = COPY_BYTES_PER_ROW_ALIGNMENT - 1;
    (self.resolution.get() * u32::try_from(COLOR_CHANNELS).unwrap() + MASK) & !MASK
  }

  pub(crate) fn capture(&self, callback: impl FnOnce(Image) + Send + 'static) -> Result {
    let bytes_per_row_with_padding = self.bytes_per_row_with_padding();

    let mut encoder = self
      .device
      .create_command_encoder(&CommandEncoderDescriptor::default());

    let pool = self.resources().pool.clone();

    let buffer = pool.lock().unwrap().pop().unwrap_or_else(|| {
      log::info!("creating new capture buffer");
      self.device.create_buffer(&BufferDescriptor {
        label: label!(),
        mapped_at_creation: false,
        size: (self.bytes_per_row_with_padding() * self.resolution.get()).into(),
        usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
      })
    });

    encoder.copy_texture_to_buffer(
      TexelCopyTextureInfo {
        texture: self.resources().targets[0].texture_view.texture(),
        mip_level: 0,
        origin: Origin3d::ZERO,
        aspect: TextureAspect::All,
      },
      TexelCopyBufferInfo {
        buffer: &buffer,
        layout: TexelCopyBufferLayout {
          bytes_per_row: Some(bytes_per_row_with_padding),
          rows_per_image: None,
          offset: 0,
        },
      },
      Extent3d {
        width: self.resolution.get(),
        height: self.resolution.get(),
        depth_or_array_layers: 1,
      },
    );

    self.queue.submit([encoder.finish()]);

    let capture = Capture {
      buffer: buffer.clone(),
      bytes_per_row_with_padding: bytes_per_row_with_padding.into_usize(),
      callback: Box::new(callback),
      pool,
      format: self.format,
      size: self.size,
    };

    let tx = self.capture_thread.tx().clone();
    buffer.map_async(MapMode::Read, .., move |result| {
      if let Err(err) = result {
        eprintln!("failed to map capture buffer: {err}");
        return;
      }
      tx.send(capture).ok();
    });

    Ok(())
  }

  fn clamp_resolution(limits: &Limits, resolution: NonZeroU32) -> NonZeroU32 {
    resolution
      .min(limits.max_texture_dimension_2d.try_into().unwrap())
      .min(5808.try_into().unwrap())
  }

  fn composite_bind_group(
    &self,
    destination: &TextureView,
    destination_sampler: &Sampler,
    source: &TextureView,
    source_sampler: &Sampler,
  ) -> BindGroup {
    let mut binding = Counter::default();
    self.device.create_bind_group(&BindGroupDescriptor {
      layout: &self.composite_pipeline.bind_group_layout,
      entries: &[
        BindGroupEntry {
          binding: binding.next(),
          resource: BindingResource::TextureView(destination),
        },
        BindGroupEntry {
          binding: binding.next(),
          resource: BindingResource::Sampler(destination_sampler),
        },
        BindGroupEntry {
          binding: binding.next(),
          resource: BindingResource::TextureView(source),
        },
        BindGroupEntry {
          binding: binding.next(),
          resource: BindingResource::Sampler(source_sampler),
        },
        BindGroupEntry {
          binding: binding.next(),
          resource: BindingResource::Buffer(BufferBinding {
            buffer: &self.composite_pipeline.uniform_buffer,
            offset: 0,
            size: Some(
              u64::from(self.composite_pipeline.uniform_buffer_size)
                .try_into()
                .unwrap(),
            ),
          }),
        },
      ],
      label: label!(),
    })
  }

  fn composite_bind_group_layout(
    device: &wgpu::Device,
    uniform_buffer_size: u32,
  ) -> BindGroupLayout {
    let mut binding = Counter::default();
    device.create_bind_group_layout(&BindGroupLayoutDescriptor {
      entries: &[
        BindGroupLayoutEntry {
          binding: binding.next(),
          count: None,
          ty: BindingType::Texture {
            multisampled: false,
            sample_type: TextureSampleType::Float { filterable: true },
            view_dimension: TextureViewDimension::D2,
          },
          visibility: ShaderStages::FRAGMENT,
        },
        BindGroupLayoutEntry {
          binding: binding.next(),
          count: None,
          ty: BindingType::Sampler(SamplerBindingType::NonFiltering),
          visibility: ShaderStages::FRAGMENT,
        },
        BindGroupLayoutEntry {
          binding: binding.next(),
          count: None,
          ty: BindingType::Texture {
            multisampled: false,
            sample_type: TextureSampleType::Float { filterable: true },
            view_dimension: TextureViewDimension::D2,
          },
          visibility: ShaderStages::FRAGMENT,
        },
        BindGroupLayoutEntry {
          binding: binding.next(),
          count: None,
          ty: BindingType::Sampler(SamplerBindingType::NonFiltering),
          visibility: ShaderStages::FRAGMENT,
        },
        BindGroupLayoutEntry {
          binding: binding.next(),
          count: None,
          ty: BindingType::Buffer {
            has_dynamic_offset: true,
            min_binding_size: Some(u64::from(uniform_buffer_size).try_into().unwrap()),
            ty: BufferBindingType::Uniform,
          },
          visibility: ShaderStages::FRAGMENT,
        },
      ],
      label: label!(),
    })
  }

  fn convert_glyph(glyph: parley::Glyph) -> vello::Glyph {
    vello::Glyph {
      id: glyph.id,
      x: glyph.x,
      y: glyph.y,
    }
  }

  fn create_render_pipeline(
    device: &wgpu::Device,
    pipeline_layout: &PipelineLayout,
    format: ImageFormat,
    vertex: &str,
    fragment: &str,
  ) -> RenderPipeline {
    let vertex = device.create_shader_module(ShaderModuleDescriptor {
      label: label!(),
      source: ShaderSource::Wgsl(vertex.into()),
    });

    let fragment = device.create_shader_module(ShaderModuleDescriptor {
      label: label!(),
      source: ShaderSource::Wgsl(fragment.into()),
    });

    device.create_render_pipeline(&RenderPipelineDescriptor {
      cache: None,
      depth_stencil: None,
      fragment: Some(FragmentState {
        compilation_options: PipelineCompilationOptions::default(),
        entry_point: Some("fragment"),
        module: &fragment,
        targets: &[Some(TextureFormat::from(format).into())],
      }),
      label: label!(),
      layout: Some(pipeline_layout),
      multisample: MultisampleState::default(),
      multiview: None,
      primitive: PrimitiveState::default(),
      vertex: VertexState {
        buffers: &[],
        compilation_options: PipelineCompilationOptions::default(),
        entry_point: Some("vertex"),
        module: &vertex,
      },
    })
  }

  pub(crate) fn create_vello_texture(&self, size: NonZeroU32) -> TextureView {
    self
      .device
      .create_texture(&TextureDescriptor {
        dimension: TextureDimension::D2,
        format: TextureFormat::Rgba8Unorm,
        label: label!(),
        mip_level_count: 1,
        sample_count: 1,
        size: Extent3d {
          depth_or_array_layers: 1,
          height: size.get(),
          width: size.get(),
        },
        usage: TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING,
        view_formats: &[TextureFormat::Rgba8Unorm],
      })
      .create_view(&TextureViewDescriptor::default())
  }

  fn draw_composite(
    &self,
    bind_group: &BindGroup,
    encoder: &mut CommandEncoder,
    uniform: u32,
    view: &TextureView,
  ) {
    let mut pass = Self::begin_render_pass(encoder, view);

    pass.set_bind_group(
      0,
      Some(bind_group),
      &[self.composite_pipeline.uniform_buffer_stride * uniform],
    );

    pass.set_pipeline(&self.composite_pipeline.render_pipeline);

    pass.draw(0..3, 0..1);
  }

  fn draw_filter(
    &self,
    bind_group: &BindGroup,
    encoder: &mut CommandEncoder,
    filter: u32,
    field_texture_bind_group: &BindGroup,
    tiling: Tiling,
    view: &TextureView,
  ) {
    let mut pass = Self::begin_render_pass(encoder, view);

    pass.set_bind_group(
      0,
      Some(bind_group),
      &[self.filter_pipeline.uniform_buffer_stride * filter],
    );

    pass.set_bind_group(1, Some(field_texture_bind_group), &[]);

    pass.set_pipeline(&self.filter_pipeline.render_pipeline);

    tiling.set_viewport(&mut pass, filter);

    pass.draw(0..3, 0..1);
  }

  fn field_texture_bind_group(&self, filter: &TextureView) -> BindGroup {
    let mut binding = Counter::default();
    self.device.create_bind_group(&BindGroupDescriptor {
      layout: &self.field_texture_bind_group_layout,
      entries: &[BindGroupEntry {
        binding: binding.next(),
        resource: BindingResource::TextureView(filter),
      }],
      label: label!(),
    })
  }

  fn field_texture_bind_group_layout(device: &wgpu::Device) -> BindGroupLayout {
    let mut binding = Counter::default();
    device.create_bind_group_layout(&BindGroupLayoutDescriptor {
      entries: &[BindGroupLayoutEntry {
        binding: binding.next(),
        count: None,
        ty: BindingType::Texture {
          multisampled: false,
          sample_type: TextureSampleType::Float { filterable: true },
          view_dimension: TextureViewDimension::D2,
        },
        visibility: ShaderStages::FRAGMENT,
      }],
      label: label!(),
    })
  }

  fn filter_bind_group(
    &self,
    frequencies: &TextureView,
    source: &TextureView,
    samples: &TextureView,
  ) -> BindGroup {
    let mut binding = Counter::default();
    self.device.create_bind_group(&BindGroupDescriptor {
      layout: &self.filter_pipeline.bind_group_layout,
      entries: &[
        BindGroupEntry {
          binding: binding.next(),
          resource: BindingResource::Sampler(&self.filtering_sampler),
        },
        BindGroupEntry {
          binding: binding.next(),
          resource: BindingResource::TextureView(frequencies),
        },
        BindGroupEntry {
          binding: binding.next(),
          resource: BindingResource::TextureView(source),
        },
        BindGroupEntry {
          binding: binding.next(),
          resource: BindingResource::Sampler(&self.non_filtering_sampler),
        },
        BindGroupEntry {
          binding: binding.next(),
          resource: BindingResource::TextureView(samples),
        },
        BindGroupEntry {
          binding: binding.next(),
          resource: BindingResource::Buffer(BufferBinding {
            buffer: &self.filter_pipeline.uniform_buffer,
            offset: 0,
            size: Some(
              u64::from(self.filter_pipeline.uniform_buffer_size)
                .try_into()
                .unwrap(),
            ),
          }),
        },
      ],
      label: label!(),
    })
  }

  fn filter_bind_group_layout(device: &wgpu::Device, uniform_buffer_size: u32) -> BindGroupLayout {
    let mut binding = Counter::default();
    device.create_bind_group_layout(&BindGroupLayoutDescriptor {
      entries: &[
        BindGroupLayoutEntry {
          binding: binding.next(),
          count: None,
          ty: BindingType::Sampler(SamplerBindingType::Filtering),
          visibility: ShaderStages::FRAGMENT,
        },
        BindGroupLayoutEntry {
          binding: binding.next(),
          count: None,
          ty: BindingType::Texture {
            multisampled: false,
            sample_type: TextureSampleType::Float { filterable: false },
            view_dimension: TextureViewDimension::D1,
          },
          visibility: ShaderStages::FRAGMENT,
        },
        BindGroupLayoutEntry {
          binding: binding.next(),
          count: None,
          ty: BindingType::Texture {
            multisampled: false,
            sample_type: TextureSampleType::Float { filterable: true },
            view_dimension: TextureViewDimension::D2,
          },
          visibility: ShaderStages::FRAGMENT,
        },
        BindGroupLayoutEntry {
          binding: binding.next(),
          count: None,
          ty: BindingType::Sampler(SamplerBindingType::NonFiltering),
          visibility: ShaderStages::FRAGMENT,
        },
        BindGroupLayoutEntry {
          binding: binding.next(),
          count: None,
          ty: BindingType::Texture {
            multisampled: false,
            sample_type: TextureSampleType::Float { filterable: false },
            view_dimension: TextureViewDimension::D1,
          },
          visibility: ShaderStages::FRAGMENT,
        },
        BindGroupLayoutEntry {
          binding: binding.next(),
          count: None,
          ty: BindingType::Buffer {
            has_dynamic_offset: true,
            min_binding_size: Some(u64::from(uniform_buffer_size).try_into().unwrap()),
            ty: BufferBindingType::Uniform,
          },
          visibility: ShaderStages::FRAGMENT,
        },
      ],
      label: label!(),
    })
  }

  pub(crate) fn finish(self) -> Result {
    self.capture_thread.finish()
  }

  pub(crate) fn frame(&self) -> u64 {
    self.frame
  }

  pub(crate) async fn new(
    format: Option<ImageFormat>,
    present_mode: Option<PresentMode>,
    resolution: NonZeroU32,
    size: Size,
    window: Option<Arc<Window>>,
  ) -> Result<Self> {
    let instance = Instance::default();

    let surface = window
      .map(|window| {
        instance
          .create_surface(window)
          .context(error::CreateSurface)
      })
      .transpose()?;

    let adapter = instance
      .request_adapter(&RequestAdapterOptions {
        power_preference: PowerPreference::default(),
        force_fallback_adapter: false,
        compatible_surface: surface.as_ref(),
      })
      .await
      .context(error::RequestAdapter)?;

    let (device, queue) = adapter
      .request_device(&DeviceDescriptor {
        label: label!(),
        memory_hints: MemoryHints::Performance,
        required_features: Features::ADDRESS_MODE_CLAMP_TO_BORDER | Features::CLEAR_TEXTURE,
        required_limits: Limits::default(),
        trace: Trace::Off,
      })
      .await
      .context(error::RequestDevice)?;

    let format = format.unwrap_or_default();

    let surface = if let Some(surface) = surface {
      let capabilities = surface.get_capabilities(&adapter);

      let mut config = surface
        .get_default_config(&adapter, size.x.get(), size.y.get())
        .context(error::DefaultConfig)?;

      if !capabilities.formats.contains(&format.into()) {
        return Err(error::UnsupportedSurfaceFormat { format }.build());
      }

      config.format = format.into();

      if let Some(present_mode) = present_mode {
        if !capabilities.present_modes.contains(&present_mode.into()) {
          return Err(error::UnsupportedSurfacePresentMode { present_mode }.build());
        }
        config.present_mode = present_mode.into();
      }

      surface.configure(&device, &config);

      Some((surface, config))
    } else {
      None
    };

    let (tx, error_channel) = mpsc::channel();

    device.on_uncaptured_error(Box::new(move |error| tx.send(error).unwrap()));

    let filtering_sampler = device.create_sampler(&SamplerDescriptor {
      address_mode_u: AddressMode::Repeat,
      address_mode_v: AddressMode::Repeat,
      mag_filter: FilterMode::Linear,
      min_filter: FilterMode::Linear,
      ..default()
    });

    let non_filtering_sampler = device.create_sampler(&SamplerDescriptor {
      address_mode_u: AddressMode::Repeat,
      address_mode_v: AddressMode::Repeat,
      mag_filter: FilterMode::Nearest,
      min_filter: FilterMode::Nearest,
      ..default()
    });

    let mirroring_sampler = device.create_sampler(&SamplerDescriptor {
      address_mode_u: AddressMode::MirrorRepeat,
      address_mode_v: AddressMode::MirrorRepeat,
      mag_filter: FilterMode::Nearest,
      min_filter: FilterMode::Nearest,
      ..default()
    });

    let clamp_to_border_sampler = device.create_sampler(&SamplerDescriptor {
      address_mode_u: AddressMode::ClampToBorder,
      address_mode_v: AddressMode::ClampToBorder,
      mag_filter: FilterMode::Nearest,
      min_filter: FilterMode::Nearest,
      ..default()
    });

    let limits = device.limits();

    let uniform_buffer_stride = |uniform_buffer_size| {
      let alignment = limits.min_uniform_buffer_offset_alignment;
      let padding = (alignment - uniform_buffer_size % alignment) % alignment;
      uniform_buffer_size + padding
    };

    let composite_pipeline = {
      let uniform_buffer_size = CompositeUniforms::size();
      let uniform_buffer_stride = uniform_buffer_stride(uniform_buffer_size);

      let bind_group_layout = Self::composite_bind_group_layout(&device, uniform_buffer_size);

      let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        bind_group_layouts: &[&bind_group_layout],
        label: label!(),
        push_constant_ranges: &[],
      });

      let render_pipeline = Self::create_render_pipeline(
        &device,
        &pipeline_layout,
        format,
        &VertexWgsl.to_string(),
        &CompositeWgsl.to_string(),
      );

      let uniform_buffer = device.create_buffer(&BufferDescriptor {
        label: label!(),
        mapped_at_creation: false,
        size: u64::from(uniform_buffer_stride) * Self::COMPOSITE_UNIFORMS.into_u64(),
        usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
      });

      Pipeline {
        bind_group_layout,
        pipeline_layout,
        render_pipeline,
        uniform_buffer,
        uniform_buffer_size,
        uniform_buffer_stride,
      }
    };

    let filter_pipeline = {
      let uniform_buffer = device.create_buffer(&BufferDescriptor {
        label: label!(),
        mapped_at_creation: false,
        size: limits.max_buffer_size,
        usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
      });

      let uniform_buffer_size = FilterUniforms::size();
      let uniform_buffer_stride = uniform_buffer_stride(uniform_buffer_size);

      let bind_group_layout = Self::filter_bind_group_layout(&device, uniform_buffer_size);

      let texture_bind_group_layout = Self::field_texture_bind_group_layout(&device);

      let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        bind_group_layouts: &[&bind_group_layout, &texture_bind_group_layout],
        label: label!(),
        push_constant_ranges: &[],
      });

      let render_pipeline = Self::create_render_pipeline(
        &device,
        &pipeline_layout,
        format,
        &VertexWgsl.to_string(),
        &FilterWgsl.to_string(),
      );

      Pipeline {
        bind_group_layout,
        pipeline_layout,
        render_pipeline,
        uniform_buffer,
        uniform_buffer_size,
        uniform_buffer_stride,
      }
    };

    let samples = device
      .create_texture(&TextureDescriptor {
        dimension: TextureDimension::D1,
        format: TextureFormat::R32Float,
        label: label!(),
        mip_level_count: 1,
        sample_count: 1,
        size: Extent3d {
          depth_or_array_layers: 1,
          height: 1,
          width: limits.max_texture_dimension_1d,
        },
        usage: TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING,
        view_formats: &[TextureFormat::R32Float],
      })
      .create_view(&TextureViewDescriptor::default());

    let frequencies = device
      .create_texture(&TextureDescriptor {
        dimension: TextureDimension::D1,
        format: TextureFormat::R32Float,
        label: label!(),
        mip_level_count: 1,
        sample_count: 1,
        size: Extent3d {
          depth_or_array_layers: 1,
          height: 1,
          width: limits.max_texture_dimension_1d,
        },
        usage: TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING,
        view_formats: &[TextureFormat::R32Float],
      })
      .create_view(&TextureViewDescriptor::default());

    let vello_renderer = vello::Renderer::new(
      &device,
      vello::RendererOptions {
        antialiasing_support: vello::AaSupport::all(),
        num_init_threads: Some(1.try_into().unwrap()),
        pipeline_cache: None,
        use_cpu: false,
      },
    )
    .context(error::CreateVelloRenderer)?;

    let field_texture_bind_group_layout = Self::field_texture_bind_group_layout(&device);

    let mut renderer = Self {
      capture_thread: CaptureThread::new()?,
      clamp_to_border_sampler,
      composite_pipeline,
      device,
      error_channel,
      filter_pipeline,
      field_texture_bind_group_layout,
      filtering_sampler,
      font_context: FontContext::new(),
      format,
      frame: 0,
      frequencies,
      layout_context: LayoutContext::new(),
      limits,
      mirroring_sampler,
      non_filtering_sampler,
      queue,
      resolution,
      resources: None,
      samples,
      size,
      surface,
      vello_renderer,
      vello_scene: vello::Scene::new(),
    };

    renderer.resize(size, resolution);

    Ok(renderer)
  }

  pub(crate) fn poll(&self) -> Result {
    self
      .device
      .poll(wgpu::PollType::Wait)
      .map(|_poll_status| ())
      .context(error::RenderPoll)
  }

  pub(crate) fn reload_shaders(&mut self) -> Result {
    let vertex = VertexWgsl
      .reload_from_path()
      .context(error::ShaderReload {
        path: VertexWgsl::PATH.unwrap(),
      })?
      .to_string();

    let filter = FilterWgsl
      .reload_from_path()
      .context(error::ShaderReload {
        path: FilterWgsl::PATH.unwrap(),
      })?
      .to_string();

    let composite = CompositeWgsl
      .reload_from_path()
      .context(error::ShaderReload {
        path: CompositeWgsl::PATH.unwrap(),
      })?
      .to_string();

    self.filter_pipeline.render_pipeline = Self::create_render_pipeline(
      &self.device,
      &self.filter_pipeline.pipeline_layout,
      self.format,
      &vertex,
      &filter,
    );

    self.composite_pipeline.render_pipeline = Self::create_render_pipeline(
      &self.device,
      &self.composite_pipeline.pipeline_layout,
      self.format,
      &vertex,
      &composite,
    );

    Ok(())
  }

  pub(crate) fn render(&mut self, analyzer: &Analyzer, state: &State, fps: Option<f32>) -> Result {
    let mut errors = Vec::new();

    loop {
      match self.error_channel.try_recv() {
        Ok(err) => errors.push(err),
        Err(mpsc::TryRecvError::Empty) => break,
        Err(mpsc::TryRecvError::Disconnected) => panic!("error channel disconnected"),
      }
    }

    if !errors.is_empty() {
      let error = errors.remove(0);
      return Err(
        error::Render {
          additional: errors,
          error,
        }
        .build(),
      );
    }

    self.render_overlay(state, fps)?;

    for filter in &state.filters {
      self.render_field_texture(filter)?;
    }

    let transient = state.transient();

    let filters = state.filters.len() + transient.iter().count();

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let tiling_size = if state.tile {
      (filters.max(1) as f64).sqrt().ceil() as u32
    } else {
      1
    };

    let tiling = Tiling {
      resolution: self.resolution.get() / tiling_size,
      size: tiling_size,
    };

    let sample_count = analyzer
      .samples()
      .len()
      .min(self.samples.texture().width().into_usize());
    let samples = if sample_count == 0 {
      &[0.0]
    } else {
      &analyzer.samples()[..sample_count]
    };
    let sample_range = sample_count as f32 / self.samples.texture().width() as f32;
    self.write_texture(samples, self.samples.texture());

    let frequency_count = analyzer
      .frequencies()
      .len()
      .min(self.frequencies.texture().width().into_usize());
    let frequencies = if frequency_count == 0 {
      &[0.0]
    } else {
      &analyzer.frequencies()[..frequency_count]
    };
    let frequency_range = frequency_count as f32 / self.frequencies.texture().width() as f32;
    self.write_texture(frequencies, self.frequencies.texture());

    let filter_count = u32::try_from(filters).unwrap();

    let gain = 10f32.powf(state.db / 20.0);

    let rms = analyzer.rms();

    {
      let mut uniforms = Vec::new();

      for (i, filter) in state.filters.iter().chain(&transient).enumerate() {
        let i = u32::try_from(i).unwrap();

        let rms = if state.spread {
          rms * (i as f32 + 1.0) / filters as f32
        } else {
          rms
        };

        let rms = rms * filter.rms[0] + filter.rms[1];

        let response = rms / 10.0 * gain;

        uniforms.push(FilterUniforms {
          alpha: filter.alpha,
          base: filter.base,
          color: filter.color_uniform(response),
          coordinates: filter.coordinates,
          destination_offset: tiling.destination_offset(i),
          field: filter.field,
          frequency_range,
          gain,
          grid: filter.grid,
          grid_transform: filter.grid_transform,
          interpolate: state.interpolate,
          mirror: filter.mirror_uniform(),
          parameter: filter.field.parameter(),
          position: filter.position_uniform(response),
          repeat: filter.repeat,
          resolution: tiling.resolution as f32,
          response,
          sample_range,
          source_offset: tiling.source_offset(i),
          tiling: tiling.size,
          wrap: filter.wrap,
        });
      }

      self.write_uniform_buffer(&self.filter_pipeline, &uniforms);
    }

    let aspect_ratio = self.size.x.get() as f32 / self.size.y.get() as f32;

    let aspect_ratio_correction = if state.fit == (aspect_ratio > 1.0) {
      Vec2f::new(1.0 * aspect_ratio, 1.0)
    } else {
      Vec2f::new(1.0, 1.0 / aspect_ratio)
    };

    {
      let aspect_ratio_correction_uniforms = CompositeUniforms {
        destination: true,
        source: true,
        viewport: Mat3f::new_nonuniform_scaling(&Vec2f::new(
          1.0 / self.size.x.get() as f32,
          1.0 / self.size.y.get() as f32,
        ))
        .append_scaling(2.0)
        .append_translation(&Vec2f::new(-1.0, -1.0))
        .append_nonuniform_scaling(&aspect_ratio_correction)
        .append_translation(&Vec2f::new(1.0, 1.0))
        .append_scaling(1.0 / 2.0)
        .to_affine(),
      };

      let uniforms: [CompositeUniforms; Self::COMPOSITE_UNIFORMS] = [
        CompositeUniforms {
          destination: tiling.destination_read(filter_count),
          source: tiling.source_read(filter_count),
          viewport: Mat3f::new_scaling(1.0 / self.resolution.get() as f32).to_affine(),
        },
        CompositeUniforms {
          source: false,
          ..aspect_ratio_correction_uniforms
        },
        aspect_ratio_correction_uniforms,
      ];

      self.write_uniform_buffer(&self.composite_pipeline, &uniforms);
    }

    let mut encoder = self
      .device
      .create_command_encoder(&CommandEncoderDescriptor::default());

    let frame = self
      .surface
      .as_ref()
      .map(|(surface, _config)| surface.get_current_texture().context(error::CurrentTexture))
      .transpose()?;

    for target in &self.resources().targets {
      encoder.clear_texture(
        target.texture_view.texture(),
        &Self::IMAGE_SUBRESOURCE_RANGE_FULL,
      );
    }

    encoder.clear_texture(
      self.resources().tiling_view.texture(),
      &Self::IMAGE_SUBRESOURCE_RANGE_FULL,
    );

    let mut source = 0;
    let mut destination = 1;
    for filter in 0..filters {
      let texture_bind_group =
        if let Some(key) = state.filters.get(filter).and_then(Filter::texture_key) {
          self.resources().field_textures.get(&key).unwrap()
        } else {
          &self.resources().dummy_field_texture
        };

      self.draw_filter(
        &self.resources().targets[source].bind_group,
        &mut encoder,
        filter.try_into().unwrap(),
        texture_bind_group,
        tiling,
        &self.resources().targets[destination].texture_view,
      );
      (source, destination) = (destination, source);
    }

    self.draw_composite(
      &self.resources().tiling_bind_group,
      &mut encoder,
      0,
      &self.resources().tiling_view,
    );

    self.draw_composite(
      &self.resources().overlay_bind_group,
      &mut encoder,
      if state.capture_status { 2 } else { 1 },
      &self.resources().targets[0].texture_view,
    );

    if let Some(frame) = &frame {
      self.draw_composite(
        &self.resources().overlay_bind_group,
        &mut encoder,
        2,
        &frame.texture.create_view(&TextureViewDescriptor::default()),
      );
    }

    self.queue.submit([encoder.finish()]);

    if let Some(frame) = frame {
      frame.present();
    }

    log::trace!(
      "{}",
      Frame {
        filters,
        fps,
        number: self.frame,
      }
    );

    self.frame += 1;

    Ok(())
  }

  pub(crate) fn render_field_texture(&mut self, filter: &Filter) -> Result {
    use {
      kurbo::{Affine, Vec2},
      parley::{Alignment, AlignmentOptions, PositionedLayoutItem, StyleProperty},
      peniko::{Brush, Color, Fill},
      vello::{AaConfig, RenderParams},
    };

    let Some(key) = filter.texture_key() else {
      return Ok(());
    };

    if self.resources().field_textures.contains_key(&key) {
      return Ok(());
    }

    let Field::Texture(texture_field) = filter.field else {
      return Ok(());
    };

    let TextureField {
      text,
      scale,
      weight,
      position,
    } = texture_field;

    self.vello_scene.reset();

    let mut layout = |font_size| {
      let mut builder =
        self
          .layout_context
          .ranged_builder(&mut self.font_context, &text, 1.0, true);
      builder.push_default(StyleProperty::FontSize(font_size));
      builder.push_default(StyleProperty::FontStack(Self::FONT_STACK));
      builder.push_default(StyleProperty::FontWeight(weight));
      let mut layout = builder.build(text);
      layout.break_all_lines(None);
      layout
    };

    let font_size = {
      let layout = layout(1.0);
      self.resolution.get() as f32 / layout.width().max(layout.height()) * scale
    };

    let mut layout = layout(font_size);

    layout.align(
      Some(self.resolution.get() as f32),
      Alignment::Center,
      AlignmentOptions {
        align_when_overflowing: true,
      },
    );

    let offset = {
      let resolution = self.resolution.get() as f64;

      let center = (resolution - layout.height() as f64) * 0.5;

      Vec2 {
        x: resolution * 0.5 * f64::from(position.x),
        y: center + resolution * 0.5 * f64::from(position.y),
      }
    };

    for line in layout.lines() {
      for item in line.items() {
        match item {
          PositionedLayoutItem::GlyphRun(glyph_run) => {
            let run = glyph_run.run();
            self
              .vello_scene
              .draw_glyphs(run.font())
              .brush(&Brush::Solid(Color::WHITE))
              .font_size(font_size)
              .glyph_transform(
                run
                  .synthesis()
                  .skew()
                  .map(|angle| Affine::skew(angle.to_radians().tan().into(), 0.0)),
              )
              .hint(true)
              .normalized_coords(run.normalized_coords())
              .transform(Affine::translate(offset))
              .draw(
                Fill::NonZero,
                glyph_run.positioned_glyphs().map(Self::convert_glyph),
              );
          }
          PositionedLayoutItem::InlineBox(_) => {
            return Err(Error::internal(
              "unexpected inline box while rendering field texture",
            ));
          }
        }
      }
    }

    log::info!("allocating new field texture");

    let view = self.create_vello_texture(self.resolution);

    self
      .vello_renderer
      .render_to_texture(
        &self.device,
        &self.queue,
        &self.vello_scene,
        &view,
        &RenderParams {
          antialiasing_method: AaConfig::Msaa16,
          base_color: Color::TRANSPARENT,
          height: self.resolution.get(),
          width: self.resolution.get(),
        },
      )
      .context(error::RenderVello)?;

    let bind_group = self.field_texture_bind_group(&view);

    self.resources_mut().field_textures.insert(key, bind_group);

    Ok(())
  }

  pub(crate) fn render_overlay(&mut self, state: &State, fps: Option<f32>) -> Result {
    use {
      kurbo::{Affine, Rect, Vec2},
      parley::{Alignment, AlignmentOptions, FontWeight, PositionedLayoutItem, StyleProperty},
      peniko::{Brush, Color, Fill},
      vello::{AaConfig, RenderParams},
    };

    if !state.status {
      let mut encoder = self
        .device
        .create_command_encoder(&CommandEncoderDescriptor::default());

      encoder.clear_texture(
        self.resources().overlay_view.texture(),
        &Self::IMAGE_SUBRESOURCE_RANGE_FULL,
      );

      self.queue.submit([encoder.finish()]);

      return Ok(());
    }

    let text = {
      let mut items = Vec::new();

      if let Some(position) = state.position {
        items.push(format!("{position}"));
      }

      if let Some(fps) = fps {
        items.push(format!("ƒ {}", fps.floor()));
      }

      items.push(format!("{:+.2}", state.encoder));

      for filter in &state.filters {
        items.push(filter.icon().into());
      }

      items.join(" ")
    };

    let bounds = if state.fit {
      Rect {
        x0: 0.0,
        y0: 0.0,
        x1: self.resolution.get() as f64,
        y1: self.resolution.get() as f64,
      }
    } else {
      let dy = self
        .size
        .x
        .get()
        .checked_sub(self.size.y.get())
        .map(|dy| dy as f64 / 2.0)
        .unwrap_or_default();

      let dx = self
        .size
        .y
        .get()
        .checked_sub(self.size.x.get())
        .map(|dx| dx as f64 / 2.0)
        .unwrap_or_default();

      Rect {
        x0: dx,
        y0: dy,
        x1: self.size.x.get() as f64 + dx,
        y1: self.size.y.get() as f64 + dy,
      }
    };

    self.vello_scene.reset();

    #[allow(clippy::cast_possible_truncation)]
    let font_size = bounds.height() as f32 * 0.033;

    let mut builder = self
      .layout_context
      .ranged_builder(&mut self.font_context, &text, 1.0, true);
    builder.push_default(StyleProperty::FontSize(font_size));
    builder.push_default(StyleProperty::FontStack(Self::FONT_STACK));
    builder.push_default(StyleProperty::FontWeight(FontWeight::LIGHT));

    let mut layout = builder.build(&text);
    layout.break_all_lines(None);
    layout.align(None, Alignment::Start, AlignmentOptions::default());

    for line in layout.lines() {
      for item in line.items() {
        match item {
          PositionedLayoutItem::GlyphRun(glyph_run) => {
            let run = glyph_run.run();
            self
              .vello_scene
              .draw_glyphs(run.font())
              .brush(&Brush::Solid(Color::WHITE))
              .font_size(font_size)
              .glyph_transform(
                run
                  .synthesis()
                  .skew()
                  .map(|angle| Affine::skew(angle.to_radians().tan().into(), 0.0)),
              )
              .hint(true)
              .normalized_coords(run.normalized_coords())
              .transform(Affine::translate(Vec2 {
                x: bounds.x0 + 10.0,
                y: bounds.y1
                  - 10.0
                  - f64::from(glyph_run.baseline()) * 2.0
                  - f64::from(run.metrics().descent),
              }))
              .draw(
                Fill::NonZero,
                glyph_run.positioned_glyphs().map(Self::convert_glyph),
              );
          }
          PositionedLayoutItem::InlineBox(_) => {
            return Err(Error::internal(
              "unexpected inline box while rendering overlay",
            ));
          }
        }
      }
    }

    self
      .vello_renderer
      .render_to_texture(
        &self.device,
        &self.queue,
        &self.vello_scene,
        &self.resources.as_ref().unwrap().overlay_view,
        &RenderParams {
          antialiasing_method: AaConfig::Msaa16,
          base_color: Color::TRANSPARENT,
          height: self.resolution.get(),
          width: self.resolution.get(),
        },
      )
      .context(error::RenderVello)?;

    Ok(())
  }

  pub(crate) fn resize(&mut self, size: Size, resolution: NonZeroU32) {
    if self.resources.is_some() && self.size == size && self.resolution == resolution {
      log::info!("skipping resize to same size");
      return;
    }

    if let Some((surface, config)) = &mut self.surface {
      config.height = size.y.get();
      config.width = size.x.get();
      surface.configure(&self.device, config);
    }

    self.resolution = Self::clamp_resolution(&self.limits, resolution);

    self.size = size;

    let tiling_view = self
      .device
      .create_texture(&TextureDescriptor {
        dimension: TextureDimension::D2,
        format: self.format.into(),
        label: label!(),
        mip_level_count: 1,
        sample_count: 1,
        size: Extent3d {
          depth_or_array_layers: 1,
          height: self.resolution.get(),
          width: self.resolution.get(),
        },
        usage: TextureUsages::RENDER_ATTACHMENT
          | TextureUsages::TEXTURE_BINDING
          | TextureUsages::COPY_SRC,
        view_formats: &[self.format.into()],
      })
      .create_view(&TextureViewDescriptor::default());

    let targets = [self.target(), self.target()];

    let tiling_bind_group = self.composite_bind_group(
      &targets[0].texture_view,
      &self.mirroring_sampler,
      &targets[1].texture_view,
      &self.mirroring_sampler,
    );

    let overlay_view = self.create_vello_texture(self.resolution);

    let overlay_bind_group = self.composite_bind_group(
      &tiling_view,
      &self.mirroring_sampler,
      &overlay_view,
      &self.clamp_to_border_sampler,
    );

    let field_texture_view = self.create_vello_texture(1.try_into().unwrap());

    self.resources = Some(Resources {
      dummy_field_texture: self.field_texture_bind_group(&field_texture_view),
      field_textures: HashMap::new(),
      overlay_bind_group,
      overlay_view,
      pool: Arc::new(Mutex::new(Vec::new())),
      targets,
      tiling_bind_group,
      tiling_view,
    });
  }

  fn resources(&self) -> &Resources {
    self.resources.as_ref().unwrap()
  }

  fn resources_mut(&mut self) -> &mut Resources {
    self.resources.as_mut().unwrap()
  }

  pub(crate) fn size(&self) -> Size {
    self.size
  }

  fn target(&self) -> Target {
    let texture_view = self
      .device
      .create_texture(&TextureDescriptor {
        dimension: TextureDimension::D2,
        format: self.format.into(),
        label: label!(),
        mip_level_count: 1,
        sample_count: 1,
        size: Extent3d {
          depth_or_array_layers: 1,
          height: self.resolution.get(),
          width: self.resolution.get(),
        },
        usage: TextureUsages::COPY_SRC
          | TextureUsages::RENDER_ATTACHMENT
          | TextureUsages::TEXTURE_BINDING,
        view_formats: &[self.format.into()],
      })
      .create_view(&TextureViewDescriptor::default());

    let bind_group = self.filter_bind_group(&self.frequencies, &texture_view, &self.samples);

    Target {
      bind_group,
      texture_view,
    }
  }

  fn write_texture(&self, data: &[f32], destination: &Texture) {
    self.queue.write_texture(
      TexelCopyTextureInfo {
        texture: destination,
        mip_level: 0,
        origin: Origin3d::ZERO,
        aspect: TextureAspect::All,
      },
      &data
        .iter()
        .flat_map(|value| value.to_le_bytes())
        .collect::<Vec<u8>>(),
      TexelCopyBufferLayout {
        offset: 0,
        bytes_per_row: None,
        rows_per_image: None,
      },
      Extent3d {
        width: data.len().try_into().unwrap(),
        height: 1,
        depth_or_array_layers: 1,
      },
    );
  }

  fn write_uniform_buffer(&self, pipeline: &Pipeline, uniforms: &[impl Uniforms]) {
    if uniforms.is_empty() {
      return;
    }

    let size = u64::from(pipeline.uniform_buffer_stride) * uniforms.len().into_u64();

    let mut buffer = self
      .queue
      .write_buffer_with(&pipeline.uniform_buffer, 0, size.try_into().unwrap())
      .unwrap();

    for (uniforms, dst) in uniforms
      .iter()
      .zip(buffer.chunks_mut(pipeline.uniform_buffer_stride.into_usize()))
    {
      uniforms.write(dst);
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn resolution_is_clamped_to_2d_texture_limit() {
    let resolution = Renderer::clamp_resolution(
      &Limits {
        max_texture_dimension_2d: 100,
        ..default()
      },
      101.try_into().unwrap(),
    );
    assert_eq!(resolution, 100.try_into().unwrap());
  }

  #[test]
  fn resolution_is_clamped_to_vello_render_bug_limit() {
    let resolution = Renderer::clamp_resolution(&Limits::default(), 5809.try_into().unwrap());
    assert_eq!(resolution, 5808.try_into().unwrap());
  }
}
