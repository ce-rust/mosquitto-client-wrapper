/// this file is mainly inspired by the great c library integration from https://github.com/eclipse/paho.mqtt.rust

fn main() {
    bundled::main();
}

const MOSQUITTO_GIT_URL: &str = "https://github.com/eclipse/mosquitto.git";
const MOSQUITTO_VERSION: &str = "2.0.4";

#[cfg(feature = "build_bindgen")]
mod bindings {
    use std::{env, fs};
    use std::path::{Path, PathBuf};
    use MOSQUITTO_VERSION;

    pub fn place_bindings(inc_dir: &Path) {
        let inc_search = format!("-I{}", inc_dir.display());

        // The bindgen::Builder is the main entry point
        // to bindgen, and lets you build up options for
        // the resulting bindings.
        let bindings = bindgen::Builder::default()
            // Older clang versions (~v3.6) improperly mangle the functions.
            // We shouldn't require mangling for straight C library. I think.
            .trust_clang_mangling(false)
            // The input header we would like to generate
            // bindings for.
            .header("wrapper.h").clang_arg(inc_search)
            // Finish the builder and generate the bindings.
            .generate()
            // Unwrap the Result and panic on failure.
            .expect("Unable to generate bindings");

        // Write the bindings to the $OUT_DIR/bindings.rs file.
        let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
        let out_path = out_dir.join("bindings.rs");

        bindings
            .write_to_file(out_path.clone())
            .expect("Couldn't write bindings!");

        // Save a copy of the bindings file into the bindings/ dir
        // with version and target name, if it doesn't already exist

        let target = env::var("TARGET").unwrap();
        println!("debug:Target: {}", target);

        let bindings = format!("bindings/bindings_mosquitto_{}-{}.rs",
                               MOSQUITTO_VERSION, target);

        if !Path::new(&bindings).exists() {
            if let Err(err) = fs::copy(out_path, &bindings) {
                println!("debug:Error copying new binding file: {}", err);
            } else {
                println!("debug:Created new bindings file {}", bindings)
            }
        }
    }
}

#[cfg(feature = "bundled")]
mod bundled {
    use std::process::Command;
    use super::*;
    use std::path::Path;
    use std::process;
    use std::env;

    extern crate cmake;

    pub fn main() {
        println!("Running the bundled build");

        let args = vec![
            "clone".to_string(),
            env::var("MOSQUITTO_GIT_URL").unwrap_or(MOSQUITTO_GIT_URL.to_string()),
            "--depth=1".to_string()
        ];

        if let Err(e) = Command::new("git").args(&args).status() {
            panic!("failed to clone the git repo: {:?}", e);
        }

        if let Ok(hash) = env::var("MOSQUITTO_GIT_HASH") {
            if let Err(e) = Command::new("git").args(&["fetch", "--depth", "1", "origin", hash.as_str()]).status() {
                panic!("failed to fetch the git hash: {:?}", e);
            }
            if let Err(e) = Command::new("git").args(&["checkout", hash.as_str()]).status() {
                panic!("failed to checkout the git hash: {:?}", e);
            }
        }

        let mut cmk_cfg = cmake::Config::new("mosquitto");
        cmk_cfg.define("WITH_BUNDLED_DEPS", "ON");
        cmk_cfg.define("WITH_EC", "OFF");
        cmk_cfg.define("WITH_TLS", "OFF");
        cmk_cfg.define("WITH_TLS_PSK", "OFF");
        cmk_cfg.define("WITH_APPS", "OFF");
        cmk_cfg.define("WITH_PLUGINS", "OFF");
        cmk_cfg.define("DOCUMENTATION", "OFF");
        let cmk = cmk_cfg.build();

        let lib_path = if cmk.join("lib").exists() {
            "lib"
        } else {
            panic!("Unknown library directory.")
        };

        let lib_dir = cmk.join(lib_path);

        let library_name = "mosquitto";
        let link_file = format!("lib{}.so.1", library_name);

        let lib = lib_dir.join(Path::new(&link_file));
        println!("debug:Using mosquitto C library at: {}", lib.display());

        if !lib.exists() {
            println!("Error building mosquitto C library: '{}'", lib.display());
            process::exit(103);
        }

        // Get bundled bindings or regenerate
        let inc_dir = cmk.join("include");
        bindings::place_bindings(&inc_dir);

        // we add the folder where all the libraries are built to the path search
        println!("cargo:rustc-link-search=native={}", lib_dir.display());
        println!("cargo:rustc-link-lib={}", "mosquitto");
    }
}
