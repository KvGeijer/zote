# The Zote programming language

[WIP, expect many breaking changes and no documentation]

Zote is an imperative, dynamically typed scripting language with some support for functional programming. Its name is inspired by the most [mighty](https://www.youtube.com/watch?v=j873sMpA16Q&ab_channel=BossFighter) and [wise](https://www.reddit.com/r/HollowKnight/comments/643usq/the_fiftyseven_precepts_of_zote/) character I know of. One of the goals was to solve [Advent of Code](https://adventofcode.com/) in it, which acts as a motivator to improve it, and a clear goal of which type of things should be possible in it, and its standard library.

One of its core values is that you should be able to logically build your programs in the same direction you write. Take this Python code
``` python
x = max(map(int, input.split("\n")))
```
This splits the input sting, maps each line to an int, takes the max of all the lines, and assigns that to x. The last sentence describes how I think of that code, but it is not really in the same order as the code is written. Each new transformation (like map or max) comes to the left of the last stages output. Furthermore, you have to enclose each stage in parenthesis, requiring you to hop back and forth to write this. Finally, the assignment to x is mostly fine, but sometimes you start with the transformations, and at some point decide to save the value in some variable for further use, and then you have to go back and write the assignment. In Zote you can instead write something like

``` python
input >> split("\n") >> map(int) >> max >>: x;
```

For me, this more closely follows how I think. Especially when thinking in terms of transformations. Rust and Java achieve this by using methods constantly, but I wanted a more mathematical notation of functions, so settled on this. However, this all just desugarizes to
```python
var x = max(map(split("\n"), int));
```
which is also valid Zote (or at least will be once the standard library is larger). These two styles can then be mixed depending on what mindset you are writing in.

## Development

In the beginning, I loosely followed the excellent book [Crafting Interpreters](craftinginterpreters.com) (kept the same development order, but tried to change most stuff up). Zote will not have any big innovations but instead combines ideas from [Rust](https://www.rust-lang.org/), [Julia](https://julialang.org/), [Python](https://www.python.org/) and [Noulith](https://github.com/betaveros/noulith), in no particular order (and of course from other languages as well).

Initially, the interpreter will directly interpret the syntax tree, but it would be cool to compile it to bytecode which can be interpreted by a virtual machine. That will however take some work and will be left until the language is rather stable.
