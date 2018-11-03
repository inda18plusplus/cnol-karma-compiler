# Karma Interpreter and Compiler

Based on description provided by [Esolang, the esoteric programming languages wiki](https://esolangs.org/wiki/Karma).

Some modifications had to be made in order to make compilation and authoring code easier:
- Anything after a non-command character will be ignored. Blank lines still terminate execution.
- Spaces and tabs will be ignored within statements.


## Examples

**Hello, world**
```
98*:55+\*1+:96*\+\:\:3+:56+4*:48*:126*55+*-\:80-+\:3+\:60-+:55+\*:48*1+:
```

**Fibonacci sequence**
```
0,
68*?-,+55+*<
\10-}>{#!@,}55+>[{]@',
#[55+]/}1}01,
,\;48*:\[+}]{<
{1+\>[}]!@';#{{##
```

**Sum of natural numbers**
```
0,
68*?-,+55+*<
\10-}>{#!@,}55+>[{]@',
#[55+]/}0}0,
{1+\>@,\}+<
{##;
```


## Benchmarks

| Benchmark              | File            | Input       | Interpreter (seconds) | Compiler (seconds) |
|------------------------|-----------------|-------------|-----------------------|--------------------|
| Sum of natural numbers | `karma/sum.kar` | `123456789` | 12.4503               | -                  |
|------------------------|-----------------|-------------|-----------------------|--------------------|

