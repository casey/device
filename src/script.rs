use super::*;

#[macro_export]
macro_rules! script {
  {
    $($beat:literal $($command:ident)+)*
  } => {
    &[
      $(
        ($beat, &[$($command,)*]),
      )*
    ]
  }
}

pub(crate) type Slice = &'static [(u64, &'static [CommandEntry])];

#[derive(Debug)]
pub(crate) struct Script {
  commands: BTreeMap<Position, Vec<CommandEntry>>,
}

impl Script {
  pub(crate) fn commands(&self) -> impl Iterator<Item = CommandEntry> {
    self.commands.values().flatten().copied()
  }

  pub(crate) fn tick(&self, tick: Tick) -> &[CommandEntry] {
    if !tick.advance() {
      return default();
    }

    let Some(position) = tick.position else {
      return default();
    };

    self
      .commands
      .get(&position)
      .map(Vec::as_slice)
      .unwrap_or_default()
  }
}

impl From<Slice> for Script {
  fn from(slice: Slice) -> Self {
    let mut commands = BTreeMap::<Position, Vec<CommandEntry>>::new();

    for (measure, line) in slice {
      let beat = measure.checked_sub(1).unwrap() * TIME;

      for (i, command) in line.iter().enumerate() {
        commands
          .entry(Position {
            index: (beat + i.into_u64()) * 4,
          })
          .or_default()
          .push(*command);
      }
    }

    Script { commands }
  }
}
