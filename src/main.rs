use {
  self::{
    alignment::Alignment,
    allocator::Allocator,
    analyzer::Analyzer,
    app::App,
    arguments::Arguments,
    array_string::ArrayString,
    axis::Axis,
    bindings::Bindings,
    bool_ext::BoolExt,
    callback::Callback,
    capture::Capture,
    capture_thread::CaptureThread,
    command::Command,
    command_entry::CommandEntry,
    command_ext::CommandExt,
    commands::Commands,
    composite_uniforms::CompositeUniforms,
    config::Config,
    controller::Controller,
    counter::Counter,
    error::Error,
    event::Event,
    field::Field,
    filter::Filter,
    filter_uniforms::FilterUniforms,
    fps::Fps,
    frame::Frame,
    functions::{default, display, tempdir, thread_spawn},
    history::History,
    hub::Hub,
    image::Image,
    image_format::ImageFormat,
    input::Input,
    interrupt::Interrupt,
    into_stereo::IntoStereo,
    into_utf8_path::IntoUtf8Path,
    message::Message,
    mirror::Mirror,
    mode::{Mode, ModeKind},
    options::Options,
    patch::Patch,
    pipeline::Pipeline,
    position::Position,
    present_mode::PresentMode,
    preset::Preset,
    press::Press,
    program::Program,
    recorder::Recorder,
    recorder_thread::RecorderThread,
    renderer::Renderer,
    resampler_ext::ResamplerExt,
    resources::Resources,
    scene::Scene,
    score::Score,
    script::Script,
    shared::Shared,
    sound::Sound,
    sound_format::SoundFormat,
    space::Space,
    state::State,
    stream_config_display::StreamConfigDisplay,
    subcommand::Subcommand,
    tally::Tally,
    tap::Tap,
    target::Target,
    templates::{CompositeWgsl, FilterWgsl, VertexWgsl},
    tempo::Tempo,
    texture_field::TextureField,
    tick::Tick,
    tiling::Tiling,
    to_affine::ToAffine,
    track::Track,
    transformation2::Transformation2,
    transformation3::Transformation3,
    uniforms::Uniforms,
    window_attributes_ext::WindowAttributesExt,
  },
  boilerplate::Boilerplate,
  camino::{Utf8Path, Utf8PathBuf},
  clap::{Parser, ValueEnum},
  cpal::{
    self, BufferSize, SampleFormat, Stream, StreamConfig, SupportedBufferSize,
    SupportedStreamConfig, SupportedStreamConfigRange,
    traits::{DeviceTrait, HostTrait, StreamTrait},
  },
  fundsp::wave::Wave,
  hound::{WavSpec, WavWriter},
  indicatif::{ProgressBar, ProgressStyle},
  midly::num::u7,
  nalgebra::{
    Rotation2, Translation2, Translation3, Unit, UnitQuaternion, Vector2, matrix, vector,
  },
  ordered_float::OrderedFloat,
  parley::{FontContext, FontFamily, FontStack, FontWeight, GenericFamily, LayoutContext},
  rand::{Rng, SeedableRng, prelude::SliceRandom, rngs::SmallRng, seq::IndexedRandom},
  regex::{Regex, RegexBuilder},
  rustfft::{FftPlanner, num_complex::Complex},
  serde::Deserialize,
  snafu::{ErrorCompat, IntoError, OptionExt, ResultExt, Snafu, ensure},
  std::{
    any::Any,
    backtrace::{Backtrace, BacktraceStatus},
    borrow::Cow,
    cmp::Reverse,
    collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, VecDeque},
    env, f32,
    ffi::OsString,
    fmt::{self, Debug, Display, Formatter},
    fs::{self, File},
    io::{self, BufReader, BufWriter, Write},
    mem,
    num::NonZeroU32,
    ops::{Add, Bound, Deref, Range},
    process::{self, ExitStatus, Stdio},
    str::FromStr,
    string::FromUtf8Error,
    sync::{
      Arc, LazyLock, Mutex,
      atomic::{self, AtomicBool, AtomicUsize},
      mpsc,
    },
    thread::JoinHandle,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
  },
  strum::{EnumDiscriminants, EnumIter, IntoEnumIterator, IntoStaticStr},
  tempfile::TempDir,
  usized::{IntoU64, IntoU128, IntoUsize},
  vello::{kurbo, peniko},
  walkdir::WalkDir,
  wgpu::{
    AddressMode, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, Buffer,
    BufferBinding, BufferBindingType, BufferDescriptor, BufferUsages, COPY_BYTES_PER_ROW_ALIGNMENT,
    CommandEncoder, CommandEncoderDescriptor, DeviceDescriptor, Extent3d, Features, FilterMode,
    FragmentState, ImageSubresourceRange, Instance, Limits, LoadOp, MapMode, MemoryHints,
    MultisampleState, Operations, Origin3d, PipelineCompilationOptions, PipelineLayout,
    PipelineLayoutDescriptor, PowerPreference, PrimitiveState, Queue, RenderPass,
    RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor,
    RequestAdapterOptions, Sampler, SamplerBindingType, SamplerDescriptor, ShaderModuleDescriptor,
    ShaderSource, ShaderStages, StoreOp, Surface, SurfaceConfiguration, TexelCopyBufferInfo,
    TexelCopyBufferLayout, TexelCopyTextureInfo, Texture, TextureAspect, TextureDescriptor,
    TextureDimension, TextureFormat, TextureSampleType, TextureUsages, TextureView,
    TextureViewDescriptor, TextureViewDimension, Trace, VertexState,
  },
  winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{DeviceId, ElementState, Modifiers, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{Key, ModifiersState, NamedKey},
    window::{Fullscreen, Window, WindowAttributes, WindowId},
  },
};

