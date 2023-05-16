extern crate core;
use std::{env, path::PathBuf, time::SystemTime};
use clap::{Parser};
use nix::unistd::alarm;
use verifier::{verify, EXIT_CODE_OK, EXIT_CODE_UNEXPECTED, INSTANCE, SUPERVISOR, semantics::Semantics };

#[derive(Parser)]
#[command(
    author = "Alexander Greßler <agressle@dbai.tuwien.ac.at>",
    version = env!("CARGO_PKG_VERSION"),
    about = "A verifier of RUP proofs for unsatisfiability results of SETAF instances."
)]
struct Args
{
    #[arg(
        short = 'i',
        long = "instance",
        help = "A file that contains the encoding of the instance.",
        value_name = "FILE",
        required = true)
    ]
    ///The path to the instance file.
    instance: PathBuf,

    #[arg(
        short = 'd',
        long = "description",
        help = "A file that contains the instance description.",
        value_name = "FILE",
        required = false)
    ]
    ///The path to the instance description file.
    description: Option<PathBuf>,

    #[arg(
        short = 'p',
        long = "proof",
        help = "A file that contains the proof.",
        value_name = "FILE",
        required = true)
    ]
    ///The path to the proof file.
    proof: PathBuf,

    #[arg(
        short = 'r',
        long = "required",
        help = "A file that contains the required arguments.",
        value_name = "FILE",
        required = false)
    ]
    required: Option<PathBuf>,

    #[arg(
        short = 's',
        long = "semantics",
        help = "The semantics that the proof adheres to.",
        required = true,
        value_enum)
    ]
    semantics: Semantics,

    #[arg(
        short = 't',
        long = "timeout",
        help = "The timeout in seconds 0 for no limit.",
        required = false,
        value_parser = clap::value_parser!(u32),
        default_value_t = 0)
    ]
    timeout: u32,

    #[arg(
        short = 'w',
        long = "threads",
        help = "The number of verifier threads to use.",
        required = false,
        value_parser = clap::value_parser!(u16).range(1..),
        default_value_t = 1)
    ]
    thread: u16,

    #[arg(
        short = 'u',
        long = "used",
        help = "When provided, indices of the attacks and clauses that where used during verification are printed.",
        required = false,
        default_value_t = false)
    ]
    used: bool,

    #[arg(
        short = 'c',
        long = "complete",
        help = "When provided, all clauses of the proof are verified. Otherwise, only those used for propagation are verified.",
        required = false,
        default_value_t = false)
    ]
    complete: bool
}

#[quit::main]
fn main() {

    let start_time : SystemTime = SystemTime::now();

    let args = Args::parse();

    if args.timeout != 0 {
        alarm::set(args.timeout);
    }

    let (result_message, exit_code) = verify(args.thread, args.instance, args.description, args.required, args.proof, args.semantics, args.complete);
    let end_time = SystemTime::now();

    println!("{}", result_message);
    let duration = end_time.duration_since(start_time).unwrap();
    println!("Time: {},{}s", duration.as_secs(), duration.subsec_millis());

    if exit_code == EXIT_CODE_OK {
        let instance = INSTANCE.get().unwrap();
        let result = SUPERVISOR.get().unwrap().get_result();
        if let Some((verification_successful, clause_index)) = result {
            if verification_successful {
                println!("Proof verified successfully.");
                if args.used {
                    let mut first : bool = true;

                    println!("The following attacks (0-based indices) of the instance were used during verification:");
                    for clause in &instance.clauses[..instance.proof_start] {
                        if clause.is_used() {
                            if first {
                                first = false;
                            }
                            else {
                                print!(", ");
                            }
                            print!("{}", clause.get_index())
                        }
                    }

                    println!();
                    println!("The following clauses (0-based indices) of the proof were used during verification:");
                    first = true;

                    for clause in &instance.clauses[instance.proof_start..] {
                        if clause.is_used() {
                            if first {
                                first = false;
                            }
                            else {
                                print!(", ");
                            }
                            print!("{}", clause.get_index() - instance.proof_start)
                        }
                    }
                    println!();
                }
            }
            else {
                print!("Proof verification failed for ");
                if let Some(clause_index) = clause_index {
                    println!("the proof clause with (0-based) index {}.", clause_index - instance.proof_start)
                }
                else {
                    println!("the empty clause.")
                }
            }
        }
        else {
            println!("Failed to get result");
            quit::with_code(EXIT_CODE_UNEXPECTED);
        }
    }
    quit::with_code(exit_code);
}