//! Golden-file snapshot test: generating a model from a committed binary
//! must reproduce the committed, campaign-verified artifact byte for byte
//! (modulo line endings). This brings the CI byte-identity gate into
//! `cargo test`, so any semantic drift in the generator fails locally
//! before it ever reaches a solver.

use rotor::config::Config;
use rotor::model::generator;
use std::path::Path;

#[test]
fn division_model_matches_verified_artifact() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let binary = root.join("benchmarks/binaries/division-by-zero-3-35.m");
    let golden = root.join("benchmarks/cse-experiment/division-by-zero-cse-on.btor2");

    let config = Config {
        bytes_to_read: 1,
        heap_allowance: 2048,
        stack_allowance: 2048,
        ..Config::default()
    };

    let mut out = Vec::new();
    generator::model_rotor(&binary, &config, &mut out).expect("model generation failed");
    let generated = String::from_utf8(out).expect("model is not UTF-8");

    let expected = std::fs::read_to_string(&golden).expect("verified artifact missing");

    // Normalize line endings so the comparison is checkout-agnostic.
    let norm = |s: &str| s.replace("\r\n", "\n");
    assert_eq!(
        norm(&generated),
        norm(&expected),
        "generated division model diverges from the campaign-verified artifact"
    );
}
