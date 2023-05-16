use std::sync::{Condvar, Mutex};
use std::{panic, thread};
use crate::instance_view::InstanceView;
use crate::{INSTANCE, SUPERVISOR};
use crate::semantics::VerifierType;
use crate::verifiers::admissibility_verifier::admissibility_verify;
use crate::verifiers::rup_verifier::rup_verify;
use crate::verifiers::stability_verifier::stability_verify;

pub enum Work {
    // Tells the worker to work on the clause with the given index.
    Work(Option<usize>),
    // Tells the worker to stall.
    Stall,
    // Tells the worker to terminate.
    Finished
}

pub struct Worker {
    id: usize,
    park: (Mutex<(usize, usize)>, Condvar)
}

impl Worker {

    pub fn new(id: usize) -> Worker {
        let mut worker = Worker {
            id,
            park: (Mutex::new((0, 0)), Condvar::new())
        };
        worker.do_work();
        worker
    }

    pub fn wake_up(&self) {
        let (mutex, cvar) = &self.park;
        let mut guard = mutex.lock().unwrap();
        let (ref mut current, ref seen) = *guard;
        *current = *seen + 1;
        cvar.notify_one();
    }

    pub fn stall(&self) {
        let (mutex, cvar) = &self.park;
        loop {
            let mut guard = mutex.lock().unwrap();
            let (ref current, ref mut seen) = *guard;
            if *seen == *current {
                if cvar.wait(guard).is_err() {
                    return;
                }
            }
            else {
                *seen = *current;
                return;
            }
        }
    }

    fn do_work(&mut self) {
        let id = self.id;
        thread::spawn(move ||{
            let result = panic::catch_unwind(|| {
                let mut instance = InstanceView::new(INSTANCE.get().unwrap());

                //Main work loop
                loop {
                    match SUPERVISOR.get().unwrap().get_work(id) {
                        Work::Work(id) => {
                            let verifier = if let Some(id) = id { instance.get_verifier(id) } else { &VerifierType::RUP };
                            let result = match verifier {
                                VerifierType::Admissibility(index) => {
                                    let index = *index;
                                    admissibility_verify(id.unwrap(), &mut instance, index) },
                                VerifierType::RUP => {
                                        instance.reset();
                                        rup_verify(id, &mut instance)
                                    },
                                VerifierType::Stability => { stability_verify(id.unwrap(), &mut instance) }
                            };
                            SUPERVISOR.get().unwrap().worker_finished(id, result);
                        },
                        Work::Stall => {
                            let worker = &SUPERVISOR.get().unwrap().get_workers()[id];
                            worker.stall();
                        },
                        Work::Finished => break //Exit main work loop
                    }
                }
            });

            if result.is_err() {
                SUPERVISOR.get().unwrap().worker_error_occurred();
            }
        });
    }
}