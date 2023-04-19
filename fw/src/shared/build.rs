use std::env;
use std::io::Write;
use std::path::PathBuf;

struct ModHeaders {
    mod_name: &'static str,
    headers: Vec<String>,
}

macro_rules! env_required {
    ($var: expr) => {
        env::var(stringify!($var)).unwrap()
    };
}

const LAST_RUN_DOTENV: &'static str = ".lastcargobuildenv";

fn main() {
    println!("cargo:rerun-if-changed={LAST_RUN_DOTENV}");

    if env::var("CARGO_PIO_BUILD_PROJECT_DIR").is_err() {
        dotenv::from_filename(LAST_RUN_DOTENV).expect(
            "CARGO_PIO flags not found and could not load a .lastcargobuildenv. \
            Try running the build from platformio first.",
        );
    }

    // macro_rules! m {
    //     ($name:ident, $headers: expr) => {
    //         ModHeaders {
    //             mod_name: stringify!($name),
    //             headers: $headers.into(),
    //         }
    //     };
    // }

    let mods: Vec<ModHeaders> = vec![
        //
    ];

    // Get build flags that platformio is using, else set some defaults.
    // Those defaults get used if you, e.g., `cargo build` instead of driving
    // the build through platformio.
    let build_flags = env_required!(CARGO_PIO_BUILD_FLAGS);

    let cppdefines: Vec<String> = env_required!(CARGO_PIO_BUILD_CPPDEFINES)
        .split_ascii_whitespace()
        .into_iter()
        .map(|f| format!("-D{f}"))
        .collect();

    for m in mods {
        for h in &m.headers {
            // Tell cargo to invalidate the built crate whenever the wrapper changes
            println!("cargo:rerun-if-changed={}", h);
        }

        let mut bindings = bindgen::Builder::default();

        for h in &m.headers {
            bindings = bindings.header(h);
        }

        let bindings = bindings
            .clang_args(build_flags.split_ascii_whitespace())
            .clang_args(&cppdefines)
            .parse_callbacks(Box::new(bindgen::CargoCallbacks))
            .default_enum_style(bindgen::EnumVariation::Rust { non_exhaustive: true })
            .generate()
            .expect("Unable to generate bindings");

        let out_path = PathBuf::from(env_required!(OUT_DIR));
        bindings
            .write_to_file(out_path.join(format!("{}.rs", m.mod_name)))
            .expect("Couldn't write bindings!");
    }

    let mut lastenv = std::fs::File::create(LAST_RUN_DOTENV).unwrap();
    for (var, val) in env::vars() {
        if !var.starts_with("CARGO_PIO") {
            continue;
        }

        lastenv
            .write_all(format!("{var}=\"{}\"\n", val.escape_debug()).as_bytes())
            .unwrap();
    }
}
