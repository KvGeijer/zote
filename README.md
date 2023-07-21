# The Zote programming language [WIP]

Zote is an imperative, dynamically typed scripting language with some inspiration from functional programming. Its name is inspired by the most [mighty](https://www.youtube.com/watch?v=j873sMpA16Q&ab_channel=BossFighter) and [wise](https://www.reddit.com/r/HollowKnight/comments/643usq/the_fiftyseven_precepts_of_zote/) character I know of. One of my goals is to solve [Advent of Code 2023](https://adventofcode.com/) in it, which acts as a motivator to improve it, and a guide of which type of things should be possible in it, and its standard library.

One of its core values is that you should be able to logically build your programs in the same direction you write. Take this Python code:
``` python
x = max(map(int, input.split("\n")))
```
This splits the input string, maps each line to an int, takes the max of all the lines, and assigns that to x. The last sentence describes how I think of that code, but it is not really in the same order as the code is written. Each new transformation (like `map` or `max`) is written to the left of all its arguments (except split which is nice and has the data to the left, and settings to the right, as it is a method).

In Zote you can instead write this:

``` python
input >> split("\n") >> map(int) >> max :>> x;
```

Here the data is piped through a series of transformations, from left to right, following the logical way of how I think of it. Languages like Rust and Java also achieve this by using methods constantly, but I wanted a more mathematical notation of functions, so settled on this. 

However, this all just desugarizes to:
```python
x := max(map(split("\n"), int));
```
which is also valid Zote. These two styles can be mixed depending on what mindset you are writing in. This might not be good for production code, but in my opinion makes for pleasant code to write.

## Development

In the beginning, I loosely followed the excellent book [Crafting Interpreters](craftinginterpreters.com) (kept the same development order, but tried to change most stuff up). Zote will not have any big innovations but instead combines ideas from [Rust](https://www.rust-lang.org/), [Julia](https://julialang.org/), [Python](https://www.python.org/) and [Noulith](https://github.com/betaveros/noulith), in no particular order (and of course from other languages as well).

Currently, the interpreter directly interprets the syntax tree, but it would be cool to compile it to bytecode which can be interpreted by a virtual machine. That will however take some work and will be left until the design of the language is rather stable (when it has nice solutions to all AoC 2022 problems)

## Benchmarks

There are some performance benchmarks [here](./benches), and using GitHub actions, you can track their approximate performance over time [here](https://kvgeijer.github.io/zote/dev/bench/).
