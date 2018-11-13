#!/usr/bin/env bash


# Build benchmark tool
cargo build --release;
BENCH=$(readlink -f ./target/release/bench);



# Load source
SOURCE=$(readlink -f $1);


cd ../compiler

echo Compiling source...
./build_executable.sh $SOURCE
EXECUTABLE=$(readlink -f ./out/a.out)

# Run benchmark with input
echo -n "$2 => "

echo $2 | $BENCH "$EXECUTABLE"


