use std::collections::HashSet;
use crate::instance_view::InstanceView;
use crate::verifiers::verification_helpers::contains_clause_witnesses;

pub fn admissibility_verify(index: usize, instance: &InstanceView, attack_index: Option<usize>) -> bool {

    let clause_members = instance.get_clause_members(index);
    let (admissibility_argument_index, admissibility_argument_sign) = clause_members.first().unwrap();

    if *admissibility_argument_sign {
        return false;
    }

    let attacks = instance.get_attacked_by(*admissibility_argument_index);
    let clause_members : HashSet<usize> = clause_members.iter().map(|(index, _) | *index).collect();

    return if let Some(attack_index) = attack_index {
        if !attacks.contains(&attack_index) {
            return false;
        }

        admissibility_verify_for_attack(&clause_members, attack_index, instance)
    }
    else {

        for attack_index in attacks {
            if admissibility_verify_for_attack(&clause_members, *attack_index, instance) {
                instance.set_clause_used(*attack_index);
                return true;
            }
        }
        false
    }
}

fn admissibility_verify_for_attack(clause_members: &HashSet<usize>, attack_index: usize, instance: &InstanceView) -> bool {

    for (attack_member_index, _) in &instance.get_clause_members(attack_index)[1..] {
        let attacked_by = instance.get_attacked_by(*attack_member_index);
        if !contains_clause_witnesses(clause_members, attacked_by, instance) {
            return false;
        }
    }

    for (attack_member_index, _) in &instance.get_clause_members(attack_index)[1..] {
        let attacked_by = instance.get_attacked_by(*attack_member_index);
        for attack in attacked_by {
            instance.set_clause_used(*attack);
        }
    }

    return true;
}