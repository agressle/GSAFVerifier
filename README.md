# GSAFVerifier
A tool for verifying RUP-style inconsistency proofs for abstract argumentation frameworks with collective attacks (SETAF).

## 1 Installation

Requires the rust toolchain (c.f. https://www.rust-lang.org and https://rustup.rs). Run 'cargo build –release' to build. 

## Usage 

Usage: verifier [OPTIONS] --instance <FILE> --proof <FILE> --semantics <SEMANTICS>

Options:
  -i, --instance <FILE>        A file that contains the encoding of the instance.
  -d, --description <FILE>     A file that contains the instance description.
  -p, --proof <FILE>           A file that contains the proof.
  -r, --required <FILE>        A file that contains the required arguments.
  -s, --semantics <SEMANTICS>  The semantics that the proof adheres to. [possible values: Admissible, ConflictFree, Stable]
  -t, --timeout <TIMEOUT>      The timeout in seconds 0 for no limit. [default: 0]
  -w, --threads <THREAD>       The number of verifier threads to use. [default: 1]
  -u, --used                   When provided, indices of the attacks and clauses that where used during verification are printed.
  -c, --complete               When provided, all clauses of the proof are verified. Otherwise, only those used for propagation are verified.
  -h, --help                   Print help
  -V, --version                Print version
