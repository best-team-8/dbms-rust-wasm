//! Chooses the parts of the programs that depend on an internal API of the
//! measured version of the core. `run.mjs` passes the core's directory in
//! FLATSQL_CORE_DIR; without it the core of this repository is used. Each
//! entry sets a `cfg` when the core's sources contain a marker of the API.

use std::env;
use std::fs;
use std::path::PathBuf;

/// (cfg name, source file of the core, marker text)
const API: &[(&str, &str, &str)] = &[
    // Expressions evaluated with a context of aggregates and parameters.
    ("core_cx", "expr.rs", "pub struct Cx"),
    // Prepared statements keep a compiled access path.
    ("core_access_plan", "plan.rs", "pub struct AccessPlan"),
    // Expressions compiled into closures.
    ("core_compile", "compile.rs", "pub fn predicate"),
    // The outbox is kept only once synchronisation is enabled.
    ("core_sync", "db.rs", "pub fn enable_sync"),
    // Results written in the binary result format straight from the table.
    ("core_encode", "db.rs", "pub fn execute_prepared_encoded"),
    // One prepared write executed for many sets of parameters in one call.
    ("core_many", "db.rs", "pub fn execute_many"),
];

fn main() {
    let core = env::var("FLATSQL_CORE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("../../../../../../crates/flatsql-core"));
    println!("cargo:rerun-if-env-changed=FLATSQL_CORE_DIR");
    println!("cargo:rerun-if-changed={}", core.join("src").display());
    for (cfg, file, marker) in API {
        println!("cargo:rustc-check-cfg=cfg({cfg})");
        let text = fs::read_to_string(core.join("src").join(file)).unwrap_or_default();
        if text.contains(marker) {
            println!("cargo:rustc-cfg={cfg}");
        }
    }
}
