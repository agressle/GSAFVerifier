use std::collections::HashMap;
use std::path::PathBuf;
use crate::argument_base::ArgumentBase;
use crate::clause_base::ClauseBase;
use crate::parsers::file_reader::FileReader;
use crate::semantics::Semantics;
use crate::should_stop;

/// Represents the instance, i.e. framework and proof, to verify.
pub struct InstanceBase {

    /// The arguments of the instance.
    pub arguments: Vec<ArgumentBase>,

    /// Which arguments have been required with the respective sign.
    pub required_arguments: Vec<(usize, bool)>,

    /// The attacks of the instance.
    pub clauses: Vec<ClauseBase>,

    /// The indices of the unit clauses of the instance.
    pub unit_clauses: Vec<usize>,

    /// The id of the first clause that is part of the proof.
    pub proof_start: usize
}

impl InstanceBase {

    /// Creates a new instance based on input data.
    pub fn new<'a>(framework_path: &PathBuf, description_path: &Option<PathBuf>, required_arguments_path: &Option<PathBuf>, proof_path: &PathBuf, semantics: &Semantics) -> Result<InstanceBase, String> {

        //Read the framework
        let (arguments, mut clauses, argument_names, mut unit_clauses) = Self::parse_framework(framework_path, description_path)?;

        let number_of_attacks = clauses.len();

        //Read required arguments
        let required_arguments = match required_arguments_path {
            Some(path) => Self::parse_required(path, arguments.len(), argument_names)?,
            None => Vec::new()
        };

