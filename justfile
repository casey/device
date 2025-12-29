set positional-arguments

alias ref := reference

watch +args='lcheck --all --all-targets':
  cargo watch --clear --exec '{{args}}'

run *args:
  cargo run --release -- "$@"

forbid:
  ./bin/forbid

ci: forbid
  cargo lclippy --workspace --all-targets -- --deny warnings
  cargo fmt --all -- --check
  cargo ltest --workspace -- --include-ignored

clippy: (watch 'lclippy --all-targets -- --deny warnings')

test: (watch 'ltest')

clean:
  rm -f reference/*.test.png
  rm -f capture.png
  rm -f recording.mp4

reference *args:
  #!/usr/bin/env bash
  rm reference/*.test.png
  cargo ltest -- --skip renderer:: --ignored "$@"
  for image in reference/*.png; do
    if [[ $image == *.test.png ]]; then
      continue
    fi
    if [[ ! -e ${image%.*}.test.png ]]; then
      echo "stale image: $image"
      exit 1
    fi
  done
  status=$?
  exit $status

accept-reference:
  #!/usr/bin/env bash
  for image in reference/*.png; do
    if [[ $image == *.test.png ]]; then
      continue
    fi
    mv ${image%.*}.test.png $image
  done

outdated:
  cargo outdated --root-deps-only --workspace

unused:
  cargo +nightly udeps --workspace

doc:
  cargo doc --workspace --open

bindings:
  cargo run bindings

commands:
  cargo run commands

program program:
  cargo run --release -- --program {{program}}

shader:
  cargo run shader

hello: (program "hello")
busy: (program "busy")
noise: (program "noise")
expo: (program "expo")
transit: (program "transit")
radio: (program "radio")

maria seed:
  cargo run \
    --release \
    -- \
    --program maria \
    --seed {{ seed }} \
    run

all-night:
  cargo run \
    --release \
    -- \
    --width 3840 \
    --height 2160 \
    --program all-night \
    run

maria-variations:
  #!/usr/bin/env bash
  cargo build --release
  for i in {0..100}; do
    ./target/release/device \
      --width 3840 \
      --height 2160 \
      --program maria \
      --seed $i \
      --verbose \
      capture \
      --stem $i
  done

curtains:
  cargo run --release -- --song 'curtains closing' run

nobrain:
  cargo run --release -- --song 'no brain$' run

preset preset:
  cargo run --release -- \
    --program grid \
    --mute \
    --preset {{preset}} \
    run

capture program:
  cargo run --release -- \
    --program {{program}} \
    --resolution 2048 \
    --verbose \
    capture

capture-hello-landscape:
  cargo run \
    --release \
    -- \
    --width 3840 \
    --height 2160 \
    --program hello-landscape \
    --verbose \
    capture

capture-hello: (capture "hello")
capture-busy: (capture "busy")
capture-noise: (capture "noise")
capture-expo: (capture "expo")
capture-transit: (capture "transit")
capture-radio: (capture "radio")

capture-maria:
  cargo run \
    --release \
    -- \
    --width 3840 \
    --height 2160 \
    --program maria \
    --seed 1024 \
    --verbose \
    capture

capture-all-night:
  cargo run \
    --release \
    -- \
    --width 3840 \
    --height 2160 \
    --program all-night \
    --verbose \
    capture

capture-suplex:
  cargo run \
    --release \
    -- \
    --width 3840 \
    --height 2160 \
    --program suplex \
    --verbose \
    capture

record-curtains:
  cargo run --release -- \
    --fps 60 \
    --resolution 2048 \
    --song 'curtains closing' \
    --verbose \
    run \
    --record

last-resort:
  git clone git@github.com:unicode-org/last-resort-font.git static/last-resort-font

ucd:
  mkdir -p tmp
  curl https://www.unicode.org/Public/17.0.0/ucd/UCD.zip -O --output-dir tmp
  curl https://www.unicode.org/Public/17.0.0/ucd/Unihan.zip -O --output-dir tmp
  rm -rf static/{ucd,unihan}
  unzip tmp/UCD.zip -d static/ucd
  unzip tmp/Unihan.zip -d static/unihan
