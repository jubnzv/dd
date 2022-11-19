# dd

![](https://github.com/jubnzv/dd/workflows/Test/badge.svg)

`dd` implements basic [delta debugging](https://www.debuggingbook.org/html/DeltaDebugger.html) technique for tools that work with the Lua language. It modifies the AST of the given Lua program to find bugs in the tool.

## Installation

```bash
git clone --recurse-submodules https://github.com/jubnzv/dd
cd dd
cargo build
```
