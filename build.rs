//! Copies `memory.x` next to the build output so the linker can find it,
//! and passes the linker scripts cortex-m-rt and defmt need.

use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn main() {
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    File::create(out.join("memory.x"))
        .unwrap()
        .write_all(include_bytes!("memory.x"))
        .unwrap();
    println!("cargo:rustc-link-search={}", out.display());
    println!("cargo:rerun-if-changed=memory.x");

    println!("cargo:rustc-link-arg-bins=--nmagic");
    println!("cargo:rustc-link-arg-bins=-Tlink.x");
    println!("cargo:rustc-link-arg-bins=-Tdefmt.x");

    // Wi-Fi credentials for the telemetry build, from a gitignored `wifi.secrets` file
    // (KEY=VALUE lines, see wifi.secrets.example). Missing file = no Wi-Fi, firmware still builds.
    let mut ssid = String::new();
    let mut password = String::new();
    let mut port = String::from("4210");
    let mut host = String::new();
    if let Ok(text) = std::fs::read_to_string("wifi.secrets") {
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') { continue; }
            if let Some((k, v)) = line.split_once('=') {
                // Accept shell-style quoting: KEY='value' or KEY="value".
                let v = v.trim();
                let v = v
                    .strip_prefix('\'')
                    .and_then(|v| v.strip_suffix('\''))
                    .or_else(|| v.strip_prefix('"').and_then(|v| v.strip_suffix('"')))
                    .unwrap_or(v);
                match k.trim() {
                    "WIFI_SSID" => ssid = v.trim().to_string(),
                    "WIFI_PASSWORD" => password = v.trim().to_string(),
                    "TELEMETRY_PORT" => port = v.trim().to_string(),
                    "TELEMETRY_HOST" => host = v.trim().to_string(),
                    _ => {}
                }
            }
        }
    }
    println!("cargo:rerun-if-changed=wifi.secrets");
    println!("cargo:rustc-env=WIFI_SSID={ssid}");
    println!("cargo:rustc-env=WIFI_PASSWORD={password}");
    println!("cargo:rustc-env=TELEMETRY_PORT={port}");
    println!("cargo:rustc-env=TELEMETRY_HOST={host}");
}
