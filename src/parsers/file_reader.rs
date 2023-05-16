use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader, ErrorKind, Lines};
use std::path::PathBuf;
use crate::argument_base::ArgumentBase;

///Used to read instance and proof files.
pub struct FileReader {
    reader : Lines<BufReader<File>>
}

impl FileReader
{
    ///Creates a new instance of FileReader.
    ///
    ///Returns Err with a description of the error or the instance.
    pub fn new(path: &PathBuf) -> Result<FileReader, String> {
        match File::open(path) {
            Ok(f) => Ok(FileReader { reader: io::BufReader::new(f).lines()}),
            Err(e) if e.kind() == ErrorKind::NotFound => Err(format!("The file {} does not exist", path.display())),
            Err(e) if e.kind() == ErrorKind::PermissionDenied => Err(format!("Required permissions to open the file {} are missing.", path.display())),
            Err(e) if e.kind() == ErrorKind::InvalidInput => Err(format!("The path {} is invalid.", path.display())),
            Err(e) => Err(format!("An unexpected error occurred while trying to open the file {}: {}.", path.display(), e))
        }
    }
}

///Used to interpret a line of a proof.
pub trait LineInterpreter {
    fn interpret(&self, line: &PathBuf, arguments: &mut Vec<&mut ArgumentBase>);
}

impl Iterator for FileReader
{
    type Item = Result<String, String>;

    ///Advances the iterator, skipping comment lines in the instance description
    fn next(&mut self) -> Option<Result<String, String>> {
        let mut next = self.reader.next();
        loop {
            match next {
                None => return None,
                Some(result) => match result {
                    //Skip comment
                    Ok(line) if line.starts_with("#") => next = self.reader.next(),

                    //Non-comment line
                    Ok(line) => return Some(Ok(line)),

                    Err(err) => return Some(Err(format!("An unexpected error occurred: {}", err)))
                }
            };
        }
    }
}
