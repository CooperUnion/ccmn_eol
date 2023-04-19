# Dependencies
# Here, we handle individual dependencies as needed.

import os

Import("env")

# VERSIONS ------------------------------------------------
env['ESP_RUST_VERSION'] = '1.69.0.0'
env['RUST_VERSION']     = '1.68.2'
env['OPENCAN_VERSION']  = 'b014266'
# ---------------------------------------------------------

# venv & pip ----------------------------------------------
env['VENV_DIR'] = env.Dir('venv')

# Make the venv
env.Command(
  env['VENV_DIR'],
  [],
  'python3 -m venv $TARGET'
)

# Add the venv/bin folder to PATH
env.PrependENVPath('PATH', env['VENV_DIR'].Dir('bin').abspath)
# Set VIRTUAL_ENV
env['ENV']['VIRTUAL_ENV'] = env['VENV_DIR'].abspath

# pip packages
REQUIREMENTS = env.File('$REPO_ROOT/requirements.txt')

[pip_deps_builder] = env.Command(
  env['VENV_DIR'].File('.requirements-installed'),
  REQUIREMENTS,
  [
    '$VENV_DIR/bin/pip3 install -r $SOURCE',
    'touch $TARGET'
  ]
)

env.Alias('deps-pip', pip_deps_builder)
env['PIP_PACKAGES'] = pip_deps_builder
# ---------------------------------------------------------


# Rust ----------------------------------------------------

# We'll install into deps/rust/VERSION/ and make a symlink
# deps/rust/current that points to it. This is so we can keep
# multiple versions around in deps/ at once (in case you're switching
# branches etc) and not have them be redownloaded every 2 seconds.

RUST_HOME           = env.Dir('rust/$RUST_VERSION')
RUST_TOOLS_PATH     = RUST_HOME.Dir('bin')
RUST_CURRENT_PATH   = env.Dir('rust/current')
env['CARGO']        = RUST_TOOLS_PATH.File('cargo')

env['ENV']['CARGO_HOME']  = RUST_HOME.abspath
env['ENV']['RUSTUP_HOME'] = RUST_HOME.abspath

# https://blog.rust-lang.org/inside-rust/2023/01/30/cargo-sparse-protocol.html
env['ENV']['CARGO_REGISTRIES_CRATES_IO_PROTOCOL'] = 'sparse'

rust_install_builder = env.Command(
  env['CARGO'], # picking cargo as the target file
  [],
  'curl https://sh.rustup.rs -sSf | ' \
    'sh -s -- -y --no-modify-path --default-toolchain $RUST_VERSION',
)

# make symlink
if os.path.realpath(RUST_CURRENT_PATH.abspath) != RUST_HOME.abspath:
    env.Execute(f'mkdir -p {RUST_CURRENT_PATH.up().abspath}')
    env.Execute(f'ln -sfn {RUST_HOME.abspath} {RUST_CURRENT_PATH.abspath}')

env.PrependENVPath('PATH', RUST_TOOLS_PATH.abspath)
env.Alias('deps-rust', rust_install_builder)
# ---------------------------------------------------------

# ESP Rust ------------------------------------------------
# Note that this will install in the rust/x.x.x path, not rust/, whatever
ESP_RUST_PATH     = env.Dir(env['ENV']['RUSTUP_HOME']).Dir('toolchains/espr')
ESP_RUST_ENV_FILE = env.Dir(env['ENV']['RUSTUP_HOME']).File('esp-env.sh')

env['ESP_CARGO']  = ESP_RUST_PATH.File('bin/cargo')

esp_rust_install_builder = env.Command(
    env['ESP_CARGO'],
    [],
    f'espup install --name espr -t esp32s3 -f {ESP_RUST_ENV_FILE.abspath} -v $ESP_RUST_VERSION'
)

# Update these as needed from esp-env.sh
env.PrependENVPath('PATH', ESP_RUST_PATH.Dir('xtensa-esp32s3-elf/esp-2021r2-patch5-8_4_0/xtensa-esp32s3-elf/bin'))
env.PrependENVPath('PATH', ESP_RUST_PATH.Dir('riscv32-esp-elf/esp-2021r2-patch5-8_4_0/riscv32-esp-elf/bin'))
env['ENV']['LIBCLANG_PATH'] = ESP_RUST_PATH.Dir('xtensa-esp32-elf-clang/esp-15.0.0-20221201/esp-clang/lib').abspath

env.Alias('deps-esp-rust', esp_rust_install_builder)
# ---------------------------------------------------------

# OpenCAN -------------------------------------------------
env['OPENCAN_CLI'] = env.Dir(env['ENV']['CARGO_HOME']).File('bin/opencan-cli')

# Note that we have $CARGO_HOME set above. Cargo will install packages there.
[opencan_cli_builder] = env.Command(
    env['OPENCAN_CLI'],
    env['CARGO'],
    '$CARGO install --locked ' \
    '--git https://github.com/opencan/opencan --rev $OPENCAN_VERSION'
)

env.Alias('deps-opencan', opencan_cli_builder)
# ---------------------------------------------------------