mod alignment;
mod allocator;
mod analyzer;
mod app;
mod arguments;
mod array_string;
mod axis;
mod bindings;
mod bool_ext;
mod callback;
mod capture;
mod capture_thread;
mod color;
mod command;
mod command_entry;
mod command_ext;
mod commands;
mod composite_uniforms;
mod config;
mod controller;
mod counter;
mod error;
mod event;
mod field;
mod filter;
mod filter_uniforms;
mod fps;
mod frame;
mod functions;
mod generated;
mod history;
mod hub;
mod image;
mod image_format;
mod input;
mod interrupt;
mod into_stereo;
mod into_utf8_path;
mod maria;
mod message;
mod mirror;
mod mode;
mod options;
mod patch;
mod pipeline;
mod position;
mod present_mode;
mod preset;
mod press;
mod program;
mod recorder;
mod recorder_thread;
#[cfg(test)]
mod reference;
mod renderer;
mod resampler_ext;
mod resources;
mod scene;
mod score;
mod script;
mod shared;
mod sound;
mod sound_format;
mod space;
mod state;
mod stream_config_display;
mod subcommand;
mod tally;
mod tap;
mod target;
mod templates;
mod tempo;
mod texture_field;
mod tick;
mod tiling;
mod to_affine;
mod track;
mod transformation2;
mod transformation3;
mod uniforms;
mod window_attributes_ext;

const KIB: usize = 1 << 10;
const MIB: usize = KIB << 10;

const AUDIO: &str = "audio.wav";
const COLOR_CHANNELS: usize = 4;
const DEFAULT_BUFFER_SIZE: u32 = 128;
const DEFAULT_FPS: NonZeroU32 = NonZeroU32::new(60).unwrap();
const DEFAULT_RESOLUTION: NonZeroU32 = NonZeroU32::new(1024).unwrap();
const RECORDING: &str = "recording.mp4";
const TAU: f32 = f32::consts::TAU;
const TIME: u64 = 4;

type Result<T = (), E = Error> = std::result::Result<T, E>;
type SmallString = ArrayString<15>;
type TextureFieldKey = (
  Vector2<OrderedFloat<f32>>,
  OrderedFloat<f32>,
  ArrayString<15>,
  OrderedFloat<f32>,
);

type Mat1x2f = nalgebra::Matrix1x2<f32>;
type Mat2x3f = nalgebra::Matrix2x3<f32>;
type Mat3f = nalgebra::Matrix3<f32>;
type Mat3x2f = nalgebra::Matrix3x2<f32>;
type Mat3x4f = nalgebra::Matrix3x4<f32>;
type Mat4f = nalgebra::Matrix4<f32>;
type Size = nalgebra::Vector2<NonZeroU32>;
type Vec2f = nalgebra::Vector2<f32>;
type Vec3f = nalgebra::Vector3<f32>;
type Vec4f = nalgebra::Vector4<f32>;

fn main() {
  env_logger::Builder::new()
    .filter_level(log::LevelFilter::Warn)
    .parse_default_env()
    .init();

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
