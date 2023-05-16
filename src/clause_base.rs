use std::sync::atomic::{AtomicBool, Ordering};
use crate::semantics::VerifierType;

/// The clauses of the instance / proof.
pub struct ClauseBase {
    /// The index of the clause
    index: usize,
    /// The IDs of the arguments contained in the clause and their sign.
    members: Vec<(usize, bool)>,
    /// The clause index at which point this clause was deleted an is no longer valid.
    deleted_at: Option<usize>,
    /// The function used to verify this clause.
    verifier: Option<VerifierType>,
    /// Whether or not the clause has been used during verification.
    used: AtomicBool,
}

impl ClauseBase {

    pub fn new(index: usize) -> ClauseBase {
        ClauseBase {
            index,
            members: Vec::new(),
            deleted_at: None,
            verifier: None,
            used: AtomicBool::new(false)
        }
    }

    /// Adds the provided argument with sign to the members of this attack.
    #[inline]
    pub fn add_member(&mut self, argument: usize, sign: bool) {
        self.members.push((argument, sign));
    }

    #[inline]
    pub fn get_index(&self) -> usize {
        self.index
    }

    #[inline]
    pub fn set_verifier(&mut self, verifier: VerifierType) {
        self.verifier = Some(verifier);
    }

    #[inline]
    pub fn get_verifier(&self) -> &VerifierType {
        self.verifier.as_ref().unwrap()
    }

    #[inline]
    pub fn set_deleted_at(&mut self, index: usize) {
        self.deleted_at = Some(index);
    }

    #[inline]
    pub fn get_number_of_members(&self) -> usize {
        self.members.len()
    }

    #[inline]
    pub fn get_members(&self) -> &Vec<(usize, bool)> {
        &self.members
    }

    #[inline]
    pub fn get_member(&self, index: usize) -> (usize, bool) {
        self.members[index]
    }

    #[inline]
    pub fn is_used(&self) -> bool {
        self.used.load(Ordering::Acquire)
    }

    #[inline]
    pub fn set_used(&self) -> bool {
        !self.used.swap(true, Ordering::AcqRel)
    }

    #[inline]
    pub fn deleted_at(&self) -> Option<usize> {
        self.deleted_at
    }

}




