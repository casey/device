set positional-arguments

watch +args='lcheck':
  cargo watch --clear --exec '{{args}}'

forbid:
  ./bin/forbid

ci: forbid
  cargo lclippy --workspace --all-targets -- --deny warnings
  cargo fmt --all -- --check
  cargo ltest --workspace -- --include-ignored

clippy: (watch 'lclippy --all-targets -- --deny warnings')

clean:
  rm -f baseline/*.test.png
  rm -f capture.png
  rm -f recording.mp4

baseline:
  #!/usr/bin/env bash
  rm baseline/*.test.png
  cargo ltest -- --ignored
  for image in baseline/*.png; do
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

hello:
  cargo run --release -- --song 'old generic boss' --program hello run

maria:
  cargo run --release -- --song 'total 4/13 maria' run

nobrain:
  cargo run --release -- --song 'no brain$' run

capture-hello:
  cargo run --release -- \
    --fps 60 \
    --program hello \
    --resolution 2048 \
    --song 'old generic boss' \
    --verbose \
    capture
