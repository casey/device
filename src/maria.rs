use {super::*, generated::*};

struct Cue {
  event: &'static str,
  op: Op,
  positions: Vec<Position>,
}

enum Op {
  Start,
  Stop,
  Once,
}

macro_rules! op {
  (+) => {
    Op::Start
  };
  (-) => {
    Op::Stop
  };
  (=) => {
    Op::Once
  };
}

fn position(literal: f32) -> Position {
  let beat = (literal.trunc() as u64).checked_sub(1).unwrap();
  let quarter = if literal.trunc() == literal {
    0
  } else {
    ((literal.fract() * 10.0).trunc() as u64)
      .checked_sub(1)
      .unwrap()
  };

  Position::from_quarter(beat * 4 + quarter)
}

macro_rules! cues {
  {
    $(
      $bar:literal $op:tt $event:ident $($position:literal)+;
    )+
  } => {
    {
      let mut cues = BTreeMap::<Position, Vec<Cue>>::new();

      $(
        let positions = vec![$(position($position as f32),)+];

        let bar: u64 = $bar;

        cues.entry(Position::from_bar(bar.checked_sub(1).unwrap())).or_default().push(
          Cue {
            event: stringify!($event),
            op: op!($op),
            positions,
          }
        );
      )*

      cues
    }
  }
}

pub(crate) fn events() -> Script {
  // todo:
  // - break kicks are too boring each one should add a single layer
  // - get the kick pickups and snare fills

  let cues = cues! {
     1  +kick  1 3;
     39 +kick  2 4;
     53 +snare 2 4;
     68 -snare 2 4;
     71 -kick  2 4;
     79 +kick  2 4;
     83 +snare 2 4;
    123 -snare 2 4;
    123 -kick  1 2 3 4;
    // 123 +kick  1.1 1.2 1.3 2.3 3.3 3.4 4.1 4.2 5.1 6.3 7.3 8.2;
    131 +snare 2 4;
    147 -snare 2 4;
    // 151 -kick  1.1 1.2 1.3 2.3 3.3 3.4 4.1 4.2 5.1 6.3 7.3 8.2;
    151 +kick  1 3;
    159 -kick  1 3;
    159 =fade  1;
  };

  let end = cues.last_key_value().unwrap().0;

  let mut events = BTreeMap::<Position, BTreeSet<&'static str>>::new();

  let mut active = BTreeMap::<Position, BTreeSet<&'static str>>::new();

  for i in 0..=end.quarters() {
    let current = Position::from_quarter(i);

    if let Some(cues) = cues.get(&current) {
      for cue in cues {
        match cue.op {
          Op::Start => {
            for position in &cue.positions {
              active.entry(*position).or_default().insert(cue.event);
            }
          }
          Op::Stop => {
            for position in &cue.positions {
              assert!(active.entry(*position).or_default().remove(cue.event));
            }
          }
          Op::Once => {
            for position in &cue.positions {
              events
                .entry(current + *position)
                .or_default()
                .insert(cue.event);
            }
          }
        }
      }
    };

    if current.quarter() == 0 {
      for (position, commands) in &active {
        let entry = events.entry(current + *position).or_default();

        for command in commands {
          entry.insert(command);
        }
      }
    }
  }

  let mut commands = BTreeMap::<Position, Vec<CommandEntry>>::new();

  for (position, events) in events {
    if events.is_empty() {
      continue;
    }

    let mut entry = commands.entry(position).or_default();
    match events.iter().copied().collect::<Vec<&str>>().as_slice() {
      ["kick"] => entry.push(BLASTER),
      ["snare"] => entry.push(ZOOM_OUT),
      ["kick", "snare"] => entry.push(ZOOM_OUT),
      ["fade"] => entry.push(UNWIND),
      _ => panic!("{events:?}"),
    }
  }

  Script { commands }
}

pub(crate) const SCRIPT: script::Slice = script! (
  3 BLASTER
  15 BLASTER
  19 BLASTER
  23 BLASTER
  27 BLASTER
  31 BLASTER
  35 BLASTER
  39 BLASTER
  43 BLASTER
  47 ZOOM_OUT
  51 BLASTER
  55 ZOOM_OUT
  59 ZOOM_OUT
  63 ZOOM_OUT
  67 BLASTER ZOOM_OUT ZOOM_OUT ZOOM_OUT
  68 BLASTER ZOOM_OUT ZOOM_OUT ZOOM_OUT
  69 BLASTER ZOOM_OUT ZOOM_OUT ZOOM_OUT
  70 BLASTER ZOOM_OUT ZOOM_OUT ZOOM_OUT
  71 BLASTER ZOOM_OUT ZOOM_OUT ZOOM_OUT
  72 BLASTER ZOOM_OUT ZOOM_OUT ZOOM_OUT
  73 BLASTER ZOOM_OUT ZOOM_OUT ZOOM_OUT
  74 BLASTER ZOOM_OUT ZOOM_OUT ZOOM_OUT
  75 BLASTER
  79 BLASTER
  83 BLASTER
  87 BLASTER
  91 BLASTER
  95 BLASTER
  107 BLASTER ZOOM_OUT ZOOM_OUT ZOOM_OUT
  108 BLASTER ZOOM_OUT ZOOM_OUT ZOOM_OUT
  109 BLASTER ZOOM_OUT ZOOM_OUT ZOOM_OUT
  110 BLASTER ZOOM_OUT ZOOM_OUT ZOOM_OUT
  111 BLASTER ZOOM_OUT ZOOM_OUT ZOOM_OUT
  112 BLASTER ZOOM_OUT ZOOM_OUT ZOOM_OUT
  113 BLASTER ZOOM_OUT ZOOM_OUT ZOOM_OUT
  114 BLASTER ZOOM_OUT ZOOM_OUT ZOOM_OUT
  115 BLASTER ZOOM_OUT ZOOM_OUT ZOOM_OUT
  116 BLASTER ZOOM_OUT ZOOM_OUT ZOOM_OUT
  117 BLASTER ZOOM_OUT ZOOM_OUT ZOOM_OUT
  118 BLASTER ZOOM_OUT ZOOM_OUT ZOOM_OUT
  119 BLASTER ZOOM_OUT ZOOM_OUT ZOOM_OUT
  120 BLASTER ZOOM_OUT ZOOM_OUT ZOOM_OUT
  121 BLASTER ZOOM_OUT ZOOM_OUT ZOOM_OUT
  122 BLASTER ZOOM_OUT ZOOM_OUT ZOOM_OUT
  123 BLASTER
  127 BLASTER
  131 BLASTER
  135 BLASTER
  139 BLASTER
  143 BLASTER
  147 BLASTER
  151 BLASTER
  155 BLASTER
  159 UNWIND
);

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn foo() {
    eprintln!("{}", events());
    panic!();
  }
}
