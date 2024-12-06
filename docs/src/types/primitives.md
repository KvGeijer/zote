# Primitive Types

These are the 4 types of primitive types:
- Nil
  - Just a single value used as `nil` in the code to represent something missing.
- Bool
  - A boolean, either `true` or `false`.
- Int
  - A 64 bit signed integer. Written as a number `23` in code.
- Float
  - A 64 bit float. Written as an integer, but with decimals, e.g. `23.0`.

## Truthiness
Essentially all types in Zote have a notion of truthiness, which can be seen as having a mapping to a Bool. For the primitive types, `nil` always map to `false`, and all numbers except zero are treted as `true`.

Skipping ahead, collections are considered `true` unless they are empty, and closures are the only types which don't have a truthiness concpept.

## Numerical Promotion

When using an operator on two primitive types, they are both promoted minimally to the same type, before carrying out the operation in that promoted type. The `nil` value is not allowed to be promoted. Otherwise, the types are ordered as follows:
1. Bool
2. Int
3. Float

If operating on two different types, the one of lower rank is promoted until it reaches the other rank. A `false` value is promoted to `0`, and `true` to `1`. An Int is promoted to float as normal by picking the closest float value.

