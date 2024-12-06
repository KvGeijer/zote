# Installation

The simplest way to install Zote is to download its source code, and build it from scratch. The code uses a few ergonomic features from the nightly Rust toolchain, so you need to download that to be able to compile.

``` sh
git clone git@github.com:KvGeijer/zote.git
cd zote
cargo +nightly install --path .
```

This assumes that you have the _nightly_ toolchain installed. If you have a specific version of nightly installed, you should adapt the command to use that version instead of `+nightly`.

Afterwards, you should be able to run scripts with

```sh
zote main.zote
```

or run the (quite poor) REPL by not specifying a file. This REPL is not recommended, as it at the moment does not track state between lines.
```sh
zote
```

TOOD: ast-zote

## Tree-Sitter Grammar

There is a tree-sitter grammar for Zote which you can use for syntax highlighting. More information can be found at its repository [https://github.com/KvGeijer/tree-sitter-zote](https://github.com/KvGeijer/tree-sitter-zote).
