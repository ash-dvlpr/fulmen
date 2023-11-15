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
- `release` | maximizes for optimizations, disenables `lto`.
- `devinfo` | `dev` profile with debug-symbols.





```toml
# Profiles are "dev", "devinfo", "release" (use with --no-default-features)
# Profile fast to compile, okay enough to run, no debug
[profile.dev]
opt-level = 1
debug = false
debug-assertions = true
overflow-checks = true
panic = 'unwind'
lto = false # Faster linking, worse optimizations
incremental = true # Better re-compilation times
codegen-units = 16 # Same as non incremental builds
# Dependencies | AKA | Packages that aren't members of the workspace
[profile.dev.package."*"]
opt-level = 3

# Profile in case 'dev' doesn't have enough debug info
[profile.devinfo]
inherits = 'dev'
debug = true

[profile.release]
opt-level = 3
debug = false
debug-assertions = false
lto = true
incremental = false
```