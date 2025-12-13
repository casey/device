use super::*;

pub(crate) trait WindowAttributesExt {
  fn with_platform_attributes(self) -> Self;
}

#[cfg(target_os = "macos")]
impl WindowAttributesExt for WindowAttributes {
  fn with_platform_attributes(self) -> Self {
    use winit::platform::macos::WindowAttributesExtMacOS;
    self
      .with_fullsize_content_view(true)
      .with_title_hidden(true)
      .with_titlebar_buttons_hidden(true)
      .with_titlebar_transparent(true)
  }
}

#[cfg(not(target_os = "macos"))]
impl WindowAttributesExt for WindowAttributes {
  fn with_platform_attributes(self) -> Self {
    self
  }
}
