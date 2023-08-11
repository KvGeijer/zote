# Improvement ideas for Zote

* Improve pattern matching:
  * Match against types (very useful!)
  * Match against singleton iterators
  * Add guards (just an if statement in some way)

* Start work on a real virtual machine interpreter. Could probably be a lot faster than the naive one.

* Add decorators
  * Mainly builtin memoize decorator
  * Maybe also define your own, but when would you actually want this? In this small language... But gives oportunity to obfuscate :D

* Fix += and similar shorthands. Now x op= y <-> x = x op y. But this breaks if x contains mutable calls (like C macros)

* Stdlib functions for all binary expressions, to use for pipes

* List of all stdib functions/builtins

* Add nice range for loops such as `for i in 1:10`

* Add priority queue.
  * Maybe just implement it in Zote in a stdlib? Would be a hassle to add it to the vm

* Make operations on env require it to me mutably borrowed.

* Add real iterators (generators) to the language.
  * Can then make slices more generic
  * Can make infinite iterators such as "1:". Or? Is this actually just bad as it becomes quite an open expression.
  * Can then add _yield_ as a keyword to create generators.
  * Can create lazy maps (Maybe all map operations on iterators are lazy while ones on collections are not?) 

* Add different error types

* Add tests for benches, so we at least can run them without error.

* Implement syntactic analysis for if/else expressions. If we use the value of such an expression we might want to force it to have both if and else? Could do for eg blocks as well.

* Add structs.
  * What features should they have? For example would be nice to implement some point struct for AoC...

