# Statements

There are only two types of statements:
- _Declaration Statements_, where you declare a (or several with pattern matching) variable and bind it to some value.
  - For example, `x := 2;`.
  - This also covers declarations of functions such as `fn func(x) -> x*2;`.
- _Expression Statements_, which is just a single expression.

Statements are normally terminated with semi-colons, but they are allowed to be left out at some points. Either when a function declaration of form `fn f(...) -> {...}`, or when an expression statement is a block `{ ... }`, or a `for, if, while, match...` which ends with a block.
