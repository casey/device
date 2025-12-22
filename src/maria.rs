use {super::*, generated::*};

struct Cue {
  name: &'static str,
  op: &'static str,
  beats: Vec<Vec<f32>>,
}

macro_rules! foo {
  {
    $(
      $bar:literal $op:tt $event:ident $($beat:literal)+ $(/ $($more:literal)+)* ;
    )*
  } => {
    fn foo() -> BTreeMap<u64, Vec<Cue>> {
      #[allow(unused_mut)]
      let mut cues = BTreeMap::<u64, Vec<Cue>>::new();

      $(
        #[allow(unused_mut)]
        let mut beats = vec![vec![$($beat as f32,)+]];

        $(
          beats.push(
            vec![$($more as f32,)+]
          );
        )*

        cues.entry($bar).or_default().push(
          Cue {
            name: stringify!($event),
            op: stringify!($op),
            beats,
          }
        );
      )*

      cues
    }
  }
}

foo! {
    1 +kick  1 3;
   39 +kick  2 4;
   53 +snare 2 4;
   68 -snare 2 4;
   71 -kick  2 4;
   79 +kick  2 4;
   83 +snare 2 4;
  123 -snare 2 4;
  123 -kick  1 2 3 4;
  123 +kick  1.1 1.2 1.3 2.3 3.3 3.4 4.1 4.2 / 1.1 2.3 3.3 4.2;
  131 +snare 2 4;
  147 -snare 2 4;
  151 -kick  1.1 1.2 1.3 2.3 3.3 3.4 4.1 4.2 / 1.1 2.3 3.3 4.2;
  151 +kick  1 3;
  159 -kick  1 3;
  159 =fade  1;
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
