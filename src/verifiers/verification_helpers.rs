use std::collections::HashSet;
use crate::instance_view::InstanceView;

pub fn contains_clause_witnesses(clause_members: &HashSet<usize>, attacked_by: &Vec<usize>, instance: &InstanceView) -> bool {
    'attack_loop: for attack_index in attacked_by {
        for (attack_member_index, _) in &instance.get_clause_members(*attack_index)[1..] {
            if clause_members.contains(attack_member_index) {
                continue 'attack_loop;
            }
        }
        return false;
    }

    return true;
}