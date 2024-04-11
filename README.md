# CCMN EOL Test

This repo contains end of line testing scripts and firmware for the Cooper
Common Microcontroller Node.

### Supported Operating Systems

* Full Support: Linux
* Partial Support MacOS/Darwin

### Testing Instructions

#### Requirements

* [SCons](https://scons.org) `pip install --user scons`
* [Rust](https://rust-lang.org) Your package manager or `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`

Depending on your distro, you may also need to install `libudev` (for Fedora,
this is `sudo dnf install libudev-devel`).

#### TL;DR

```bash
# Make sure to update & initialize submodules
git submodule update --init --recursive

# Set up and compile
scons fw

# Compile eoltest
(cd "eoltest" && cargo build)

# Flash board with tester firmware
scons fw --pioflags="run -e tester -t upload"

# Connect all required connections then run tester
(cd "eoltest" && cargo run -- --tester-port /dev/ttyACM0 --serial-number 9)
```

