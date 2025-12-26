use {
  super::*,
  generated::*,
  position::{bar, bars, beat, quarter},
};

pub(crate) fn script() -> Script {
  let mut script = Script::default();

  // kick 1 3
  for bar in bars(1..39) {
    script.on(bar + beat(1), BLASTER_BLACK_AND_WHITE);
    script.on(bar + beat(3), BLASTER_BLACK_AND_WHITE);
  }

  script.on(bar(39), TOGGLE_FIT);

  // kick 1 3 with 2 4
  for bar in bars(39..123) {
    script.on(bar + beat(1), BLASTER);
    script.on(bar + beat(3), BLASTER);
  }

  // kick 2 4
  for bar in bars(39..71).chain(bars(79..123)) {
    script.on(bar + beat(2), BLASTER);
    script.on(bar + beat(4), BLASTER);
  }

  // lead
  for bar in bars(67..75).chain(bars(107..123)).step_by(2) {
    script.only(bar + beat(1), BLASTER);
    script.only(bar + beat(2), ZOOM_OUT);
    script.only(bar + beat(3), ZOOM_OUT);
    script.only(bar + beat(4), ZOOM_OUT);

    script.only(bar + beat(5), ZOOM_OUT);
    script.only(bar + beat(6), ZOOM_OUT);
    script.only(bar + beat(7), ZOOM_OUT);
    script.only(bar + beat(8), ZOOM_OUT);
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
    script.on(bar + beat(3) + quarter(1), PUSH_TOP);
    script.on(bar + beat(3) + quarter(3), PUSH_TOP);
    script.on(bar + beat(3) + quarter(4), PUSH_TOP);
    script.on(bar + beat(4) + quarter(1), PUSH_TOP);
    script.on(bar + beat(4) + quarter(2), PUSH_TOP);
    script.on(bar + beat(4) + quarter(4), PUSH_TOP);
    script.on(bar + beat(5) + quarter(1), PUSH_TOP);
    script.on(bar + beat(6) + quarter(3), PUSH_TOP);
    script.on(bar + beat(7) + quarter(1), PUSH_TOP);
    script.on(bar + beat(7) + quarter(3), PUSH_TOP);
    script.on(bar + beat(8) + quarter(3), PUSH_TOP);
    script.on(bar + beat(8) + quarter(4), PUSH_TOP);
  }

  // kick 1 3
  for bar in bars(151..159) {
    script.on(bar + beat(1), BLASTER_BLACK_AND_WHITE);
    script.on(bar + beat(3), BLASTER_BLACK_AND_WHITE);
  }

  // end
  script.on(bar(159) + beat(1), CLEAR);

  script
}
