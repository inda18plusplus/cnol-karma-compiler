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


## Current optimizations

Both the interpreter and compiler are really dumb. They push, pop, remove and
insert values all the time, even though most of them could be avoided. For
example, in the following snippet we push the number 7 to the stack, pop it and
insert it to the front of the deque: 
```
    7}
```
In this simple example the compiler has to perform 3 function calls (`push`,
`pop` and `insert_front`). It is possible to remove the `push` and `pop`
altogether and just insert the number 7 straight away.

The compiler does not handle constant expressions very well either. The Karma
specification states that every intermediate in a computation should be
pushed to the stack. As a result we end up with 3 `push`es and 2 `pop`s in the
following snippet:
```
    12+
```
The LLVM IR optimizer is not aware that `push`ing and then `pop`ing technically
does not affect state in any way. In it's current form the Karma parser has an
optional optimization pass which reduces the simplest case in the form `12+` 
directly to `3`.


## Future optimizations

In theory it is possible to track all constant values as they travel through the
source tree and remove even more `push` and `pop` calls. Currently the following
snippet will not be reduced any further:
```
    5}?{1+
```
while it could be reduced to the following:
```
    ?6
```


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

