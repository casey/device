set positional-arguments

alias ref := reference

watch +args='lcheck':
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

program program:
  cargo run --release -- --program {{program}}

hello: (program "hello")
busy: (program "busy")
noise: (program "noise")
expo: (program "expo")
transit: (program "transit")

curtains:
  cargo run --release -- --song 'curtains closing' run

maria:
  cargo run --release -- --song 'total 4/13 maria' run

blaster:
  cargo run --release -- --format bgra8unorm --song 'total 4/13 maria' --db -15 --preset test

preset preset:
  cargo run --release -- \
    --format bgra8unorm \
    --song 'total 4/13 maria' \
    --mute \
    --preset test \
    --preset {{preset}} \
    run

nobrain:
  cargo run --release -- --song 'no brain$' run

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

record-curtains:
  cargo run --release -- \
    --fps 60 \
    --resolution 2048 \
    --song 'curtains closing' \
    --verbose \
    run \
    --record
