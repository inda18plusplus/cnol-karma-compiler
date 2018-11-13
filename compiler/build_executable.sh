#!/usr/bin/env bash

if [ -z $1 ]
then
    echo "no input ir given";
else
    mkdir -p out;
    cargo run --release -- $1 > out/out.ll;
    opt -O3 -S out/out.ll -o out/opt.ll;
    llc -filetype=obj out/opt.ll;
    gcc -no-pie out/opt.o -o out/a.out;
fi

