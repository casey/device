use super::*;

pub(crate) struct Renderer {
  bindings: Option<Bindings>,
  composite_pipeline: Pipeline,
  device: wgpu::Device,
  error_channel: mpsc::Receiver<wgpu::Error>,
  filter_pipeline: Pipeline,
  filtering_sampler: Sampler,
  font_context: FontContext,
  format: Format,
  frame: u64,
  frame_times: VecDeque<Instant>,
  frequencies: TextureView,
  layout_context: LayoutContext,
  limits: Limits,
  non_filtering_sampler: Sampler,
  overlay_renderer: vello::Renderer,
  overlay_scene: vello::Scene,
  queue: Queue,
  resolution: NonZeroU32,
  samples: TextureView,
  size: Vector2<NonZeroU32>,
  surface: Option<(Surface<'static>, SurfaceConfiguration)>,
}

impl Renderer {
  fn bindings(&self) -> &Bindings {
    self.bindings.as_ref().unwrap()
  }

  fn bytes_per_row_with_padding(&self) -> u32 {
    const MASK: u32 = COPY_BYTES_PER_ROW_ALIGNMENT - 1;
    (self.resolution.get() * COLOR_CHANNELS + MASK) & !MASK
  }

  pub(crate) fn capture(&self, callback: impl FnOnce(Image) + Send + 'static) -> Result {
    let bytes_per_row_with_padding = self.bytes_per_row_with_padding();

    let mut encoder = self
      .device
      .create_command_encoder(&CommandEncoderDescriptor::default());

    let captures = self.bindings().captures.clone();

    let capture = captures.lock().unwrap().pop().unwrap_or_else(|| {
      self.device.create_buffer(&BufferDescriptor {
        label: label!(),
        mapped_at_creation: false,
        size: (self.bytes_per_row_with_padding() * self.resolution.get()).into(),
        usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
      })
    });

    encoder.copy_texture_to_buffer(
      TexelCopyTextureInfo {
        texture: self.bindings().targets[0].texture_view.texture(),
        mip_level: 0,
        origin: Origin3d::ZERO,
        aspect: TextureAspect::All,
      },
      TexelCopyBufferInfo {
        buffer: &capture,
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

    let buffer = capture.clone();
    let resolution = self.resolution;
    let format = self.format;
    capture.map_async(MapMode::Read, .., move |result| {
      if let Err(err) = result {
        eprintln!("failed to map capture buffer: {err}");
        return;
      }

      thread::spawn(move || {
        let view = buffer.get_mapped_range(..);

        let channels = COLOR_CHANNELS.into_usize();
        let bytes_per_row = resolution.get().into_usize() * channels;

        let mut image = Image::default();
        image.resize(resolution.get(), resolution.get());
        for (src, dst) in view
          .chunks(bytes_per_row_with_padding.into_usize())
          .map(|src| &src[..bytes_per_row])
          .zip(image.data_mut().chunks_mut(bytes_per_row))
        {
          for (src, dst) in src.chunks(channels).zip(dst.chunks_mut(channels)) {
            format.swizzle(src, dst);
          }
        }

        drop(view);

        buffer.unmap();

        captures.lock().unwrap().push(buffer);

        callback(image);
      });
    });

    Ok(())
  }

  fn composite_bind_group(&self, back: &TextureView, front: &TextureView) -> BindGroup {
    let mut next = 0;
    let mut binding = || {
      let binding = next;
      next += 1;
      binding
    };
    self.device.create_bind_group(&BindGroupDescriptor {
      layout: &self.composite_pipeline.bind_group_layout,
      entries: &[
        BindGroupEntry {
          binding: binding(),
          resource: BindingResource::TextureView(back),
        },
        BindGroupEntry {
          binding: binding(),
          resource: BindingResource::TextureView(front),
        },
        BindGroupEntry {
          binding: binding(),
          resource: BindingResource::Sampler(&self.non_filtering_sampler),
        },
        BindGroupEntry {
          binding: binding(),
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
    let mut next = 0;
    let mut binding = || {
      let binding = next;
      next += 1;
      binding
    };
    device.create_bind_group_layout(&BindGroupLayoutDescriptor {
      entries: &[
        BindGroupLayoutEntry {
          binding: binding(),
          count: None,
          ty: BindingType::Texture {
            multisampled: false,
            sample_type: TextureSampleType::Float { filterable: true },
            view_dimension: TextureViewDimension::D2,
          },
          visibility: ShaderStages::FRAGMENT,
        },
        BindGroupLayoutEntry {
          binding: binding(),
          count: None,
          ty: BindingType::Texture {
            multisampled: false,
            sample_type: TextureSampleType::Float { filterable: true },
            view_dimension: TextureViewDimension::D2,
          },
          visibility: ShaderStages::FRAGMENT,
        },
        BindGroupLayoutEntry {
          binding: binding(),
          count: None,
          ty: BindingType::Sampler(SamplerBindingType::NonFiltering),
          visibility: ShaderStages::FRAGMENT,
        },
        BindGroupLayoutEntry {
          binding: binding(),
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

  fn create_render_pipeline(
    device: &wgpu::Device,
    pipeline_layout: &PipelineLayout,
    format: Format,
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

  fn draw(
    bind_group: &BindGroup,
    encoder: &mut CommandEncoder,
    tiling: Option<(Tiling, u32)>,
    uniform: u32,
    view: &TextureView,
    pipeline: &Pipeline,
  ) {
    let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
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
    });

    pass.set_bind_group(
      0,
      Some(bind_group),
      &[pipeline.uniform_buffer_stride * uniform],
    );

    pass.set_pipeline(&pipeline.render_pipeline);

    if let Some((tiling, filter)) = tiling {
      tiling.set_viewport(&mut pass, filter);
    }

    pass.draw(0..3, 0..1);
  }

  fn filter_bind_group(
    &self,
    frequencies: &TextureView,
    source: &TextureView,
    samples: &TextureView,
  ) -> BindGroup {
    let mut next = 0;
    let mut binding = || {
      let binding = next;
      next += 1;
      binding
    };
    self.device.create_bind_group(&BindGroupDescriptor {
      layout: &self.filter_pipeline.bind_group_layout,
      entries: &[
        BindGroupEntry {
          binding: binding(),
          resource: BindingResource::Sampler(&self.filtering_sampler),
        },
        BindGroupEntry {
          binding: binding(),
          resource: BindingResource::TextureView(frequencies),
        },
        BindGroupEntry {
          binding: binding(),
          resource: BindingResource::TextureView(source),
        },
        BindGroupEntry {
          binding: binding(),
          resource: BindingResource::Sampler(&self.non_filtering_sampler),
        },
        BindGroupEntry {
          binding: binding(),
          resource: BindingResource::TextureView(samples),
        },
        BindGroupEntry {
          binding: binding(),
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
    let mut next = 0;
    let mut binding = || {
      let binding = next;
      next += 1;
      binding
    };
    device.create_bind_group_layout(&BindGroupLayoutDescriptor {
      entries: &[
        BindGroupLayoutEntry {
          binding: binding(),
          count: None,
          ty: BindingType::Sampler(SamplerBindingType::Filtering),
          visibility: ShaderStages::FRAGMENT,
        },
        BindGroupLayoutEntry {
          binding: binding(),
          count: None,
          ty: BindingType::Texture {
            multisampled: false,
            sample_type: TextureSampleType::Float { filterable: false },
            view_dimension: TextureViewDimension::D1,
          },
          visibility: ShaderStages::FRAGMENT,
        },
        BindGroupLayoutEntry {
          binding: binding(),
          count: None,
          ty: BindingType::Texture {
            multisampled: false,
            sample_type: TextureSampleType::Float { filterable: true },
            view_dimension: TextureViewDimension::D2,
          },
          visibility: ShaderStages::FRAGMENT,
        },
        BindGroupLayoutEntry {
          binding: binding(),
          count: None,
          ty: BindingType::Sampler(SamplerBindingType::NonFiltering),
          visibility: ShaderStages::FRAGMENT,
        },
        BindGroupLayoutEntry {
          binding: binding(),
          count: None,
          ty: BindingType::Texture {
            multisampled: false,
            sample_type: TextureSampleType::Float { filterable: false },
            view_dimension: TextureViewDimension::D1,
          },
          visibility: ShaderStages::FRAGMENT,
        },
        BindGroupLayoutEntry {
          binding: binding(),
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

  pub(crate) fn frame(&self) -> u64 {
    self.frame
  }

  pub(crate) async fn new(
    format: Option<Format>,
    present_mode: Option<PresentMode>,
    resolution: NonZeroU32,
    size: Vector2<NonZeroU32>,
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
        required_features: Features::CLEAR_TEXTURE,
        required_limits: Limits::default(),
        trace: Trace::Off,
      })
      .await
      .context(error::RequestDevice)?;

    let (surface, format) = if let Some(surface) = surface {
      let formats = surface.get_capabilities(&adapter).formats;

      let format = if let Some(format) = format {
        if !formats.iter().any(|supported| *supported == format.into()) {
          return Err(error::UnsupportedSurfaceFormat { format }.build());
        }
        format
      } else {
        Format::try_from(formats[0])?
      };

      let mut config = surface
        .get_default_config(&adapter, size.x.get(), size.y.get())
        .context(error::DefaultConfig)?;

      config.format = format.into();

      if let Some(present_mode) = present_mode {
        config.present_mode = present_mode.into();
      }

      surface.configure(&device, &config);

      (Some((surface, config)), format)
    } else {
      (None, format.unwrap_or_default())
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
        size: u64::from(uniform_buffer_stride) * 3,
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

    let overlay_renderer = vello::Renderer::new(
      &device,
      vello::RendererOptions {
        antialiasing_support: vello::AaSupport::all(),
        num_init_threads: Some(1.try_into().unwrap()),
        pipeline_cache: None,
        use_cpu: false,
      },
    )
    .context(error::CreateOverlayRenderer)?;

    let mut renderer = Self {
      bindings: None,
      composite_pipeline,
      device,
      error_channel,
      filter_pipeline,
      filtering_sampler,
      font_context: FontContext::new(),
      format,
      frame: 0,
      frame_times: VecDeque::with_capacity(100),
      frequencies,
      layout_context: LayoutContext::new(),
      limits,
      non_filtering_sampler,
      overlay_renderer,
      overlay_scene: vello::Scene::new(),
      queue,
      resolution,
      samples,
      size,
      surface,
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

  pub(crate) fn reload_shader(&mut self) -> Result {
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

  pub(crate) fn render(&mut self, analyzer: &Analyzer, state: &State, now: Instant) -> Result {
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

    if self.frame_times.len() == self.frame_times.capacity() {
      self.frame_times.pop_front();
    }

    self.frame_times.push_back(now);

    let fps = if self.frame_times.len() >= 2 {
      let elapsed = *self.frame_times.back().unwrap() - *self.frame_times.front().unwrap();
      Some(1000.0 / (elapsed.as_millis() as f32 / self.frame_times.len() as f32))
    } else {
      None
    };

    let filters = state.filters.len() + 1;

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

    let transient = state.transient();

    {
      let mut uniforms = Vec::new();

      for (i, filter) in state
        .filters
        .iter()
        .chain(iter::once(&transient))
        .enumerate()
      {
        let i = u32::try_from(i).unwrap();
        uniforms.push(FilterUniforms {
          color: filter.color,
          coordinates: filter.coordinates,
          field: filter.field,
          frequency_range,
          front_offset: tiling.source_offset(i),
          gain,
          interpolate: state.interpolate,
          offset: tiling.offset(i),
          parameter: filter.field.parameter(),
          position: filter.position,
          repeat: state.repeat,
          resolution: tiling.resolution as f32,
          rms: if state.spread {
            rms * (i as f32 + 1.0) / filters as f32
          } else {
            rms
          },
          sample_range,
          tiling: tiling.size,
          wrap: filter.wrap,
        });
      }

      self.write_uniform_buffer(&self.filter_pipeline, &uniforms);
    }

    let resolution = Vec2f::new(self.resolution.get() as f32, self.resolution.get() as f32);

    {
      let uniforms = [
        CompositeUniforms {
          back_read: tiling.back_read(filter_count),
          fit: false,
          front_read: tiling.front_read(filter_count),
          resolution,
        },
        CompositeUniforms {
          back_read: true,
          fit: false,
          front_read: true,
          resolution,
        },
        CompositeUniforms {
          back_read: true,
          fit: state.fit,
          front_read: true,
          resolution: Vec2f::new(self.size.x.get() as f32, self.size.y.get() as f32),
        },
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

    for target in &self.bindings().targets {
      encoder.clear_texture(
        target.texture_view.texture(),
        &ImageSubresourceRange {
          array_layer_count: None,
          aspect: TextureAspect::All,
          base_array_layer: 0,
          base_mip_level: 0,
          mip_level_count: None,
        },
      );
    }

    encoder.clear_texture(
      self.bindings().tiling_view.texture(),
      &ImageSubresourceRange {
        array_layer_count: None,
        aspect: TextureAspect::All,
        base_array_layer: 0,
        base_mip_level: 0,
        mip_level_count: None,
      },
    );

    let mut source = 0;
    let mut destination = 1;
    for i in 0..filters {
      let i = u32::try_from(i).unwrap();
      Self::draw(
        &self.bindings().targets[source].bind_group,
        &mut encoder,
        Some((tiling, i)),
        i,
        &self.bindings().targets[destination].texture_view,
        &self.filter_pipeline,
      );
      (source, destination) = (destination, source);
    }

    Self::draw(
      &self.bindings().tiling_bind_group,
      &mut encoder,
      None,
      0,
      &self.bindings().tiling_view,
      &self.composite_pipeline,
    );

    self.render_overlay(state, fps)?;

    Self::draw(
      &self.bindings().overlay_bind_group,
      &mut encoder,
      None,
      1,
      &self.bindings().targets[0].texture_view,
      &self.composite_pipeline,
    );

    if let Some(frame) = &frame {
      Self::draw(
        &self.bindings().overlay_bind_group,
        &mut encoder,
        None,
        2,
        &frame.texture.create_view(&TextureViewDescriptor::default()),
        &self.composite_pipeline,
      );
    }

    self.queue.submit([encoder.finish()]);

    if let Some(frame) = frame {
      frame.present();
    }

    log::info!(
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

  pub(crate) fn render_overlay(&mut self, state: &State, fps: Option<f32>) -> Result {
    use {
      kurbo::{Affine, Rect, Vec2},
      parley::{
        Alignment, AlignmentOptions, FontFamily, FontStack, FontWeight, GenericFamily,
        PositionedLayoutItem, StyleProperty,
      },
      peniko::{Brush, Color, Fill},
      vello::{AaConfig, RenderParams},
    };

    self.overlay_scene.reset();

    let text = if let Some(text) = state.text.clone() {
      text
    } else if state.status {
      let mut items = Vec::new();

      if let Some(fps) = fps {
        items.push(format!("ƒ {}", fps.floor()));
      }

      let parameter = state.parameter.value();
      items.push(if parameter >= 0 {
        format!("+{parameter}")
      } else {
        parameter.to_string()
      });

      for filter in &state.filters {
        items.push(filter.icon().into());
      }

      Text {
        size: 0.033,
        string: items.join(" "),
        x: 0.0,
        y: 0.0,
      }
    } else {
      return Ok(());
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

    #[allow(clippy::cast_possible_truncation)]
    let font_size = bounds.height() as f32 * text.size;

    let mut builder =
      self
        .layout_context
        .ranged_builder(&mut self.font_context, &text.string, 1.0, true);
    builder.push_default(StyleProperty::FontSize(font_size));
    builder.push_default(StyleProperty::FontStack(FontStack::List(Cow::Borrowed(&[
      FontFamily::Named("Helvetica Neue".into()),
      FontFamily::Generic(GenericFamily::SansSerif),
      FontFamily::Named("Apple Symbols".into()),
      FontFamily::Named("Zapf Dingbats".into()),
      FontFamily::Named("Last Resort".into()),
    ]))));
    builder.push_default(StyleProperty::FontWeight(FontWeight::LIGHT));

    let mut layout = builder.build(&text.string);
    layout.break_all_lines(None);
    layout.align(None, Alignment::Start, AlignmentOptions::default());

    for line in layout.lines() {
      for item in line.items() {
        match item {
          PositionedLayoutItem::GlyphRun(glyph_run) => {
            let run = glyph_run.run();
            let mut offset = glyph_run.offset();

            self
              .overlay_scene
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
                x: text.x * bounds.width() + bounds.x0 + 10.0,
                y: text.y * bounds.height() + bounds.y1
                  - 10.0
                  - f64::from(glyph_run.baseline())
                  - f64::from(run.metrics().descent),
              }))
              .draw(
                Fill::NonZero,
                glyph_run.glyphs().map(|glyph| {
                  let x = offset + glyph.x;
                  offset += glyph.advance;
                  vello::Glyph {
                    id: glyph.id,
                    x,
                    y: glyph.y,
                  }
                }),
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
      .overlay_renderer
      .render_to_texture(
        &self.device,
        &self.queue,
        &self.overlay_scene,
        &self.bindings.as_ref().unwrap().overlay_view,
        &RenderParams {
          antialiasing_method: AaConfig::Msaa16,
          base_color: Color::TRANSPARENT,
          height: self.resolution.get(),
          width: self.resolution.get(),
        },
      )
      .context(error::RenderOverlay)?;

    Ok(())
  }

  pub(crate) fn resize(&mut self, size: Vector2<NonZeroU32>, resolution: NonZeroU32) {
    if let Some((surface, config)) = &mut self.surface {
      config.height = size.y.get();
      config.width = size.x.get();
      surface.configure(&self.device, config);
    }

    self.resolution = resolution
      .min(self.limits.max_texture_dimension_2d.try_into().unwrap())
      .min(5808.try_into().unwrap());
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

    let tiling_bind_group =
      self.composite_bind_group(&targets[0].texture_view, &targets[1].texture_view);

    let overlay_view = self
      .device
      .create_texture(&TextureDescriptor {
        dimension: TextureDimension::D2,
        format: TextureFormat::Rgba8Unorm,
        label: label!(),
        mip_level_count: 1,
        sample_count: 1,
        size: Extent3d {
          depth_or_array_layers: 1,
          height: self.resolution.get(),
          width: self.resolution.get(),
        },
        usage: TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING,
        view_formats: &[TextureFormat::Rgba8Unorm],
      })
      .create_view(&TextureViewDescriptor::default());

    let overlay_bind_group = self.composite_bind_group(&tiling_view, &overlay_view);

    self.bindings = Some(Bindings {
      captures: Arc::new(Mutex::new(Vec::new())),
      overlay_bind_group,
      overlay_view,
      targets,
      tiling_bind_group,
      tiling_view,
    });
  }

  fn target(&self) -> Target {
    let texture = self.device.create_texture(&TextureDescriptor {
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
    });

    let texture_view = texture.create_view(&TextureViewDescriptor::default());

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
  use {super::*, std::sync::LazyLock};

  macro_rules! name {
    () => {{
      fn f() {}
      std::any::type_name_of_val(&f)
        .rsplit("::")
        .skip(1)
        .next()
        .unwrap()
        .replace('_', "-")
    }};
  }

  static RENDERER: LazyLock<Mutex<Renderer>> = LazyLock::new(|| {
    let resolution = 256.try_into().unwrap();
    Mutex::new(
      pollster::block_on(Renderer::new(
        None,
        None,
        resolution,
        Vector2::new(resolution, resolution),
        None,
      ))
      .unwrap(),
    )
  });

  struct Baseline {
    height: Option<u32>,
    name: String,
    resolution: Option<u32>,
    state: State,
    width: Option<u32>,
  }

  impl Baseline {
    fn height(mut self, height: u32) -> Self {
      self.height = Some(height);
      self
    }

    fn new(name: String) -> Self {
      Self {
        height: None,
        name,
        resolution: None,
        state: State::default(),
        width: None,
      }
    }

    fn resolution(mut self, resolution: u32) -> Self {
      self.resolution = Some(resolution);
      self
    }

    #[track_caller]
    fn run(self) {
      let mut renderer = RENDERER.lock().unwrap();

      let resolution = self.resolution.unwrap_or(256);

      let width = self.width.unwrap_or(resolution).try_into().unwrap();

      let height = self.height.unwrap_or(resolution).try_into().unwrap();

      renderer.resize(Vector2::new(width, height), resolution.try_into().unwrap());

      renderer
        .render(&Analyzer::new(), &self.state, Instant::now())
        .unwrap();

      let (tx, rx) = mpsc::channel();

      let expected = Utf8PathBuf::from(format!("baseline/{}.png", self.name));
      let actual = expected.with_extension("test.png");

      renderer
        .capture(move |image| {
          image.save(&actual).unwrap();
          tx.send(image).unwrap();
        })
        .unwrap();

      renderer.device.poll(wgpu::PollType::Wait).unwrap();

      drop(renderer);

      let actual = rx.recv().unwrap();

      if expected.try_exists().unwrap() {
        assert!(
          actual == Image::load(&expected).unwrap(),
          "baseline image mismatch",
        );
      } else {
        panic!("no baseline image found for {}", self.name);
      }
    }

    fn state(mut self, state: State) -> Self {
      self.state = state;
      self
    }

    fn width(mut self, width: u32) -> Self {
      self.width = Some(width);
      self
    }
  }

  #[test]
  #[ignore]
  fn circle() {
    Baseline::new(name!())
      .state(State::default().invert().circle().push())
      .run();
  }

  #[test]
  #[ignore]
  fn circle_small_even() {
    Baseline::new(name!())
      .resolution(10)
      .state(State::default().invert().circle().push())
      .run();
  }

  #[test]
  #[ignore]
  fn circle_small_odd() {
    Baseline::new(name!())
      .resolution(9)
      .state(State::default().invert().circle().push())
      .run();
  }

  #[test]
  #[ignore]
  fn circle_medium_even() {
    Baseline::new(name!())
      .resolution(32)
      .state(State::default().invert().circle().push())
      .run();
  }

  #[test]
  #[ignore]
  fn circle_medium_odd() {
    Baseline::new(name!())
      .resolution(31)
      .state(State::default().invert().circle().push())
      .run();
  }

  #[test]
  #[ignore]
  fn default() {
    Baseline::new(name!()).state(State::default()).run();
  }

  #[test]
  #[ignore]
  fn left() {
    Baseline::new(name!())
      .state(State::default().invert().left().push())
      .run();
  }

  #[test]
  #[ignore]
  fn x() {
    Baseline::new(name!())
      .state(State::default().invert().x().push())
      .run();
  }

  #[test]
  #[ignore]
  fn x_oblong() {
    Baseline::new(name!())
      .width(256)
      .height(128)
      .state(State::default().invert().x().push())
      .run();
  }

  #[test]
  #[ignore]
  fn x_small_even() {
    Baseline::new(name!())
      .resolution(10)
      .state(State::default().invert().x().push())
      .run();
  }

  #[test]
  #[ignore]
  fn x_small_odd() {
    Baseline::new(name!())
      .resolution(9)
      .state(State::default().invert().x().push())
      .run();
  }

  #[test]
  #[ignore]
  fn x_medium_even() {
    Baseline::new(name!())
      .resolution(32)
      .state(State::default().invert().x().push())
      .run();
  }

  #[test]
  #[ignore]
  fn x_medium_odd() {
    Baseline::new(name!())
      .resolution(31)
      .state(State::default().invert().x().push())
      .run();
  }

  #[test]
  #[ignore]
  fn tile() {
    Baseline::new(name!())
      .state(
        State::default()
          .invert()
          .x()
          .push()
          .circle()
          .push()
          .x()
          .push()
          .circle()
          .push()
          .tile(true),
      )
      .run();
  }

  #[test]
  #[ignore]
  fn circle_scale() {
    Baseline::new(name!())
      .state(State::default().invert().circle().scale(2.0).times(2))
      .run();
  }

  #[test]
  #[ignore]
  fn x_scale() {
    Baseline::new(name!())
      .state(State::default().invert().x().scale(2.0).times(2))
      .run();
  }

  #[test]
  #[ignore]
  fn circle_scale_interpolated() {
    Baseline::new(name!())
      .state(
        State::default()
          .invert()
          .circle()
          .scale(2.0)
          .times(2)
          .interpolate(true),
      )
      .run();
  }

  #[test]
  #[ignore]
  fn x_scale_interpolated() {
    Baseline::new(name!())
      .state(
        State::default()
          .invert()
          .x()
          .scale(2.0)
          .times(2)
          .interpolate(true),
      )
      .run();
  }

  #[test]
  #[ignore]
  fn sampling_repeat_on() {
    Baseline::new(name!())
      .state(
        State::default()
          .repeat(true)
          .rotate_position(0.2 * TAU)
          .rotate_color(Axis::Green, 0.1 * TAU)
          .all()
          .push()
          .push(),
      )
      .run();
  }

  #[test]
  #[ignore]
  fn sampling_repeat_off() {
    Baseline::new(name!())
      .state(
        State::default()
          .repeat(false)
          .rotate_position(0.2 * TAU)
          .rotate_color(Axis::Green, 0.1 * TAU)
          .all()
          .push()
          .push(),
      )
      .run();
  }

  #[test]
  #[ignore]
  fn cross() {
    Baseline::new(name!())
      .state(State::default().invert().cross().push())
      .run();
  }

  #[test]
  #[ignore]
  fn square() {
    Baseline::new(name!())
      .state(State::default().invert().square().push())
      .run();
  }

  #[test]
  #[ignore]
  fn triangle() {
    Baseline::new(name!())
      .state(State::default().invert().triangle().push())
      .run();
  }

  #[test]
  #[ignore]
  fn all() {
    Baseline::new(name!())
      .state(State::default().invert().all().push())
      .run();
  }

  #[test]
  #[ignore]
  fn none() {
    Baseline::new(name!())
      .state(State::default().invert().none().push())
      .run();
  }

  #[test]
  #[ignore]
  fn rotate_red() {
    Baseline::new(name!())
      .state(
        State::default()
          .rotate_color(Axis::Red, TAU / 2.0)
          .all()
          .push(),
      )
      .run();
  }

  #[test]
  #[ignore]
  fn rotate_green() {
    Baseline::new(name!())
      .state(
        State::default()
          .rotate_color(Axis::Green, TAU / 2.0)
          .all()
          .push(),
      )
      .run();
  }

  #[test]
  #[ignore]
  fn rotate_blue() {
    Baseline::new(name!())
      .state(
        State::default()
          .rotate_color(Axis::Blue, TAU / 2.0)
          .all()
          .push(),
      )
      .run();
  }

  #[test]
  #[ignore]
  fn coordinates() {
    Baseline::new(name!())
      .state(State::default().coordinates(true).all().push())
      .run();
  }

  #[test]
  #[ignore]
  fn resolution_is_clamped_to_2d_texture_limit() {
    let resolution = 65536.try_into().unwrap();
    let size = Vector2::new(resolution, resolution);
    let mut renderer =
      pollster::block_on(Renderer::new(None, None, resolution, size, None)).unwrap();
    renderer.resize(size, resolution);
  }

  #[test]
  #[ignore]
  fn resolution_is_clamped_to_vello_render_bug_limit() {
    env_logger::init();

    let resolution = 5809.try_into().unwrap();
    let size = Vector2::new(resolution, resolution);
    let mut renderer =
      pollster::block_on(Renderer::new(None, None, resolution, size, None)).unwrap();
    renderer.resize(size, resolution);
    renderer
      .render(
        &Analyzer::new(),
        &State::default().text(Some(Text {
          string: "hi".into(),
          size: 1.0,
          x: 1.0,
          y: 1.0,
        })),
        Instant::now(),
      )
      .unwrap();
  }
}
