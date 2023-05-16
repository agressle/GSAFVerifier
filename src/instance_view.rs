use std::collections::VecDeque;
use crate::argument_view::ArgumentView;
use crate::clause_view::ClauseView;
use crate::instance_base::InstanceBase;
use crate::semantics::VerifierType;

/// Represents a worker threads view of an instance, with its respective argument and clause views.
pub struct InstanceView<'a> {
    base: &'a InstanceBase,
    iteration: usize,
    arguments: Vec<ArgumentView>,
    clauses: Vec<ClauseView<'a>>,
    clauses_to_check: VecDeque<usize>
}

impl InstanceView<'_> {
    pub fn new(base: & InstanceBase) -> InstanceView {
        let mut view = InstanceView {
            base,
            iteration: 0,
            arguments: Vec::with_capacity(base.arguments.len()),
            clauses: Vec::with_capacity(base.clauses.len()),
            clauses_to_check: VecDeque::new()
        };

        for _ in &base.arguments{
            view.arguments.push(ArgumentView::new());
        }

        for clause in &base.clauses {
            let clause_view = ClauseView::new(clause, &mut view);
            view.clauses.push(clause_view);
        }

        view
    }

    pub fn reset(&mut self) {
        self.iteration += 1;
        self.clauses_to_check.clear();
        for index in &self.base.unit_clauses {
            self.clauses_to_check.push_back(*index);
        }
        for (argument, value) in &self.base.required_arguments {
            self.set_argument_value(*argument, *value);
        }
    }

    #[inline]
    pub fn get_next_clause_to_check(&mut self) -> Option<usize> {
        self.clauses_to_check.pop_front()
    }

    #[inline]
    pub fn get_clause_members(&self, id: usize) -> &Vec<(usize, bool)> {
        &self.clauses[id].get_members()
    }

    pub fn check_clause_propagation(&mut self, id: usize) -> Option<(usize, bool)> {
        let (result, first_watch_update, second_watch_update) = self.clauses[id].check_propagation(&self);
        if let Some(
            (watch_index,
            (remove_argument_index, remove_argument_sign),
            (add_argument_index, add_argument_sign))
        ) = first_watch_update {
            self.clauses[id].set_watch(0, watch_index);
            self.set_argument_watch(false, remove_argument_index, id, remove_argument_sign);
            self.set_argument_watch(true, add_argument_index, id, add_argument_sign);
        };

        if let Some(
            (watch_index,
            (remove_argument_index, remove_argument_sign),
            (add_argument_index, add_argument_sign))
        ) = second_watch_update {
            self.clauses[id].set_watch(1, watch_index);
            self.set_argument_watch(false, remove_argument_index, id, remove_argument_sign);
            self.set_argument_watch(true, add_argument_index, id, add_argument_sign);
        };

        result
    }

    #[inline]
    pub fn get_max_clause_index(&self) -> usize {
        self.base.clauses.len() - 1
    }

    #[inline]
    pub fn get_verifier(&self, id: usize) -> &VerifierType {
        self.clauses[id].get_verifier()
    }

    #[inline]
    pub fn set_argument_watch(&mut self, add: bool, index: usize, clause_id: usize, sign: bool) {
        let argument = &mut self.arguments[index];
        if add {
            argument.add_watched_in(clause_id, sign);
        }
        else {
            argument.remove_watched_in(clause_id, sign)
        }
    }

    #[inline]
    pub fn get_argument_value(&self, index: usize) -> Option<bool> {
        self.arguments[index].get_value(self.iteration)
    }

    #[inline]
    pub fn set_argument_value(&mut self, index: usize, value: bool,) {
        self.arguments[index].set_value(value, self.iteration, &mut self.clauses_to_check);
    }

    #[inline]
    pub fn set_clause_used(&self, index: usize) -> bool {
        self.base.clauses[index].set_used()
    }

    #[inline]
    pub fn clause_is_not_deleted_for(&self, clause_index: usize, verification_index: usize) -> bool {
        if let Some(deletion_index) = self.clauses[clause_index].deleted_at() {
            deletion_index <= verification_index
        }
        else { 
            true   
        }        
    }

    #[inline]
    pub fn get_attacked_by(&self, argument_index: usize) -> &Vec<usize> {
        self.base.arguments[argument_index].get_attacked_by()
    }
}