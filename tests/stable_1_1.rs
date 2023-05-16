use std::path::PathBuf;
use verifier::semantics::Semantics;
use verifier::{verify};

#[test]
fn stable_test_1_1() {

    let result = verify(1, PathBuf::from("./tests/data/stable_1.ccl"), None, Some(PathBuf::from("./tests/data/stable_1.required")),
                        PathBuf::from("./tests/data/stable_1_1.proof"), Semantics::Stable, false);
    assert_eq!(result.1, verifier::EXIT_CODE_OK);
}




