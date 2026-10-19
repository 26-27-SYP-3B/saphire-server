default: run

run:
    cargo run

run-release:
    cargo run --release

build:
    cargo build --release

clean:
    rm -rf target