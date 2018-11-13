
# Bench

Benchmarks utilities for the karma interpreter `bench_interpreter.sh` and the
karma compiler `bench_compiler.sh`.


## Usage

`bench_interpreter.sh SOURCE_FILE STDIN`
`bench_compiler.sh SOURCE_FILE STDIN`

`STDIN` is piped to the interpreter's/executable's standard input.

### Output

Output is in the form:
```
STDIN => STDOUT
seconds: <seconds>
```

Where `STDOUT` is the output of the interpreter/executable and `<seconds>` is
the number of seconds it took for the program to complete.
