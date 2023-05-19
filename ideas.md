# Improvement ideas for Zote

* Treat Int, Float, Nil and Bool as a Bit enum, allowing more equalities and comaprisons between them

* Should we have tuples? They are in place. They are more suitable when using as keys for example. 
  * Or should we use them in some way for >> instead. So that 1 >> f(2, 3) is the same as (1, 2) >> f(3) or (1,2,3) >> f? 

* Benchmark performance of the ast interpreter. It is probably very slow, but can we optimize it to be fast enough for Aoc? We need benchmarks to be able to do the optimizations. Some things to benchmark:
  * Variable resolution: We can resolve variables with syntactic anaysis before runtime, in maybe a single global hashmap.
  * String operations: Now clone stings left and right when reading values. Maybe change to using reference counting?
  * Some general tests. Solve a few AoC problems to use for benchmarking general improvements/degradations.

* Implement syntactic analysis for if/else expressions. If we use the value of such an expression we might want to force it to have both if and else? Could do for eg blocks as well.

* Add structs.
  * What features should they have? For example would be nice to implement some point struct for AoC...

* Expand arrays to be actually useable.
  * Decide if we want methods, and how functions should be in the language
  * Add parsing to allow , after last element in initializer

* Add maps (sets as well)

* Add cool for each loops.

* Expand standard library
  * Implement I/O
  * String operations (mainly split)

* Remove need for semicolons after blocks in some cases. Such as an expression statement ending with a block probably.

* Start work on a real virtual machine interpreter. Could probably be a lot faster than the naive one.

* Add pattern matching.
  * Do we want to just add a match expression, or do we want to do it in other places as well? Such as assinments, probably good...

* Maybe add >>= which is like >>: except it does not declare a new variable, just assigns. Could then be part of chain.
  * Should only be added after I feel like it is needed.
  * If it is not used, maybe change >>: to >>=, which might be more loigcal, and not introduce :

