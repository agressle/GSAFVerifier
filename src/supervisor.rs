use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicUsize, Ordering};
use std::sync::{Mutex};
use std::cmp::min;
use num_enum::IntoPrimitive;
use num_enum::TryFromPrimitive;
use once_cell::sync::OnceCell;
use signal_hook::consts::SIGUSR1;
use crate::instance_base::InstanceBase;
use crate::semantics::Semantics;
use crate::{INSTANCE, should_stop};
use crate::worker::{Work, Worker};


#[derive(Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum SupervisorState {
    NotStarted,
    Working,
    ParsingFailed,
    RequiredArgumentInconsistent,
    Finished,
    UnexpectedError,
    Unknown
}

struct SupervisorData {
    /// The workers that have been stalled due to lazy verification
    stalled_workers: Vec<usize>,
    /// The number of workers deployed
    deployed_worker_counter: u16,
    /// The indices of the clauses that still need to be checked
    to_check: Vec<Option<usize>>,
}

impl SupervisorData {
    pub fn new<>(deployed_worker_counter: u16) -> SupervisorData{
        SupervisorData {
            deployed_worker_counter,
            stalled_workers: Vec::new(),
            to_check: vec![None],
        }
    }
}

pub struct Supervisor {
    /// The state of the supervisor.
    state: Box<AtomicU8>,
    /// The workers that have been spawned.
    workers: OnceCell<Vec<Worker>>,
    /// The data the supervisor is working on.
    data: Mutex<Option<SupervisorData>>,
    /// Whether or not the proof was verified successfully.
    verification_successful: Box<AtomicBool>,
    /// An index of a clause that failed to verify.
    failed_clause_index: Box<AtomicUsize>,
    /// Whether or not the empty clause failed to verify.
    failed_empty_clause_verification: Box<AtomicBool>,
    /// The index of the first clause that needs to be verified.
    first_clause_index_to_verify: Box<AtomicUsize>,
    /// The error message that occurred during instance parsing.
    parsing_error_message: OnceCell<String>
}

impl Supervisor {

    pub fn get_result(&self) -> Option<(bool, Option<usize>)> {
        match self.get_state() {
            SupervisorState::Finished | SupervisorState::RequiredArgumentInconsistent => {
                if self.verification_successful.load(Ordering::Acquire) {
                    Some((true, None))
                }
                else {
                    if self.failed_empty_clause_verification.load(Ordering::Acquire) {
                        Some((false, None))
                    }
                    else {
                        Some((false, Some(self.failed_clause_index.load(Ordering::Acquire))))
                    }
                }
            },
            SupervisorState::NotStarted | SupervisorState::Working | SupervisorState::ParsingFailed | SupervisorState::UnexpectedError | SupervisorState::Unknown => None
        }
    }

    pub fn new() -> Supervisor {
        Supervisor {
            state: Box::new(AtomicU8::new(SupervisorState::NotStarted.into())),
            workers: OnceCell::new(),
            data: Mutex::new(None),
            verification_successful: Box::new(AtomicBool::new(true)),
            failed_clause_index: Box::new(AtomicUsize::new(0)),
            failed_empty_clause_verification: Box::new(AtomicBool::new(false)),
            first_clause_index_to_verify: Box::new(AtomicUsize::new(0)),
            parsing_error_message: OnceCell::new()
        }
    }

    /// Starts the supervisor in a new thread. Parses the instance, description, proof and required arguments and delegates the work of checking the proof over the provided number of threads. Stops scheduling new work once DO_WORK in main is set to false.
    pub fn start(&'static self, number_of_threads: u16, framework_path: PathBuf, description_path: Option<PathBuf>, required_arguments_path: Option<PathBuf>, proof_path: PathBuf, semantics: Semantics, complete: bool) {

        // Start by parsing the instance
        let instance = InstanceBase::new(&framework_path, &description_path, &required_arguments_path, &proof_path, &semantics);
        if let Err(message) = instance {
            if self.parsing_error_message.set(message).is_err() {
                self.set_state_and_exit(SupervisorState::UnexpectedError);
            }
            else {
                self.set_state_and_exit(SupervisorState::ParsingFailed);
            }
            return;
        }

        let instance = instance.unwrap();

        if !instance.is_required_arguments_consistent() {
            self.verification_successful.store(true, Ordering::Release);
            self.set_state_and_exit(SupervisorState::RequiredArgumentInconsistent);
            return;
        }

        let number_of_clauses = instance.clauses.len();
        self.first_clause_index_to_verify.store(instance.proof_start, Ordering::Release);

        //This should not cause an error, as no one else should have used the lock yet
        if let Ok(mut data) = self.data.lock()
        {
            assert!(data.is_none());

            let number_of_workers = min(number_of_threads as usize,number_of_clauses) as u16; //Cast is fine, as we take the minimum and thus are never larger then u16
            *data = Some(SupervisorData::new(number_of_workers));

            if complete {
                let data = data.as_mut().unwrap();
                for clause in &instance.clauses[instance.proof_start..] {
                    clause.set_used();
                    data.to_check.push(Some(clause.get_index()));
                }
            }

            if INSTANCE.set(Box::new(instance)).is_err() {
                self.set_state_and_exit(SupervisorState::UnexpectedError);
                return;
            }

            // Setup
            let mut workers = Vec::new();
            for i in 0..number_of_workers as usize {
                let worker = Worker::new(i);
                workers.push(worker);
            }
            if self.workers.set(workers).is_err() {
                self.set_state_and_exit(SupervisorState::UnexpectedError);
                return;
            }

            self.set_state(SupervisorState::Working);
        }
        else
        {
            self.set_state_and_exit(SupervisorState::UnexpectedError);
        }
    }

