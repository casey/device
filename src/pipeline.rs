use super::*;

pub(crate) struct Pipeline {
  pub(crate) bind_group_layout: BindGroupLayout,
  pub(crate) pipeline_layout: PipelineLayout,
  pub(crate) render_pipeline: RenderPipeline,
  pub(crate) uniform_buffer: Buffer,
  pub(crate) uniform_buffer_size: u32,
  pub(crate) uniform_buffer_stride: u32,
}
