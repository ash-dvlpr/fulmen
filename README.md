# Fulmen
A Vulkan Game Engine written in Rust. This is a pet project of mine for learning purposes.

Inspired by both the [Hazel](https://github.com/TheCherno/Hazel) and [Kohi](https://github.com/travisvroman/kohi) game engine projects.


# Running
```sh
cargo run --manifest-path=test_app/Cargo.toml --profile dev
```

___

I'm dividing the project into two parts.

Part one is the engine itself.
The other part is the game/application/visualizer, that makes use of the "engine" to render stuff to the screen.



## Custom Profiles
- `dev` | customized for optimized dependencies, incremental builds and no debug symbols (debug-assertions still on).
- `release` | maximizes for optimizations, disables `lto`s.
- `devinfo` | `dev` profile with debug-symbols.

