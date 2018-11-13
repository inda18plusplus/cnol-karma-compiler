#!/usr/bin/env bash


# Build benchmark tool
cargo build --release;
BENCH=$(readlink -f ./target/release/bench);


# Build interpreter
cd ../interpreter;

cargo build --release;
INTERPRETER=$(readlink -f ./target/release/karmai);

# Load source
SOURCE=$(readlink -f $1);

# Run benchmark with input
echo -n "$2 => "

echo $2 | $BENCH "$INTERPRETER $SOURCE"


