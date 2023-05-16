use clap::builder::PossibleValue;
use clap::ValueEnum;

use crate::parsers::file_reader::LineInterpreter;


///Represents the semantics of the instance
#[derive(Clone)]
pub enum Semantics {
    Admissible,
    Stable
}

pub enum VerifierType {
    RUP,
    Admissibility(Option<usize>),
    Stability
}

pub trait LineInterpreterDeterminer {
    fn get_proof_parser_type(&self) -> dyn LineInterpreter;
}

impl ValueEnum for Semantics {
    fn value_variants<'a>() -> &'a [Self] {
        &[Semantics::Admissible, Semantics::Stable]
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        match self {
            Semantics::Admissible => Some(PossibleValue::new("Admissible")),
            Semantics::Stable => Some(PossibleValue::new("Stable"))
        }
    }
}

impl Semantics {

    /// Returns the starts index in the proof line and verifier to use for a given type of semantics and proof line.
    pub fn get_verifier(&self, line: &String) -> Option<(usize, VerifierType)> {
        match self {
            Semantics::Admissible => Self::get_verifier_admissible(line),
            Semantics::Stable => Self::get_verifier_stable(line),
        }
    }

    fn get_verifier_stable(line: &String) -> Option<(usize, VerifierType)> {
        match line {
            l if l.starts_with("i ") => Some((2, VerifierType::Stability)),
            _ => Some((0, VerifierType::RUP))
        }
    }

    fn get_verifier_admissible(line: &String) -> Option<(usize, VerifierType)> {
        match line {
            l if l.starts_with("i") => {

                let first_space = l.find(' ');
                if let Some(first_space) = first_space {

                    if first_space == 1 {
                        Some((2, VerifierType::Admissibility(None)))
                    }
                    else {
                        let index = l[1..=first_space].parse::<usize>();
                        if let Ok(index) = index {
                            Some((first_space + 1, VerifierType::Admissibility(Some(index))))
                        }
                        else {
                            None
                        }
                    }
                }
                else {
                    None
                }
            },
            _ => Some((0, VerifierType::RUP))
        }
    }
}