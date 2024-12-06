# Collection Types

There are also 4 types of collection types:
- List
  - This is a list of items, constructed such as `[1, 2, false]`. Sadly, there is no full list comprehension yet, and the Zote alternative is using `map` to generate the list. However, there is a sort of list comprehension to generate lists of integers, on the form `[start:stop:step]` where `stop` is excluded and `step` optional. For example, `[1:4] = [1, 2, 3]` and `[8:0:-2] = [8, 6, 4, 2]`.
  - Zote does not have tuples, and instead always uses Lists.
- String
  - These Strings are simple to work with, represented as a vector of bytes. The nice part of this is that you can use functions such as `map` on them, and index into them easily. However, indexing becomes strange when you use characters outside ascii. To create a String, use double or single quotes (they are equivalent) such as `"This is a strig"`. Strings are mutable.
- Dictionary
  - Dictionaries are hash-maps, mapping keys to values. A key can be any primitive type, a List, or a String. When we use a List or a String as a key, it is copied to avoid issues with mutating the key afterwards. You can create one with the `dict` built-in function.
- PriorityQueue
  - This is a bit of a strange type, and included to have an efficient priority queue for programming challenges. It could also be implemented directly in Zote over a list.
  - You create a priority queue with `priority_queue()`, push to it with `push_pq(value, priority, queue)`, and pop the item with the _highest_ priority with `pop(queue)`.
  - All priorities in a queue must be comparable and have the same type, but don't have to be primitives.
  
As you might have guessed, our dynamic type system allows you to mix the contained types in these collections.
