# j2gbc

A Rust Game Boy emulator.

This emulator generally needs to be run in release mode. Debug mode performance is very bad.

    # Debian/Ubuntu dependencies
    sudo apt-get install -y libasound-dev libgtk-3-dev byacc flex

    cargo run --release --bin minifb_frontend -- /path/to/rom/file

Controls are:
 - Arrow keys for Up / Down / Left / Right
 - Keyboard Z => Game Boy A
 - Keyboard X => Game Boy B
 - Keyboard A => Game Boy Start
 - Keyboard S => Game Boy Select
 - Escape to quit

To run tests, be sure to clone all submodules and then build the conformance ROMs.

    git submodule update --init --recursive
    make -C j2gbc/gb-conformance
    cargo test