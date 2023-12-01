# The Zote programming language

Zote is an imperative, dynamically typed scripting language with inspiration from functional programming. A target of mine is to solve [Advent of Code 2023](https://adventofcode.com/) with zote's virtual machine implementation, and the whole language is designed to be pleasant to use for that type of small problems. For example, it should be easy and fast to write scripts of up to a couple of hundred lines, but easy maintenance is not a priority.

At the moment, Zote has two working interpreters, split as two binaries.
- The `ast-zote` binary is a simple interpreter that directly traverses the syntax tree during runtime. In [aoc-2022/ast-solutions](./aoc-2022/ast-solutions) there are working solutions for all AoC problems from 2022 for this one, proving it is in a usable state. However, this type of an interpreter is slow, and rarely ever used in production languages.
- The `zote` binary is the more advanced virtual machine (vm) interpreter. It compiles the syntax tree to a custom bytecode format (see [vm/src/compiler/bytecode.rs](vm/src/compiler/bytecode.rs)), and then interprets this bytecode with a virtual machine. This is very similar to how languages like Python work, and is way faster than the simple interpreter as things will be stored much more compactly in memory. At the moment, this is not completely finished, but some solutions to AoC 2022 can be found in [aoc-2022/vm-solutions](./aoc-2022/vm-solutions). This is also what I will use for my [2023 solutions](https://github.com/KvGeijer/advent-of-zote-2023).

One of Zote's core values is that you should be able to logically build your programs in the same direction you write. Take this Python code:
``` python
x = max(map(int, input.split("\n")))
```
This splits the input string, maps each line to an int, takes the max of all the lines, and assigns that to x. This can be seen as a series of transformations of the input, but the order you write the code does not correspond to the order of those transformations. In Zote you can instead write

``` python
input >> split("\n") >> map(int) >> maximum :>> x;
```

Here the data is piped through a series of transformations, from left to right. This makes it easy to write each line as you go, starting with the input, and adding transformations as you think of the next step. Languages like Rust and Java also achieve this by using methods, but I wanted a more mathematical notation of functions, so settled on this.

However, that code desugarizes to:
```python
x := maximum(map(split(input, "\n"), int));
```
which is also valid Zote. These two styles can be mixed depending on what mindset you are writing in. For example, if you are writing very functional code, the pipes might be more clear, but in some cases, you might want to think in a more imperative style, and use normal functions to signify the difference (I usually don't use pipes for functions with side effects).

## Examples

Here are two examples from [aoc-2022/ast-interpreter](./aoc-2022/ast-interpreter) to give a brief introduction to how the language works and looks. See the next heading for some descriptions about the parts of the language.

This is day 2 of AoC 2022, and shows a few different features, some of which are a bit unique. For example, the functions are only singleton expressions and no blocks, and that we extentively use pipes `>>` where e.g. `input >> split("\n")` is the same as `split(input, "\n")`. Additionally, you can see that the map uses `\>>` which is a shorthand for `\x -> x >>` commonly useful in those situations where you just want a lambda that starts a pipe with the input.

``` rust
fn to_int(char) ->
	match char {
		'A' -> 0,
		'B' -> 1,
		'C' -> 2,
		'X' -> 0,
		'Y' -> 1,
		'Z' -> 2,
	};

fn score1((opp, you)) -> you + 1 + ((you - opp + 1) % 3) * 3;

fn score2((opp, res)) -> res*3 + (opp + res - 1) % 3 + 1;

[score1, score2] >> map(\score ->
	read("aoc-2022/inputs/02.txt")
		>> split("\n")
		>> map(\>> split(" ") >> map(to_int) >> score)
		>> sum
		>> print
	);
```

Following, we have a bit of a longer example of day 14 of AoC 2022 where we simulate sand falling by using a dfs. Here we can see that we use a set for the positions of rocks in the map, which really is just a dict.

``` rust
rocks := set();

// Parse all lines of rocks in the input
for line in read("aoc-2022/inputs/14.txt") >> split("\n") {
	pairs := split(line, " -> ") >> map(\>> split(",") >> map(int));
	for ((x1, y1), (x2, y2)) in zip(pairs, pairs[1:]) {
		if y1 == y2 for x in [min(x1, x2):max(x1,x2)+1] {
			[x, y1] >> insert(rocks);
		}
		else for y in [min(y1, y2):max(y1, y2)+1] {
			[x1, y] >> insert(rocks);
		}
	}
}

maxy := rocks >> map(\((_, y),_) -> y) >> max;

origin := [500, 0];
visited := set();

part1 := false;

fn dfs((x, y)) -> {
	// Cache results, so as to not visit places already occupied by sand (or rock)
	if ([x, y] >> in(visited)) or ([x, y] >> in(rocks)) or y >= maxy + 2 return;

	if y >= maxy and !part1 {
		print(len(visited));
		part1 = true;
		return
	}

	dfs([x, y+1]);
	dfs([x-1, y+1]);
	dfs([x+1, y+1]);

	[x, y] >> insert(visited);
}

dfs(origin);
print(len(visited));
```

## Features

Zote is in development, and there is no real documentation, except reading all the code. Here is a short list of features in the language, to get you writing some simple code in no time.

* **Types**, there currently are Iterables (List, Dict, String), Numericals (Float, Int, Bool), Nil, and Closures. They can be created in a similar way to Python, with the difference that dicts must be created with the `dict` functions, and that there are no list comprehensions. Notably, there is no set, but its functionality is achieved with dicts and set-like functions.
* **Variables**
  * Declare x with the value y as `x := y` or `y :>> x`,
  * Assign x to y as `x = y` or `y =>> x`.
* **Functions**
  * Call f as `f(x, y, z)`, or the equivalent `x >> f(y, z)`,
  * Declare f as `fn f(x, y, z) -> _expr_` or as equivalently as a lambda `\x, y, z -> _expr_`.
    * There is also a shorthand to create a lambda with one unnamed argument. Instead of e.g. `\line -> line >> split(" ") >> map(int) >> sum`, you can write `\>> split(" ") >> map(int) >> sum`, as it is a common pattern in map calls in pipes.
* **Pattern matching**
  * In all declarations/assignments, the code expects a variable, a constant (e.g. `1` or `"const"`), or an iterator of further l-values such as `(x, y, (z1, z2)) := [1, [], "yo"];`,
  * The **match** expression uses this matching on the form `match x { arm1 -> _res_ ...}`.
* **Expressions**
  * **Math** works as in most modern languages, maybe with the exception that exponentiation is `^` and that there are no bit-operations, and that `!` is used for negation while `and`/`or` are used instead of `&&`/`||`.
  * **Blocks** `{...}` contains a sequence of statements, and returns Nil or the value of the last statement if it is not terminated with a `;`.
  * Everything except declarations are expressions and return values (such as if-expressions), but loops currently only return Nil.
  * **Loops**, while loops are as you expect, and for loops are only for each loops, in the form `for x in [1, 2, 3] {...}` (same as `for x in [1:4] {...}`).
  * **Slicing**, you can slice lists much as in Python with `xs[start:exclusive_stop:step]`.
  * One neat thing is that everything such as loops/if-expressions/functions expects expressions as their bodies, which does not have to be blocks. So you can e.g. write loops as `for line in lines for char in line if char != "#" {...}` or similar.
* **Standard library**, so far there is no standard library in the ast-interpreter and no way of importing files (something like C includes will be added in the future). But it has quite several built-in functions which all are in [one file](src/ast_interpreter/functions/builtins.rs), and are quite easy to look up.


## Development

I read the excellent book [Crafting Interpreters](craftinginterpreters.com) for inspiration and advice in what order to implement things. Zote does not have any big innovations but instead combines ideas from [Rust](https://www.rust-lang.org/), [Julia](https://julialang.org/), [Python](https://www.python.org/) and [Noulith](https://github.com/betaveros/noulith), in no particular order (and of course from other languages as well).

Currently, at the beginning of December 2023, there is a working simple interpreter directly traversing the abstract syntax tree (AST). It does not contain every feature I want in the language, but most core features. The next virtual machine interpreter is more interesting, and mostly in a working state. However, it still contains some bugs.

So what is next? It would be very nice to add some more features, such as efficient iterators, tinker a bit with the syntax of pattern matching, and improve the standard library (indluding documentations). Then, as I have started using it for Advent of Code, it would be lovely to get some syntax highlightig (and a LSP, but that is probably too much work).

## Benchmarks

There are some performance benchmarks [here](./benches), and using GitHub actions, you can track their approximate performance over time [here](https://kvgeijer.github.io/zote/dev/bench/). These are temporarily disabled, but will soon be re-enabled when the vm interpreter is mature enough.
