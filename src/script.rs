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

#[derive(Debug, Default)]
pub(crate) struct Script {
  pub(crate) commands: BTreeMap<Position, Vec<CommandEntry>>,
}

impl Script {
  pub(crate) fn clear(&mut self, start: Bound<Position>, end: Bound<Position>) {
    let keys = self
      .commands
      .range((start, end))
      .map(|(position, _commands)| *position)
      .collect::<Vec<Position>>();

    for key in keys {
      self.commands.remove(&key).unwrap();
    }
  }

  pub(crate) fn commands(&self) -> impl Iterator<Item = CommandEntry> {
    self.commands.values().flatten().copied()
  }

  pub(crate) fn on(&mut self, position: Position, entry: CommandEntry) {
    self.commands.entry(position).or_default().push(entry);
  }

  pub(crate) fn only(&mut self, position: Position, entry: CommandEntry) {
    self
      .commands
      .entry(position)
      .and_modify(Vec::clear)
      .or_default()
      .push(entry);
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
          .entry(Position::from_beat(beat + i.into_u64()))
          .or_default()
          .push(*command);
      }
    }

    Script { commands }
  }
}

impl Display for Script {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    for (position, commands) in &self.commands {
      write!(f, "{position}")?;
      for command in commands {
        write!(f, " {}", command.0)?;
      }
      writeln!(f)?;
    }
    Ok(())
  }
}
