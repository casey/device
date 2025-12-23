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

hello: (program "hello")
busy: (program "busy")
noise: (program "noise")
expo: (program "expo")
transit: (program "transit")
radio: (program "radio")

blaster seed:
  cargo run \
    --release \
    -- \
    --program blaster \
    --seed {{ seed }} \
    run

curtains:
  cargo run --release -- --song 'curtains closing' run

maria:
  cargo run --release -- --song 'total 4/13 maria' run

nobrain:
  cargo run --release -- --song 'no brain$' run

preset preset:
  cargo run --release -- \
    --program blaster \
    --mute \
    --preset {{preset}} \
    run

capture program:
  cargo run --release -- \
    --program {{program}} \
    --resolution 2048 \
    --verbose \
    capture

capture-hello: (capture "hello")
capture-busy: (capture "busy")
capture-noise: (capture "noise")
capture-expo: (capture "expo")
capture-transit: (capture "transit")
capture-radio: (capture "radio")

capture-blaster:
  cargo run \
    --release \
    -- \
    --width 3840 \
    --height 2160 \
    --fps 60 \
    --program blaster \
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
