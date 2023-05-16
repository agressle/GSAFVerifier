use std::collections::VecDeque;
use crate::instance_view::InstanceView;
use crate::SUPERVISOR;

pub fn rup_verify(index: Option<usize>, instance: &mut InstanceView) -> bool {
    let mut assignments_todo = VecDeque::new();

    //If we are not handling the empty clause, we start by assigning the argument of the clause we are working on
    if let Some(index) = index {
        for member in instance.get_clause_members(index) {
            assignments_todo.push_back(*member);
        }
    }

    let verification_index = if index.is_some() { index.unwrap() } else { instance.get_max_clause_index() + 1 };
    let mut propagated: bool;
    loop {
        propagated = false;
        while let Some((argument_index, value)) = assignments_todo.pop_front() {
            let current_value = instance.get_argument_value(argument_index);
            if let Some(current_value) = current_value {
                if current_value != value {
                    return true;
                }
            }
            else {
                instance.set_argument_value(argument_index, value);
            }
        }

        while let Some(clause_index) = instance.get_next_clause_to_check() {
            if clause_index < verification_index && instance.clause_is_not_deleted_for(clause_index, verification_index) {
                let result = instance.check_clause_propagation(clause_index);
                if let Some((argument_index, sign)) = result {
                    propagated = true;
                    assignments_todo.push_back((argument_index, sign));
                    if instance.set_clause_used(clause_index) {
                        SUPERVISOR.get().unwrap().add_clause_to_check(clause_index);
                    }
                    break;
                };
            }
        }

        if !propagated {
            break;
        }
    }

    return false;
}


