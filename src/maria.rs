use {super::*, generated::*};

fn bar(bar: u64) -> Position {
  Position::from_bar(bar.checked_sub(1).unwrap())
}

fn bars(range: Range<u64>) -> impl Iterator<Item = Position> {
  range.map(|bar| Position::from_bar(bar.checked_sub(1).unwrap()))
}

fn beat(beat: u64) -> Position {
  Position::from_beat(beat.checked_sub(1).unwrap())
}

fn quarter(quarter: u64) -> Position {
  Position::from_quarter(quarter.checked_sub(1).unwrap())
}

pub(crate) fn script() -> Script {
  let mut script = Script::default();

  // kick 1 3
  for bar in bars(1..123).chain(bars(151..159)) {
    script.on(bar + beat(1), BLASTER);
    script.on(bar + beat(3), BLASTER);
  }

  // kick 2 4
  for bar in bars(39..71).chain(bars(79..123)) {
    script.on(bar + beat(2), BLASTER);
    script.on(bar + beat(4), BLASTER);
  }

  // snare 2 4
  for bar in bars(53..68).chain(bars(83..123)).chain(bars(131..147)) {
    script.only(bar + beat(2), ZOOM_OUT);
    script.only(bar + beat(4), ZOOM_OUT);
  }

  // break kick
  for bar in bars(123..151).step_by(2) {
    script.clear(
      Bound::Included(bar),
      Bound::Excluded(bar + Position::from_bar(2)),
    );
    script.on(bar + beat(1) + quarter(1), CLEAR);
    script.on(bar + beat(1) + quarter(1), PUSH_TOP);
    script.on(bar + beat(1) + quarter(2), PUSH_TOP);
    script.on(bar + beat(1) + quarter(3), PUSH_TOP);
    script.on(bar + beat(2) + quarter(3), PUSH_TOP);
    script.on(bar + beat(3) + quarter(3), PUSH_TOP);
    script.on(bar + beat(3) + quarter(4), PUSH_TOP);
    script.on(bar + beat(4) + quarter(1), PUSH_TOP);
    script.on(bar + beat(4) + quarter(2), PUSH_TOP);
    script.on(bar + beat(5) + quarter(1), PUSH_TOP);
    script.on(bar + beat(6) + quarter(3), PUSH_TOP);
    script.on(bar + beat(7) + quarter(3), PUSH_TOP);
    script.on(bar + beat(8) + quarter(2), PUSH_TOP);
  }

  // end
  script.on(bar(159) + beat(1), UNWIND);

  script
}
