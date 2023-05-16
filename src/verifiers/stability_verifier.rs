use std::collections::HashSet;
use crate::instance_view::InstanceView;
use crate::verifiers::verification_helpers::contains_clause_witnesses;

pub fn stability_verify(index: usize, instance: &InstanceView) -> bool {

    let clause_members = instance.get_clause_members(index);
    let (stability_argument_index, _) = clause_members.first().unwrap();
    let attacked_by = instance.get_attacked_by(*stability_argument_index);

    //Implicit clauses for stability only contain positive literals
    for (_, sign) in clause_members {
        if !sign {
            return false;
        }
    }

    let clause_support : HashSet<usize> = clause_members.iter().map(|(index, _)| *index).collect();
    let result = contains_clause_witnesses(&clause_support, attacked_by, instance);

    if result {
        for attack in attacked_by {
            instance.set_clause_used(*attack);
        }
    }

    return result;
}


