use {
  self::{
    analyzer::Analyzer, app::App, arguments::Arguments, bindings::Bindings, config::Config,
    controller::Controller, error::Error, event::Event, field::Field, filter::Filter,
    format::Format, fps::Fps, frame::Frame, hub::Hub, image::Image, input::Input,
    into_u64::IntoU64, into_u128::IntoU128, into_usize::IntoUsize, into_utf8_path::IntoUtf8Path,
    message::Message, options::Options, parameter::Parameter, patch::Patch, program::Program,
    recorder::Recorder, renderer::Renderer, scene::Scene, score::Score, shared::Shared,
    sound::Sound, state::State, subcommand::Subcommand, tally::Tally, tap::Tap, target::Target,
    templates::ShaderWgsl, text::Text, tiling::Tiling, uniforms::Uniforms,
  },
  boilerplate::Boilerplate,
  camino::{Utf8Path, Utf8PathBuf},
  clap::{Parser, ValueEnum},
  hound::{WavSpec, WavWriter},
  indicatif::{ProgressBar, ProgressStyle},
  nalgebra::Vector2,
  parley::{FontContext, LayoutContext},
  regex::{Regex, RegexBuilder},
  rodio::{
    Decoder, OutputStream, Sink, Source,
    cpal::{
      self, SampleFormat, StreamConfig, SupportedBufferSize, SupportedStreamConfig,
      SupportedStreamConfigRange,
      traits::{DeviceTrait, HostTrait, StreamTrait},
    },
    source::UniformSourceIterator,
  },
  rustfft::{FftPlanner, num_complex::Complex},
  serde::Deserialize,
  snafu::{ErrorCompat, IntoError, OptionExt, ResultExt, Snafu},
  std::{
    backtrace::{Backtrace, BacktraceStatus},
    borrow::Cow,
    collections::VecDeque,
    env, f32,
    fmt::{self, Display, Formatter},
    fs::{self, File},
    io::{self, BufReader, BufWriter, Write},
    iter, mem,
    num::NonZeroU32,
    ops::{Add, AddAssign, SubAssign},
    process::{self, Command, ExitStatus, Stdio},
    str::FromStr,
    sync::{Arc, Mutex, mpsc},
    thread,
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
mod subcommand;
mod tally;
mod tap;
mod target;
mod templates;
mod text;
mod tiling;
mod uniforms;

const KIB: usize = 1 << 10;
const MIB: usize = KIB << 10;

const AUDIO: &str = "audio.wav";
const COLOR_CHANNELS: u32 = 4;
const DEFAULT_BUFFER_SIZE: u32 = 128;
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

fn display<'a, T: Display + 'a>(t: T) -> Box<dyn Display + 'a> {
  Box::new(t)
}

fn invert_color() -> Mat4f {
  Mat4f::from_diagonal(&Vec4f::new(-1.0, -1.0, -1.0, 1.0))
}

fn pad(i: usize, alignment: usize) -> usize {
  assert!(alignment.is_power_of_two());
  (i + alignment - 1) & !(alignment - 1)
}

fn open_audio_file(path: &Utf8Path) -> Result<Decoder<BufReader<File>>> {
  let file = File::open(path).context(error::FilesystemIo { path })?;
  let reader = BufReader::new(file);
  let source = Decoder::new(reader).context(error::DecoderOpen { path })?;
  Ok(source)
}

fn open_audio_file_fundsp(path: &Utf8Path) -> Result<fundsp::wave::Wave> {
  use {
    fundsp::wave::{Wave, WavePlayer},
    rubato::{FftFixedIn, Resampler},
  };

  let mut wave = Wave::load(path).unwrap();

  dbg!(wave.channels());
  dbg!(wave.sample_rate());

  let mut resampler = rubato::FftFixedIn::<f32>::new(
    wave.sample_rate() as usize,
    48_000,
    1024,
    2,
    wave.channels(),
  )
  .unwrap();

  // (0..wave.channels()).map(|channel| wave.channel(channel).chunks(1024))
  // iterater of iterator of chunks

  let mut output_buffer = resampler.output_buffer_allocate(true);
  let mut input_buffer = resampler.input_buffer_allocate(true);

  let mut output_channels = vec![Vec::<f32>::new(); wave.channels()];

  // todo:
  // - deal with partial chunks
  // - deal with delay
  // - deal with there still being chunks in the resampler

  for chunk in 0.. {
    let start = chunk * 1024;
    let end = start + 1024;

    if wave.len() == start {
      break;
    } else if wave.len() < end {
      let samples = wave.len() - start;

      for channel in 0..wave.channels() {
        input_buffer[channel][0..samples]
          .copy_from_slice(&wave.channel(channel)[start..start + samples]);
        input_buffer[channel].truncate(samples);
      }

      let (input, output) = resampler
        .process_partial_into_buffer(Some(&input_buffer), &mut output_buffer, None)
        .unwrap();

      for channel in 0..wave.channels() {
        output_channels[channel].extend(&output_buffer[channel][0..output]);
      }

      break;
    } else {
      for channel in 0..wave.channels() {
        input_buffer[channel][0..1024].copy_from_slice(&wave.channel(channel)[start..end]);
      }

      let (input, output) = resampler
        .process_into_buffer(&input_buffer, &mut output_buffer, None)
        .unwrap();

      assert_eq!(input, 1024);

      for channel in 0..wave.channels() {
        output_channels[channel].extend(&output_buffer[channel][0..output]);
      }
    }
  }

  let mut output_wave = Wave::new(0, 48_000.0);

  for channel in 0..wave.channels() {
    output_wave.push_channel(&output_channels[channel]);
  }

  Ok(output_wave)
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
