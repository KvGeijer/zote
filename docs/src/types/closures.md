# Closures

The final type is Closures, which is a bit different from the others, for example the only one without a notion of truthiness. A close can be declared either as a lambda function, or a function declaration, which are equivalent.

```
fn add(x, y) -> x + y;
fn add(x, y) -> {
  x + y
}   
add2 := \x, y -> x + y; 
```

These are true closures keeping references to referenced variables, making sure to keep the variable alive as long as the closure is alive.

```
fn closure_ret() -> {
  x := 1;
  y := 3;
  fn inc_y() -> y += 1;
  fn add() -> x + y;

  return [inc_y, add];
}

(inc_y, add) := closure_ret();

// Safe to Call
inc_y();
inc_y();

// y will have been incremented
6 = add();
```

These functions/closures can't have a variable number of parameters of default parameters.

## Builtin function

Zote also has a set of builtin functions, which are implemented directly in the virtual machine. These are not really closures, but as you can't define them yourself, they can be thought of similarly to closures. The only difference is that they can have variable number of parameters. For example, `zip` can zip an arbitrary number of lists.
