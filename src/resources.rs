use super::*;

pub(crate) struct Resources {
  pub(crate) dummy_field_texture: BindGroup,
  pub(crate) field_textures: HashMap<&'static str, BindGroup>,
  pub(crate) overlay_bind_group: BindGroup,
  pub(crate) overlay_view: TextureView,
  pub(crate) pool: Arc<Mutex<Vec<Buffer>>>,
  pub(crate) targets: [Target; 2],
  pub(crate) tiling_bind_group: BindGroup,
  pub(crate) tiling_view: TextureView,
}
