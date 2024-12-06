# Hello, World!

When writing your first Zote program, we recoomend the normal _Hello, World!_ one. Such a script is very simple, only requiring a file as seen below.
```
// main.zote
print("Hello, World!");
```

Actually, we can make it even shorter, as Zote treats the whole script as a scope, returning (and printing) the last value unless supressed by a semi-colon. Therefore, a file only containing the string would suffice:

```
// main.zote
"Hello, World!"
```

Then, you can execute this script with `zote`:

``` sh
$ zote main.zote
Hello, World!
```


Fun fact, the print function also returns its input value, which we will see can be very convenient, so if you write a script like the following:


```
// main.zote
print("hello, world!")
```
you will print the string twice. Once by the `print` function, and once from the return value of the script:
``` sh
$ zote main.zote
Hello, World!
Hello, World!
```

## A More Complex Example

To give a taste of a more complex script, we here have a script which reads a file, and prints all the characters of each line in descending order, if the line is longer than 10 characters:

```
include!("stdlib");

fn filter_and_sort_lines(text) -> {
  text
    >> split("\n")
    >> filter(\>> len >> geq(10))
    >> map(list)
    >> map(\line -> line >> sort >> [::-1] >> join(""))
}

result := read("input.txt") >> filter_and_sort_lines;
result >> map(print);
```

This script tries to show off some of the core language features of Zote (being unnecessarily verbose):
- `name := expression;` is a declaration statement, declaring a new variable `name`.
- `arg1 >> f(arg2, arg3)` is a pipe, and is actually syntax sugar for `f(arg1, arg2, arg3)`, and works for any number of arguments (above 0), as seen above where we use both `... >> filter_and_sort_lines` (1 argument), and `... >> geq(10)` (2 arguments). This is just syntax sugar, but a design principle the language is built around.
- `variable[::-1]` is list slicing, much like it works in Python, here reversing a list. It is also usable as part of a pipe, as seen above `... sort >> [::-1] >> ...`.
- `fn name(args) -> <body>` declares a function (actually a closure, and syntax sugar for a lambda function).
- `\arg1, arg2 -> body` defines a lambda function of two parameters (also a closure).
- `\>> len` is syntax sugar for creating a lambda function that pipes its parameter direction into the function (equivalent to `\arg -> arg >> len`). This is very useful when using `map` or `filter`, but requiring a function which takes more than one argument, or chaining several function together inside. 
- `{ ... }` is a local scope, confining variable bindings, and also returning the value of the last contained statement, if it is not terminated with a semi-colon. 
- `include!("stdlib")` is a macro that includes the standard library, containing functions such as `map` and `filter`. This is the only macro in the language so far (you cannot define custom ones).

This script could also be written more compactly:

```
include!("stdlib");

read("input.txt")
  >> split("\n")
  >> filter(\>> len >> geq(10))
  >> map(\>> list >> sort >> [::-1] >> join("") >> print);
```
