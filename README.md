# The Zote Programming Language

Zote is an imperative, dynamically typed scripting language with inspiration from functional programming. A target of mine is to solve [Advent of Code 2023](https://adventofcode.com/) with zote's virtual machine implementation, and the whole language is designed to be pleasant to use for that type of small problems. -It should be easy and pleasant to write files of up to a couple hudred lines of code, but it is not at all designed for larger projects.

One of Zote's core values is that you should write your programs in the same direction as you think. Take this Python code:
``` python
max(map(int, input.split("\n")))
```
This splits the input string, maps each line to an int, and finally takes the max of all the lines. This can be seen as a series of transformations of the input, but to write it you have to start writing the final transformation step (unless you jump back and forth in the code). In Zote you can instead write

``` python
input >> split("\n") >> map(int) >> maximum
```

Here the data is piped through a series of functions, from left to right, where you start with the input data, and apply a series of tranformations as you write. Languages like Rust and Java also achieve this by using methods, but I wanted a more mathematical notation of functions. The Julia language has pipes, but they felt a bit clunky to me, only working as I wanted for functions with one parameter. The notation for Zote works on any number of argument, and are usually constructed in a way that the first parameter is the _data_ which can be piped.

There is no difference in performance between pipes and normal function calls, and the above Zote code desugarizes to
```python
maximum(map(split(input, "\n"), int))
```
which is also valid Zote. These two styles can be mixed depending on what mindset you are writing in. For example, if you are writing very functional code, the pipes might be more clear. But, in some cases you don't have a clear data to pipe, or you do some mutation, and then it can be nice to just use normal function call syntax.

## Installation

At the moment, Zote has two working interpreters, split as two binaries.
- The `zote` binary is the recommended virtual machine (vm) interpreter. It compiles the syntax tree to a custom bytecode format (see [vm/src/compiler/bytecode.rs](vm/src/compiler/bytecode.rs)), and then interprets this bytecode with a virtual machine. This is very similar to how languages like Python work, and is often used as code will be stored compactly in memory. This is also what I use for my [2023 solutions](https://github.com/KvGeijer/advent-of-zote-2023) of [Advent of Code](https://adventofcode.com/). However, this does not have a fully working repl (each line is treated as a stand-alone program).
- The `ast-zote` binary is a simpler interpreter that directly traverses the syntax tree during runtime. It works well, but this type of interpreter is rather slow, and rarely ever used in production languages. However, it has a better repl, and might be used for that purpose.

There is a precompiled binary for x86 Linux and the latest relase at GitHub. However, the recommended way is to install from source. First [install Rust](https://www.rust-lang.org/tools/install) and set it up so that you can use `cargo`. Then install as below.

``` bash
git clone git@github.com:KvGeijer/zote.git
cd zote
cargo install --path .
```

This will install the standard virtual machine interpreter, which you can use to run code with as `zote <code.zote>`. You can also install the ast-interpreter, which primarily is recommended if you want to use the repl. Then you add ```--bin ast-zote``` to the installation command, and use the installed ```ast-zote``` command.

## Examples

Here are two examples from Advent of Code to give a brief overview to how the language works and looks. See the next heading for some descriptions about the parts of the language.

This is day 2 of AoC 2022, and shows a few fun features. For example, functions bodies don't have to be blocks, and can instead be singleton expressions. Furthermore, the code extensively uses pipes `>>` where e.g. `input >> split("\n")` is the same as `split(input, "\n")`. Additionally, you can see that the map uses `\>>` which is a shorthand for `\x -> x >>`, commonly useful in situations where you just want a lambda that starts a pipe chain with the input. It also shows that expressions such as `match` returns the values from the computed brach (similarly with `if`).

``` rust
include!("stdlib");

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

for score in [score1, score2]
	read("../inputs/02.txt")
		>> split("\n")
		>> map(\>> split(" ") >> map(to_int) >> score)
		>> sum
		>> print
```

Following, we have a bit of a longer example of [AoC 2023 Day 17](https://adventofcode.com/2023/day/17) part 2. Here I implemented Dijkstras pahtfinding algorithm, with a bit strange rule to find neighbors in the implicit graph (the movement rules were wierd). It is partly included here as it is a more real example. But mainly as it is the first solution with which I came top 100 on the global leaderboards.

``` rust
include!("stdlib");
include!("aoc.zote");

karta := read("input")
	>> trim
	>> split("\n")
	>> map(\>> map(int));

rows := len(karta);
cols := len(karta[0]);

dirs := [[0,1], [0,-1], [1, 0], [-1,0]];

prioq := priority_queue();
[[0, 0], [-1, 0]] >> push_pq(0, prioq);
visited := set();

while true {
	(loss, pos_dir) := prioq >> pop;

	if pos_dir >> in(visited) continue;
	pos_dir >> insert(visited);
	(pos, ldir) := pos_dir;

	if pos == [rows-1, cols - 1] {
		// Found the optimal path to the end
		print(-loss);
		break;
	}

	pdirs := dirs >> filter(\>> neq(ldir));

	for dir in pdirs if dir[1] != -ldir[1] {
		npos := pos;
		nloss := loss;
		for step in [1:11] {
			npos = npos >> vadd(dir);

			// out of bounds?
			if npos[0] >= rows or npos[0] < 0 or npos[1] >= cols or npos[1] < 0 break;

			nloss -= karta[npos[0]][npos[1]];

			if step >= 4
				[npos >> clone, dir] >> push_pq(nloss, prioq);
		}
	}
}
```

## Features

Zote is in active development, and there is no great documentation (except reading all the code). Here is a short list of features in the language (vm version), to get you writing some simple code in no time. There is also a [standard library](vm/stdlib.zote) which has quite a lot of simple functions with some documentation.

* **Types**, there are currently Collections (List, Dict, String, PriorityQueue), Numericals (Float, Int, Bool), Nil, and Closures. They can be constructed in a similar way to Python, with the difference that dicts must be created with the `dict` function, and that there are no list comprehensions. Notably, there is no set, but its functionality is achieved with dicts and set-like functions on dicts (see `insert` in stdlib).
* **Variables**
  * Declare x with the value y as `x := y`,
  * Assign x to y as `x = y`.
* **Functions**
  * Call f as `f(x, y, z)`, or the equivalent `x >> f(y, z)`,
  * Declare f as `fn f(x, y, z) -> _expr_` or as equivalently as a lambda `f := \x, y, z -> _expr_`.
    * There is also a shorthand to create a lambda with one unnamed argument. Instead of e.g. `\line -> line >> split(" ") >> map(int) >> sum`, you can write `\>> split(" ") >> map(int) >> sum`, as it is a common pattern in map calls in pipes.
  * Both `fn f(...` and `f := \...` parse to the same syntax node, and can both be called recursively (and are real closures).
* **Pattern matching**
  * In all declarations/assignments, the code expects either a variable, a constant, or an iterator of further l-values such as `(x, y, (z1, z2)) := [1, [], "yo"];`,
  * The **match** expression uses this matching on the form `match x { arm1 -> _res_ ...}`.
* **Expressions**
  * **Math** works as in most modern languages, maybe with the exception that exponentiation is `^`, that there are no special operators for bitwise functions, and that `!` is used for negation while `and`/`or` are used instead of `&&`/`||`.
  * **Blocks** `{...}` contains a sequence of statements, and returns `nil` or the value of the last statement if it is not terminated with a `;`.
  * Everything except declarations are expressions and return values. However, loops currently only return `nil`, as it is uncler what they should output.
  * **Loops**, while loops are as you expect, and for loops are for-each loops, in the form `for x in [1, 2, 3] ...` (same as `for x in [1:4] ...`).
  * **Slicing**, you can slice lists similarly as in Python with `xs[start:exclusive_stop:step]`. The fields are optional, and you can for example write `xs[::-1]` to reverse a list.
  * One neat thing is that everything such as loops/if-expressions/functions expect expressions as their bodies, which does not have to be blocks. So you can e.g. write loops as `for line in lines for char in line if char != "#" {...}` or similar.
* **Standard library**, there is a standard library in [stdlib.zote](vm/stdlib.zote) which can be included with a `include!("stdlib")` macro. This macro can also be used to include any other local file such as `include!("aoc.zote")`. Otherwise there are also native functions such as `print`, `push` and more in [vm-natives](vm/src/value/builtins/natives.rs) and [ast-builtins](ast_interpreter/src/functions/builtins.rs).


## Development

I read the excellent book [Crafting Interpreters](craftinginterpreters.com) for inspiration and advice in what order to implement things. Zote does not have any big innovations but instead combines ideas from [Rust](https://www.rust-lang.org/), [Julia](https://julialang.org/), [Python](https://www.python.org/) and [Noulith](https://github.com/betaveros/noulith), in no particular order (and of course from other languages as well).

At the time of writing (the beginning of December 2023) the virtual machine interpreter is working and the default `zote` binary. The simpler `ast-zote` is also working, but usually slower.

So what is next? It would be very nice to add some more features, such as efficient iterators, tinker a bit with the syntax of pattern matching, and improve the standard library (indluding documentations). Then, as I have started using it for Advent of Code, it would be lovely to get some syntax highlightig (I have started work on a tree-sitter parser).

## Benchmarks

There were some performance benchmarks [here](./benches) maintained using GitHub actions [here](https://kvgeijer.github.io/zote/dev/bench/). These are temporarily disabled, but will soon be re-enabled when I have time to set them up for the vm.
