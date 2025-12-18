use super::*;

#[derive(Clone, Copy)]
pub(crate) struct Tiling {
  pub(crate) resolution: u32,
  pub(crate) size: u32,
}

impl Tiling {
  pub(crate) fn destination_offset(self, filter: u32) -> Vec2f {
    if self.size == 1 {
      return Vec2f::new(0.0, 0.0);
    }

    let col = filter % self.size;
    let row = filter / self.size;

    Vec2f::new(
      (self.resolution * col) as f32,
      (self.resolution * row) as f32,
    )
  }

  pub(crate) fn destination_read(self, filters: u32) -> bool {
    if self.size == 1 {
      filters.is_multiple_of(2)
    } else {
      true
    }
  }

  pub(crate) fn set_viewport(self, render_pass: &mut RenderPass, filter: u32) {
    if self.size == 1 {
      return;
    }

    let col = filter % self.size;
    let row = filter / self.size;

    render_pass.set_viewport(
      (col * self.resolution) as f32,
      (row * self.resolution) as f32,
      self.resolution as f32,
      self.resolution as f32,
      0.0,
      0.0,
    );
  }

  pub(crate) fn source_offset(self, filter: u32) -> Vec2f {
    if self.size == 1 {
      return Vec2f::new(0.0, 0.0);
    }

    let Some(filter) = filter.checked_sub(1) else {
      return Vec2f::new(0.0, 0.0);
    };

    let row = filter / self.size;
    let col = filter % self.size;

    Vec2f::new(col as f32 / self.size as f32, row as f32 / self.size as f32)
  }

  pub(crate) fn source_read(self, filters: u32) -> bool {
    if self.size == 1 {
      !filters.is_multiple_of(2)
    } else {
      true
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn tiling() {
    #[track_caller]
    fn case(filter: u32, offset: Vec2f, source_offset: Vec2f) {
      let tiling = Tiling {
        resolution: 100,
        size: 2,
      };

      assert_eq!(tiling.source_offset(filter), source_offset,);
      assert_eq!(tiling.destination_offset(filter), offset,);
    }

    case(0, vector!(0.0, 0.0), vector!(0.0, 0.0));
    case(1, vector!(100.0, 0.0), vector!(0.0, 0.0));
    case(2, vector!(0.0, 100.0), vector!(0.5, 0.0));
    case(3, vector!(100.0, 100.0), vector!(0.0, 0.5));
    case(4, vector!(0.0, 200.0), vector!(0.5, 0.5));
  }
}
