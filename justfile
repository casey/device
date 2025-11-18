set positional-arguments

watch +args='lcheck':
  cargo watch --clear --exec '{{args}}'

run *args:
  cargo build --release
  ./target/release/x "$@"

forbid:
  ./bin/forbid

ci: forbid
  cargo lclippy --workspace --all-targets -- --deny warnings
  cargo fmt --all -- --check
  cargo ltest --workspace -- --include-ignored

clippy: (watch 'lclippy --all-targets -- --deny warnings')

clean:
  rm baseline/*.test.png

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
  cargo run --release -- --song 'old generic boss' --program hello

maria:
  cargo run --release -- --song 'total 4/13 maria'

nobrain:
  cargo run --release -- --song 'no brain$'
