# Description

this crate is for [cc wasm](https://www.curseforge.com/minecraft/mc-mods/cc-wasm) mod

# Usage

- add this crate as a dependence.
- set `[lib]`-`crate-type` to `"cdylib"`
- build with `--target wasm32-wasi`
- copy `target/wasm32-wasi/` [`release`|`debug`] / `[crate name].wasm`
  to `.minecraft/wasm/`
- load it in game

# Example

please see the example [here](https://github.com/wefcdse/ccwasm/tree/master/wasmlib)

