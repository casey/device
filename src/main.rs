use {
  self::{
    analyzer::Analyzer, app::App, arguments::Arguments, bindings::Bindings, config::Config,
    controller::Controller, emitter::Emitter, error::Error, event::Event, field::Field,
    filter::Filter, format::Format, fps::Fps, frame::Frame, hub::Hub, image::Image, input::Input,
    into_u64::IntoU64, into_u128::IntoU128, into_usize::IntoUsize, into_utf8_path::IntoUtf8Path,
    message::Message, options::Options, parameter::Parameter, patch::Patch, program::Program,
    recorder::Recorder, renderer::Renderer, scene::Scene, score::Score, shared::Shared,
    sound::Sound, state::State, stream::Stream, subcommand::Subcommand, synthesizer::Synthesizer,
    tally::Tally, target::Target, templates::ShaderWgsl, text::Text, tiling::Tiling, track::Track,
    uniforms::Uniforms, voice::Voice,
  },
  boilerplate::Boilerplate,
  camino::{Utf8Path, Utf8PathBuf},
  clap::{Parser, ValueEnum},
  hound::{WavSpec, WavWriter},
  indicatif::{ProgressBar, ProgressStyle},
  nalgebra::Vector2,
  parley::{FontContext, LayoutContext},
  rand::{Rng, SeedableRng, distr::Uniform, rngs::SmallRng},
  regex::{Regex, RegexBuilder},
  rodio::{
    Decoder, OutputStream, Sink, Source,
    cpal::{
      self, Sample, SampleFormat, StreamConfig, SupportedBufferSize, SupportedStreamConfig,
      SupportedStreamConfigRange,
      traits::{DeviceTrait, HostTrait, StreamTrait},
    },
    mixer::Mixer,
  },
  rustfft::{FftPlanner, num_complex::Complex},
  serde::Deserialize,
  snafu::{ErrorCompat, IntoError, OptionExt, ResultExt, Snafu},
  std::{
    array,
    backtrace::{Backtrace, BacktraceStatus},
    borrow::Cow,
    collections::VecDeque,
    env, f32,
    fmt::{self, Display, Formatter},
    fs::{self, File},
    io::{self, BufReader, BufWriter, Write},
    iter,
    num::NonZeroU32,
    ops::{Add, AddAssign, SubAssign},
    process::{self, Command, ExitStatus, Stdio},
    str::FromStr,
    sync::{Arc, Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard, mpsc},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
  },
  strum::{EnumIter, IntoEnumIterator, IntoStaticStr},
  tempfile::TempDir,
  vello::{kurbo, peniko},
  walkdir::WalkDir,
  wgpu::{
    AddressMode, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, Buffer,
    BufferBinding, BufferBindingType, BufferDescriptor, BufferUsages, COPY_BYTES_PER_ROW_ALIGNMENT,
    CommandEncoder, CommandEncoderDescriptor, DeviceDescriptor, Extent3d, Features, FilterMode,
    FragmentState, ImageSubresourceRange, Instance, Limits, LoadOp, MapMode, MemoryHints,
    MultisampleState, Operations, Origin3d, PipelineCompilationOptions, PipelineLayoutDescriptor,
    PowerPreference, PrimitiveState, Queue, RenderPass, RenderPassColorAttachment,
    RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, RequestAdapterOptions, Sampler,
    SamplerBindingType, SamplerDescriptor, ShaderModuleDescriptor, ShaderSource, ShaderStages,
    StoreOp, Surface, SurfaceConfiguration, TexelCopyBufferInfo, TexelCopyBufferLayout,
    TexelCopyTextureInfo, Texture, TextureAspect, TextureDescriptor, TextureDimension,
    TextureFormat, TextureSampleType, TextureUsages, TextureView, TextureViewDescriptor,
    TextureViewDimension, Trace, VertexState,
  },
  winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{ElementState, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{Key, NamedKey},
    window::{Window, WindowAttributes, WindowId},
  },
};

macro_rules! label {
  () => {
    Some(concat!(file!(), ":", line!(), ":", column!()))
  };
}

mod analyzer;
mod app;
mod arguments;
mod bindings;
mod config;
mod controller;
mod emitter;
mod error;
mod event;
mod field;
mod filter;
mod format;
mod fps;
mod frame;
mod hub;
mod image;
mod input;
mod into_u128;
mod into_u64;
mod into_usize;
mod into_utf8_path;
mod message;
mod options;
mod parameter;
mod patch;
mod program;
mod recorder;
mod renderer;
mod scene;
mod score;
mod shared;
mod sound;
mod state;
mod stream;
mod subcommand;
mod synthesizer;
mod tally;
mod target;
mod templates;
mod text;
mod tiling;
mod track;
mod uniforms;
mod voice;

const KIB: usize = 1 << 10;
const MIB: usize = KIB << 10;

const AUDIO: &str = "audio.wav";
const COLOR_CHANNELS: u32 = 4;
const DEFAULT_FPS: NonZeroU32 = NonZeroU32::new(60).unwrap();
const DEFAULT_RESOLUTION: NonZeroU32 = NonZeroU32::new(1024).unwrap();
const FONT: &str = "Helvetica Neue";
const RECORDING: &str = "recording.mp4";

type Result<T = (), E = Error> = std::result::Result<T, E>;

type Mat3f = nalgebra::Matrix3<f32>;
type Mat4f = nalgebra::Matrix4<f32>;
type Vec2f = nalgebra::Vector2<f32>;
type Vec4f = nalgebra::Vector4<f32>;

fn default<T: Default>() -> T {
  T::default()
}

fn invert_color() -> Mat4f {
  Mat4f::from_diagonal(&Vec4f::new(-1.0, -1.0, -1.0, 1.0))
}

fn pad(i: usize, alignment: usize) -> usize {
  assert!(alignment.is_power_of_two());
  (i + alignment - 1) & !(alignment - 1)
}

fn main() {
  env_logger::init();

  if let Err(err) = Arguments::parse().run() {
    eprintln!("error: {err}");

    for (i, err) in err.iter_chain().skip(1).enumerate() {
      if i == 0 {
        eprintln!();
        eprintln!("because:");
      }

      eprintln!("- {err}");
    }

    if let Some(backtrace) = err.backtrace()
      && backtrace.status() == BacktraceStatus::Captured
    {
      eprintln!("backtrace:");
      eprintln!("{backtrace}");
    }

    process::exit(1);
  }
}
