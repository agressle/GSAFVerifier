mod argument_base;
mod argument_view;
mod clause_base;
mod clause_view;
mod instance_base;
mod instance_view;
pub mod parsers;
pub mod semantics;
mod supervisor;
mod verifiers;
mod worker;

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use signal_hook::consts::{SIGALRM, SIGINT, SIGTERM, SIGUSR1};
use signal_hook::iterator::{Signals, SignalsInfo};
use once_cell::sync::OnceCell;
use crate::supervisor::{Supervisor, SupervisorState};
use crate::instance_base::InstanceBase;
use crate::semantics::Semantics;

//Constants
pub const EXIT_CODE_OK: u8 = 0;
pub const EXIT_CODE_SETUP_SIGNALS: u8 = 2;
pub const EXIT_CODE_SIGNALS: u8 = 4;
pub const EXIT_CODE_INSTANCE: u8 = 8;
pub const EXIT_CODE_TIMEOUT: u8 = 32;
pub const EXIT_CODE_FAILURE: u8 = 64;
pub const EXIT_CODE_UNEXPECTED: u8 = 128;

// Whether or not we should terminate
static DO_WORK : AtomicBool = AtomicBool::new(true);

//The supervisor and schedules the worker threads
pub static INSTANCE: OnceCell<Box<InstanceBase>> = OnceCell::new();

// The instance we are working on
pub static SUPERVISOR : OnceCell<Box<Supervisor>> = OnceCell::new();

/// Returns Ok(false) if threads should continue working and Err(String) if the should terminate as soon as possible.
pub fn should_stop() -> Result<bool, String> {
    match DO_WORK.load(Ordering::Acquire) {
        true => Ok(false),
        false => Err("Interrupted".to_string())
    }
}

pub fn verify(number_of_threads: u16, framework_path: PathBuf, description_path: Option<PathBuf>, required_arguments_path: Option<PathBuf>, proof_path: PathBuf, semantics: Semantics, complete: bool) -> (String, u8) {

    //Setup signal handlers
    let signals = Signals::new(&[SIGINT, SIGTERM, SIGALRM, SIGUSR1]);
    if let Err(err) = signals
    {
        eprintln!("Failed to setup signal handlers: {}", err);
        quit::with_code(EXIT_CODE_SETUP_SIGNALS);
    }

    let result = verify_internal(signals.unwrap(), number_of_threads, framework_path, description_path, required_arguments_path, proof_path, semantics, complete);
    DO_WORK.store(false, Ordering::Release);
    return result;
}

fn verify_internal(mut signals: SignalsInfo, number_of_threads: u16, framework_path: PathBuf, description_path: Option<PathBuf>, required_arguments_path: Option<PathBuf>, proof_path: PathBuf, semantics: Semantics, complete: bool) -> (String, u8) {

    if SUPERVISOR.set(Box::new(Supervisor::new())).is_err() {
        return (format!("Failed to initialize supervisor."), EXIT_CODE_UNEXPECTED);
    }

    SUPERVISOR.get().unwrap().start(number_of_threads, framework_path, description_path, required_arguments_path, proof_path, semantics, complete);
    // Handle signals
    for sig in signals.forever() {
        return match sig {
            SIGALRM => (format!("Timeout reached."), EXIT_CODE_TIMEOUT),
            SIGUSR1 => //Worker Finished
                {
                    match SUPERVISOR.get().unwrap().get_state() {
                        SupervisorState::NotStarted => (format!("Verification has not been started."), EXIT_CODE_FAILURE),
                        SupervisorState::Working => (format!("Verification has been interrupted unexpectedly."), EXIT_CODE_FAILURE),
                        SupervisorState::ParsingFailed => (format!("Failed to parse the instance: {}", SUPERVISOR.get().unwrap().get_parsing_error_message()), EXIT_CODE_INSTANCE),
                        SupervisorState::RequiredArgumentInconsistent => (format!("The required arguments are inconsistent."), EXIT_CODE_OK),
                        SupervisorState::Finished => (format!("Finished."), EXIT_CODE_OK),
                        SupervisorState::UnexpectedError => (format!("An unexpected error occurred."), EXIT_CODE_UNEXPECTED),
                        SupervisorState::Unknown => (format!("Failed to determine the internal state."), EXIT_CODE_UNEXPECTED)
                    }
                },
            _ => (format!("Interrupted by signal."), EXIT_CODE_SIGNALS) //Something unexpected happened
        };
    }

    (format!("Did not receive signal"), EXIT_CODE_SIGNALS)
}