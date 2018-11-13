# Karma Interpreter and Compiler

Based on description provided by [Esolang, the esoteric programming languages wiki](https://esolangs.org/wiki/Karma).

Some modifications were made in order to make compilation and authoring code easier:
- Anything after a non-command character will be ignored. Blank lines still terminate execution.
- Spaces and tabs will be ignored within statements.
- Use a 64-bit integer instead of an 8-bit byte as standard data type.


## Examples

### Hello, world!
```
98*:55+\*1+:96*\+\:\:3+:56+4*:48*:126*55+*-\:80-+\:3+\:60-+:55+\*:48*1+:
```

### Fibonacci sequence
```
0,
68*?-,+55+*<
\10-}>{#!@,}55+>[{]@',
#[55+]/}1}01,
,\;48*:\[+}]{<
{1+\>[}]!@';#{{##
```

### Sum of natural numbers
```
0,
68*?-,+55+*<
\10-}>{#!@,}55+>[{]@',
#[55+]/}0}0,
{1+\>@,\}+<
{##;
```


## Benchmarks

Benchmarks were run using `bench/bench_interpreter.sh` and `bench/bench_compiler.sh` for the interpreter and compiler, respectively.

| Benchmark                | File              | Input         | Interpreter (seconds)  | Compiler (seconds) |
| ------------------------ | ----------------- | ------------- | ---------------------- | -------------      |
| Sum of natural numbers   | `karma/sum.kar`   | `123456789`   | 13.683850786           | 5.650353131        |



## TODO

- [ ] Make all builder functions safe.
- [X] Jump directly from section to section.
- [X] Reference jump tables across sequences.
- [X] Implement the double ended queue.
- [X] Implement all instructions in the instruction builder.
- [X] Implement dynamic stack and deque
- [X] Build benchmark tool
- [ ] Write code to benchmark
- [ ] End world hunger.

