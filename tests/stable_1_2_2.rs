use std::path::PathBuf;
use verifier::semantics::Semantics;
use verifier::{SUPERVISOR, verify};

#[test]
fn stable_test_1_2_2(){
    let result = verify(1, PathBuf::from("./tests/data/stable_1.ccl"), None, Some(PathBuf::from("./tests/data/stable_1.required")),
                        PathBuf::from("./tests/data/stable_1_2.proof"), Semantics::Stable, true);
    assert_eq!(result.1, verifier::EXIT_CODE_OK);
    let result = SUPERVISOR.get().unwrap().get_result();
    assert_eq!(result.unwrap().1.unwrap(), 10);
}
