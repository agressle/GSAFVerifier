use std::path::PathBuf;
use verifier::semantics::Semantics;
use verifier::{verify};

#[test]
fn admissible_test_1_3_1() {

    let result = verify(1, PathBuf::from("./tests/data/admissible_1.ccl"), None, Some(PathBuf::from("./tests/data/admissible_1.required")),
                        PathBuf::from("./tests/data/admissible_1_3.proof"), Semantics::Admissible, false);
    assert_eq!(result.1, verifier::EXIT_CODE_OK);
}




