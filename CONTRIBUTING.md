# How to contribute

To contribute to sn0int, clone the repository and make sure both the build and
tests pass for you:

    git clone https://github.com/kpcyrd/sn0int.git
    cd sn0int
    # build the project
    cargo build
    # run regular tests
    cargo test
    # run tests depending on the network
    # these might fail if a service is down
    cargo test -- --ignored

The project is loosely structured into a few folders:

- `src/models/` - database models
- `src/runtime/` - the stdlib that's exposed to lua
- `src/engine/` - code related to lua
- `src/sandbox/` - code related to sandboxing
- `src/cmd/` - cli commands
- `src/` - misc modules

After you're done, make sure the build completes without any warnings and both
tests pass successfully:

    cargo test
    cargo test -- --ignored

If you want to introduce a new feature feel free to open an issue first to make
sure your feature is a good fit for the project before implementing it.
