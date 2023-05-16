use crate::clause_base::ClauseBase;
use crate::clause_view::WatchUpdateResult::*;
use crate::instance_view::InstanceView;
use crate::semantics::VerifierType;

/// Represents a worker threads view of a clause, with its current watches for this thread.

pub struct ClauseView<'a> {
    base: &'a ClauseBase,
    watches: [usize; 2]
}

enum WatchUpdateResult {
    Success((usize,(usize, bool), (usize, bool))),
    AlreadySatisfied,
    Failed
}

impl ClauseView<'_> {

    pub fn new<'a>(base: &'a ClauseBase, instance: &mut InstanceView) -> ClauseView<'a> {
        let mut view = ClauseView {
            base,
            watches: [0, 0]
        };

        let clause_index = base.get_index();
        let (index, sign) = base.get_members()[0];
        instance.set_argument_watch(true, index, clause_index, sign);

        if base.get_number_of_members() > 1 {
            let (index, sign) = base.get_members()[1];
            view.watches[1] = 1;
            instance.set_argument_watch(true, index, clause_index, sign);
        }

        view
    }

    #[inline]
    pub fn get_verifier(&self) -> &VerifierType {
          self.base.get_verifier()
    }

    pub fn check_propagation(&self, instance: &InstanceView) -> (Option<(usize, bool)>, Option<(usize, (usize, bool), (usize, bool))> , Option<(usize, (usize, bool), (usize, bool))>) {

        if self.base.get_number_of_members() == 1 {
            return (Some(self.base.get_member(0)), None, None);
        }

        //Determine current values
        let (first_watch_index, first_watch_sign) = self.base.get_member(self.watches[0]);
        let first_watch_value = instance.get_argument_value(first_watch_index);
        let first_watch_value_set =  if let Some(value) = first_watch_value {
            if value == first_watch_sign { return (None, None, None); }
            true
        }
        else {
            false
        };

        let (second_watch_index, second_watch_sign) = self.base.get_member(self.watches[1]);
        let second_watch_value = instance.get_argument_value(second_watch_index);
        let second_watch_value_set =  if let Some(value) = second_watch_value {
            if value == second_watch_sign { return (None, None, None); }
            true
        }
        else {
            false
        };

        //Both watches are not set to the target value if we are here
        //Update every watch that points to an argument that is already set
        let mut first_watch_update: Option<(usize, (usize, bool), (usize, bool))> = None;
        let mut second_watch_update: Option<(usize, (usize, bool), (usize, bool))> = None;

        if first_watch_value_set {
            match self.update_watch(instance, self.watches[0], first_watch_index, first_watch_sign, self.watches[1]) {
                Success(update) => {first_watch_update = Some(update)}, //Successfully updated the watch, nothing more to do for this watch
                AlreadySatisfied => { return (None, first_watch_update, second_watch_update); } //Clause already satisfied, nothing more to do for this clause
                Failed => { return (Some((second_watch_index, second_watch_sign)), first_watch_update, second_watch_update); } //Failed to update first watch -> assert second watch
            }
        }

        let (first_watch, first_watch_index, first_watch_sign) = if let Some((first_watch, _, (first_watch_index,first_watch_sign ))) = first_watch_update { (first_watch, first_watch_index, first_watch_sign) } else { (self.watches[0], first_watch_index, first_watch_sign) };

        if second_watch_value_set {
            match self.update_watch(instance, self.watches[1], second_watch_index, second_watch_sign, first_watch) {
                Success(update) => {second_watch_update = Some(update)}, //Successfully updated the watch, nothing more to do for this watch
                AlreadySatisfied => { return (None, first_watch_update, second_watch_update); } //Clause already satisfied, nothing more to do for this clause
                Failed => { return (Some((first_watch_index, first_watch_sign)), first_watch_update, second_watch_update); } //Failed to update second watch -> assert first watch
            }
        }

        (None, first_watch_update, second_watch_update)
    }

    fn update_watch(&self, instance: &InstanceView, initial_index: usize, current_argument_index: usize, current_argument_sign: bool, other_index: usize) -> WatchUpdateResult {

        let mut running_index = (initial_index + 1) % self.base.get_number_of_members();

        while initial_index != running_index {
            if running_index != other_index {
                let (argument_index, argument_sign) = self.base.get_member(running_index);
                let argument_value = instance.get_argument_value(argument_index);

                if let Some(value) = argument_value {
                    //If the value equals the target value, we are done for the clause. Otherwise we continue with the next iteration
                    if value == argument_sign {
                        return AlreadySatisfied;
                    }
                }
                else {
                    //Argument is not yet set => change watch to here
                    return Success((running_index, (current_argument_index, current_argument_sign), (argument_index, argument_sign)));
                }

            }
            running_index = (running_index + 1) % self.base.get_number_of_members();
        }

        //Failed to find new watch value. initial_index = running_index, thus no update to argument watches is necessary
        Failed
    }

    #[inline]
    pub fn set_watch(&mut self, watch: usize, index: usize) {
        self.watches[watch] = index;
    }

    #[inline]
    pub fn deleted_at(&self) -> Option<usize> {
        self.base.deleted_at()
    }

    #[inline]
    pub fn get_members(&self) -> &Vec<(usize, bool)> {
        &self.base.get_members()
    }


}