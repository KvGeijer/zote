# The Zote programming language

[Work in Progress]

The ultimate goal is to create a language which can solve Advent of Code next year. It would be cool if I could make compile and run bytecode, but that takes a lot more effort than interpreting it directly...

I'll implement it following the excellent book [Crafting Interpreters](craftinginterpreters.com), but I'll probably deviate on some parts. For example, I really didn't want to create a scanner completely by hand, so I sort of made a scanner generator in Rust (but it is super inefficient due to me not finding a simple longest match function for Rust regexes).

## Feature ideas

Here are some of the ideas I want to implement in my language.

### Pipe

One thing object oriented languages do right is the notation for methods. They write the source data first, and then apply one or several functions. This makes it easy to understand how we go from the data to the result through a series of transformations. Haskell on the other hand is made for that in a way, but as you write the outermost function first there I feel it becomes a bit unintuitive to read. So I want something similar to the method syntax where I can pipe data through one or several functions.

Therefore I introduce the pipe operator ```>>``` which takes data from the left and a function from the right. 

``` rust
[1,2,3,4] >> max;     -> 4
[1,2,3,4] >> first;   -> 1
```

### Weak partially applied functions

Our pipe operator is similar to ```|>``` in Julia, but that becomes a bit strange when the function to the rigth takes several arguments. We fix this by some wonderful syntactic sugar and indroduces the ```|``` operator which is a shorthand for a lambda function abstracting away the first parameter. This can then be used for partially applying functions.

``` rust
max(1, 2);        -> 2
|max(2)(1);       -> 2
1 >> |max(2);     -> 2
```

This is under heavy consideration. In one way it is nice because it can also be used for things such as map, but it takes three chars to write a single pipe. First I thought this would be done implicitly for the pipe operator, but that might give rise to strange behaviour in some cases, such as when we pipe something into a function call returning another function...

### Structs, not classes

The book I'm following implements a class system, but I would like to stay away from that if possilbe. Instead I just want the old fashioned structs. However I must admit that classes might be nice for scoping functions with the same name on different types, or implementing for example traits (although, we could go the Rust routh here I guess). For example, what shall I do if I want to make a struct an iterable, or implement addidion for one of them? We will have to see how it goes. Especially since the language is dynamically typed (this would be easy if we knew all types at compile time).

### Minimal language

The goal is to have a small scripting language where it is super quick and easy to write running code. It will not be that fast, and probably quite bad for larger projects. For example I think some form of trait/interface systhem is required in a good language, but for a small one like this we can probably skip it as we seldom implement traits in such small scripts.