    pub fn get_state(&self) -> SupervisorState {
        match SupervisorState::try_from(self.state.load(Ordering::Acquire)) {
            Ok(state) => state,
            Err(_) => SupervisorState::Unknown
        }
    }

    fn set_finished(&self) {
        _ = self.state.compare_exchange(SupervisorState::Working.into(), SupervisorState::Finished.into(), Ordering::AcqRel, Ordering::Relaxed); //Nothing to be done if failed, as we were no longer working then anyway.
        if let Err(_) = signal_hook::low_level::raise(SIGUSR1) {
            println!("Failed to raise signal during verification.");
            quit::with_code(1);
        };
    }

    fn set_state(&self, state: SupervisorState)
    {
        self.state.store(state.into(), Ordering::Release);
    }

    fn set_state_and_exit(&self, state: SupervisorState) {
        self.set_state(state);
        if let Err(_) = signal_hook::low_level::raise(SIGUSR1) {
            println!("Failed to raise signal during verification.");
            quit::with_code(1);
        };
    }

    pub fn worker_error_occurred(&self) {
        self.set_state_and_exit(SupervisorState::UnexpectedError);
    }

    pub fn get_work(&self, worker_index: usize) -> Work {
        if let Err(_) = should_stop() {
            Work::Finished
        }
        else {

            if self.get_state() != SupervisorState::Working {
                return Work::Finished;
            }

            if let Ok(mut guard) = self.data.lock() {
                let ref mut data = *guard.as_mut().unwrap();

                //We first see if we can schedule any clauses
                if let Some(index) = data.to_check.pop() {
                    Work::Work(index)
                }
                else {
                    // We do not know of any clauses that need verification yet, thus we tell the worker to stall.
                    // If this would cause the last worker to stall, all clauses have been verified and we are finished.
                    if data.stalled_workers.len() == (data.deployed_worker_counter - 1) as usize {
                        self.set_finished();
                        for worker_id in &data.stalled_workers {
                            let worker = &self.workers.get().unwrap()[*worker_id];
                            worker.wake_up();
                        }

                        Work::Finished
                    }
                    else {

                        if self.get_state() == SupervisorState::Working {
                            data.stalled_workers.push(worker_index);
                            Work::Stall
                        }
                        else {
                            Work::Finished
                        }
                    }
                }
            } else {
                self.set_state_and_exit(SupervisorState::UnexpectedError);
                Work::Finished
            }
        }
    }

    pub fn worker_finished(&self, clause_index: Option<usize>, result: bool) {
        if !result {
            // Clause failed to verify
            self.verification_successful.store(false, Ordering::Release);
            if let Some(index) = clause_index {
                self.failed_clause_index.store(index, Ordering::Release);
            }
            else {
                self.failed_empty_clause_verification.store(true, Ordering::Release);
            }

            self.set_finished();
        }
    }

    pub fn add_clause_to_check(&self, clause_index: usize) {

        if clause_index < self.first_clause_index_to_verify.load(Ordering::Acquire) {
            return; //We dont verify clauses that are not part of the proof.
        }

        if let Ok(mut guard) = self.data.lock() {
            let ref mut data = *guard.as_mut().unwrap();
            data.to_check.push(Some(clause_index));
            //Wake up another worker if available to schedule it
            if let Some(worker_id) = data.stalled_workers.pop()
            {
                let worker = &self.workers.get().unwrap()[worker_id];
                worker.wake_up();
            }
        } else {
            self.worker_error_occurred();
        }
    }

    pub fn get_workers(&self) -> &Vec<Worker> {
        &self.workers.get().unwrap()
    }

    pub fn get_parsing_error_message(&self) -> &String {
        self.parsing_error_message.get().unwrap()
    }
}