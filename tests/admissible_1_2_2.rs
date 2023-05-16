use std::path::PathBuf;
use verifier::semantics::Semantics;
use verifier::{SUPERVISOR, verify};

#[test]
fn admissible_test_1_2_2() {

    let result = verify(1, PathBuf::from("./tests/data/admissible_1.ccl"), None, Some(PathBuf::from("./tests/data/admissible_1.required")),
                        PathBuf::from("./tests/data/admissible_1_2.proof"), Semantics::Admissible, true);
    assert_eq!(result.1, verifier::EXIT_CODE_OK);
    let result = SUPERVISOR.get().unwrap().get_result();
    assert_eq!(result.unwrap().1.unwrap(), 8);
}