        //Read proof
        match Self::parse_proof(proof_path, semantics, &arguments, &mut clauses, &mut unit_clauses) {
            None => Ok(InstanceBase {arguments, required_arguments, clauses, unit_clauses, proof_start: number_of_attacks }),
            Some(message) => Err(message)
        }
    }

    /// Reads the framework and description files and creates the arguments and clauses.
    fn parse_framework<'a>(framework_path: &PathBuf, description_path: &Option<PathBuf>,) -> Result<(Vec<ArgumentBase>, Vec<ClauseBase>, HashMap<String, Option<usize>>, Vec<usize>), String> {

        let mut instance_reader = FileReader::new(framework_path)?;

        //Read preamble
        let preamble = instance_reader.next();
        if let None = preamble {
            return Err(format!("The supplied instance contains no preamble"));
        }
        else if let Some(Err(message)) = preamble {
            return Err(format!("An error occurred while reading the preamble: {}", message));
        }
        let preamble = preamble.unwrap().unwrap();
        let split: Vec<&str> = preamble.split(" ").collect();
        if split.len() != 3 || split[2] != "0" {
            return Err(format!("Preamble is malformed: {}", preamble));
        }

        //Parse number of arguments
        let num_arguments = split[0].parse();
        if let Err(_) = num_arguments {
            return Err(format!("The number of arguments in the preamble is invalid: {}", split[0]));
        }
        let num_arguments :usize = num_arguments.unwrap();

        //Parse number of attacks
        let num_attacks = split[1].parse();
        if let Err(_) = num_attacks {
            return Err(format!("The number of attacks in the preamble is invalid: {}", split[1]));
        }

        let num_attacks :usize = num_attacks.unwrap();

        let mut arguments= vec![ArgumentBase::new(); num_arguments];
        let mut attacks= Vec::with_capacity(num_attacks);
        let mut unit_clauses = Vec::new();
        for id in 0 .. num_attacks {
            attacks.push(ClauseBase::new(id))
        }

        //Set the indices of the arguments
        for (index, argument) in arguments.iter_mut().enumerate() {
            argument.set_id(index);
        }

        //Read the attacks
        let mut argument_occurrence_watch = vec![0 as usize; num_arguments]; //Used to make sure that every argument is only contained once in every clause
        let mut attacks_iter = attacks.iter_mut();
        for line in instance_reader {

            should_stop()?;

            //Check that reading succeeded
            if let Err(message) = line {
                return Err(format!("An error occurred while reading the instance: {}", message))
            }

            //Check that we dont exceed the specified number of attacks
            let att = attacks_iter.next();
            if let None = att {
                return Err(format!("Instance contains more attacks than specified in the preamble"))
            }

            let att = att.unwrap();

            let line = line.unwrap();
            let split : Vec<&str> = line.split(" ").collect();

            if split.len() < 3 || *split.last().unwrap() != "0" { //We need at least the attacked argument, at least one attacker and the trailing 0
                return Err(format!("The attack '{}' is malformed", line))
            }
            let mut split = split.into_iter();

            //Read the attacked argument
            let attacked_arg_str = split.next().unwrap();
            let attacked_arg_number = attacked_arg_str.parse();
            if let Err(_) = attacked_arg_number {
                return Err(format!("The attacked argument index '{}' is invalid in line '{}'", attacked_arg_str, line));
            }
            let attacked_arg_number: usize = attacked_arg_number.unwrap();
            if attacked_arg_number == 0 || attacked_arg_number > num_arguments {
                return Err(format!("The attacked argument index '{}' is invalid in line '{}'", attacked_arg_str, line));
            }
            let attacked_arg_index = attacked_arg_number - 1;
            att.add_member(attacked_arg_index, false);
            arguments[attacked_arg_index].add_attacked_by(att.get_index());

            //Read the remaining attack members
            for attack_member_str in split {
                //Parse the attack member
                let attack_member_number = attack_member_str.parse();
                if let Err(_) = attack_member_number {
                    return Err(format!("The attack '{}' contains an invalid attack member '{}'", line, attack_member_str));
                }
                let attack_member_number: usize = attack_member_number.unwrap();

                //If 0 we assume we have reached the end and exit the loop
                if attack_member_number == 0 {
                    break;
                }

                if attack_member_number > num_arguments {
                    return Err(format!("The attack '{}' refers to an invalid attack member '{}'", line, attack_member_str));
                }
                let attack_member_index :usize = attack_member_number - 1;
                let arg_occurrence = argument_occurrence_watch.get_mut(attack_member_index).unwrap();
                if *arg_occurrence < att.get_index() + 1 {
                    att.add_member(attack_member_index, false);
                    *arg_occurrence = att.get_index() + 1;
                }
            }

            if att.get_number_of_members() == 1 {
                unit_clauses.push(att.get_index());
            }
        }

        //Check that there are no attacks left after reading the instance
        if attacks_iter.next().is_some() {
            return Err(format!("Instance contains more attacks than specified in the preamble"))
        }

        //Read Description file
        let mut argument_index_to_names_map: HashMap<String, Option<usize>> = HashMap::new(); //A mapping from names to indices or None is the name occurs multiple time
        if let Some(description_path) = description_path
        {
            let instance_reader = match FileReader::new(&description_path) {
                Ok(instance) => instance,
                Err(message) => return Err(message)
            };

            for line in instance_reader {

                should_stop()?;

                //Check that reading succeeded
                if let Err(message) = line {
                    return Err(format!("An error occurred while reading the description file: {}", message))
                }

                let line = line.unwrap();
                let split : Vec<&str> = line.split(" ").collect();

                if split.len() < 2 {
                    return Err(format!("The description line '{}' is malformed", line))
                }
                let description_argument_number = split[0].parse();
                if let Err(_) = description_argument_number {
                    return Err(format!("The description line '{}' references an invalid argument '{}'", line, split[0]));
                }
                let description_argument_number: usize = description_argument_number.unwrap();
                let description_argument_index: usize = description_argument_number - 1;
                if description_argument_number == 0 || description_argument_number > num_arguments {
                    return Err(format!("The description line '{}' references an invalid argument '{}'", line, split[0]));
                }

                let argument_name = &line[split[0].len() + 1 ..]; //Length of the number + 1 space
                if let Some(entry) = argument_index_to_names_map.get_mut(argument_name) {
                    *entry = None;
                }
                else {
                    //We have not seen an argument with that name
                    argument_index_to_names_map.insert(argument_name.to_string(), Some(description_argument_index));
                }
            }
        }

        Ok((arguments, attacks, argument_index_to_names_map, unit_clauses))
    }

    /// Parses the required arguments file.
    fn parse_required(required_arguments_path: &PathBuf, number_of_arguments: usize, argument_names: HashMap<String, Option<usize>>) -> Result<Vec<(usize, bool)>, String> {
        let mut required_arguments : Vec<(usize, bool)> = Vec::new();
        let instance_reader = match FileReader::new(&required_arguments_path) {
            Ok(instance) => instance,
            Err(message) => return Err(message)
        };

        for line in instance_reader {

            should_stop()?;

            //Check that reading succeeded
            if let Err(message) = line {
                return Err(format!("An error occurred while reading the required arguments file: {}", message))
            }

            let line = line.unwrap();
            let split : Vec<&str> = line.split(" ").collect();
            let required_argument = match split.len() {
                1 => //Just the argument number
                    {
                        let is_negative = split[0].starts_with("-");
                        let argument_string =
                            if is_negative {
                                &(split[0])[1..]
                            }
                            else {
                                &split[0]
                            };

                        let argument_number = argument_string.parse();
                        if let Err(_) = argument_number {
                            return Err(format!("The required argument file references argument number '{}' that is invalid", argument_string));
                        }
                        let argument_number :usize = argument_number.unwrap();
                        if argument_number == 0 || argument_number > number_of_arguments {
                            return Err(format!("The required argument file references argument number '{}' that is invalid", argument_string));
                        }
                        (argument_number - 1, !is_negative)
                    },
                _ => { //The argument name
                        if split[0] != "s" {
                            return Err(format!("The line '{}' in the arguments file is malformed", line))
                        }
                        let is_negative = split[1].starts_with("-");
                        let argument_string =
                            if is_negative {
                                &(split[1])[1..]
                            }
                            else {
                                &split[1]
                            };

                        let entry = argument_names.get(argument_string);
                        if let Some(entry) = entry {
                            if let Some(index) = entry {
                                (*index, !is_negative)
                            }
                            else {
                                return Err(format!("The required argument file references argument name '{}' that is not unique", argument_string));
                            }
                        }
                        else {
                            return Err(format!("The required argument file references argument name '{}' that is invalid", argument_string));
                        }
                    }
            };

            required_arguments.push(required_argument);
        }

        Ok(required_arguments)
    }

    /// Parses the proof file and adds the respective clauses.
    fn parse_proof(proof_path: &PathBuf, semantics: &Semantics, arguments: &Vec<ArgumentBase>, clauses: &mut Vec<ClauseBase>, unit_clauses: &mut Vec<usize>) -> Option<String> {

        let number_of_arguments = arguments.len();
        let instance_reader = match FileReader::new(proof_path) {
            Ok(reader) => reader,
            Err(message) => return Some(message)
        };

        let mut argument_occurrence_watch = vec![0 as usize; number_of_arguments]; //Used to make sure that every argument is only contained once in every clause
        let mut clause_string_to_bases : HashMap<String, (Vec<usize>, usize)> = HashMap::new();

        let mut iterator = instance_reader.into_iter();
        let mut found_empty_clause = false;
        while let Some(line) = iterator.next()
        {
            if let Err(message) = should_stop() {
                return Some(message);
            }

            //Check that reading succeeded
            if let Err(message) = line {
                return Some(format!("An error occurred while reading the proof: {}", message))
            }

            let line = line.unwrap();

            if line.starts_with('d') { //Handle clause deletion
                let cleaned_line = &line[2..];
                let deletion_clause = match Self::parse_proof_clause(clauses.len(),&line, cleaned_line, &mut argument_occurrence_watch, number_of_arguments, true) {
                    Ok(clause) => clause,
                    Err(message) => return Some(message)
                };

                if deletion_clause.get_number_of_members() == 0 {
                    return Some(format!("Clause deletion line '{}' cannot be empty", line));
                }

                let current_clause_id = clauses.len();

                //Find the deleted clause an mark it
                match clause_string_to_bases.get_mut(cleaned_line) {
                    Some((clause_vec, index)) if index < &mut clause_vec.len() => {
                        let clause_index = clause_vec.get(*index).unwrap();
                        let clause = clauses.get_mut(*clause_index).unwrap();
                        *index = *index + 1;
                        clause.set_deleted_at(current_clause_id);
                    }
                    Some(_) => return Some(format!("Clause deletion line '{}' references a clause that has already been deleted.", line)),
                    None => return Some(format!("Clause deletion line '{}' references a clause that does not exist", line))
                };

            } else { //Handle other clause types
                let verifier = semantics.get_verifier(&line);
                if verifier.is_none() {
                    return Some(format!("Clause line '{}' is malformed", line))
                }

                let (start_index, verifier) = verifier.unwrap();
                let cleaned_line = &line[start_index..];
                match Self::parse_proof_clause(clauses.len(),&line, &cleaned_line, &mut argument_occurrence_watch, number_of_arguments, false) {
                    Ok(mut clause) => {
                        if clause.get_number_of_members() == 0 {
                            found_empty_clause = true;
                            break; //Empty clause at the end of the proof
                        }

                        clause.set_verifier(verifier);
                        clause_string_to_bases.entry(cleaned_line.to_string()).or_insert((Vec::new(), 0)).0.push(clause.get_index());
                        if clause.get_number_of_members() == 1 {
                            unit_clauses.push(clause.get_index());
                        }
                        clauses.push(clause);
                    },
                    Err(message) => return Some(message)
                }
            }
        }

        if !found_empty_clause {
            return Some(format!("The last line of the proof must be the empty clause"));
        }
        else {
            if let Some(result) = iterator.next() {
                return if result.is_ok() {
                    Some(format!("The last line of the proof must be the empty clause"))
                } else {
                    Some(format!("An error occurred while reading the end of the proof: {}", result.err().unwrap()))
                }
            }
        }

        None
    }

    /// Parses a proof clause
    fn parse_proof_clause(clause_id: usize, complete_line: &str, cleaned_line: &str, argument_occurrence_watch: &mut Vec<usize>, number_of_arguments: usize, is_deletion_clause: bool) -> Result<ClauseBase, String> {
        let mut clause = ClauseBase::new(clause_id);

        let split : Vec<&str> = cleaned_line.split(" ").collect();
        if split.len() < 1 || *split.last().unwrap() != "0" { //We need at least the trailing 0

            return Err(format!("The proof line '{}' is malformed", complete_line))
        }

        //Read the clause members
        for clause_member_str in split.split_last().unwrap().1 {
            //Parse the clause member
            let clause_member_number = clause_member_str.parse();
            if let Err(_) = clause_member_number {
                return Err(format!("The proof line '{}' contains an invalid argument '{}'", complete_line, clause_member_str));
            }
            let clause_member_number: isize = clause_member_number.unwrap();
            let sign = clause_member_number.is_positive();
            let clause_member_number= usize::try_from(clause_member_number.abs()).unwrap();

            if clause_member_number > number_of_arguments {
                return Err(format!("The clause '{}' refers to an invalid argument '{}'", complete_line, clause_member_str));
            }
            let clause_member_index:usize = clause_member_number - 1;
            let arg_occurrence = argument_occurrence_watch.get_mut(clause_member_index).unwrap();
            if *arg_occurrence < clause.get_index() + 1 {
                clause.add_member(clause_member_index, sign);
                *arg_occurrence = clause.get_index() + 1;
            }
        }

        if is_deletion_clause {
            for (clause_member_index, _) in clause.get_members() {
                let arg_occurrence = argument_occurrence_watch.get_mut(*clause_member_index).unwrap();
                *arg_occurrence = clause.get_index();
            }
        }

        Ok(clause)
    }

    pub fn is_required_arguments_consistent(&self) -> bool {
        let mut map : HashMap<usize, bool> = HashMap::new();
        for (argument, sign) in &self.required_arguments {
            let sign = *sign;
            let result = map.insert(*argument, sign);
            if let Some(value) = result {
                if value != sign {
                    return false;
                }
            }
        }
        return true;
    }
}