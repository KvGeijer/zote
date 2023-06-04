# Improvement ideas for Zote

* Tighten up grammar to improve inconsistencies such as "if true [1]", as that will seem as an indexing into true

* Add different error types

* Add tests for benches, so we at least can run them without error.

* Should we have tuples? They are in place. They are more suitable when using as keys for example. 
  * Or should we use them in some way for >> instead. So that 1 >> f(2, 3) is the same as (1, 2) >> f(3) or (1,2,3) >> f? 

* Implement syntactic analysis for if/else expressions. If we use the value of such an expression we might want to force it to have both if and else? Could do for eg blocks as well.

* Add structs.
  * What features should they have? For example would be nice to implement some point struct for AoC...

* Add maps (sets as well)

* Add cool for each loops.

* Expand standard library

* Remove need for semicolons after blocks in some cases. Such as an expression statement ending with a block probably.

* Start work on a real virtual machine interpreter. Could probably be a lot faster than the naive one.

* Add pattern matching.
  * Pattern match against a sequence
  * Incorporate it in function definitions
  
