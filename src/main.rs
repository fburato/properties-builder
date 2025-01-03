#[macro_use]
extern crate maplit;
mod model;
mod overriding;
mod properties_parser;
mod test_utils;

use crate::model::InternalError;
use crate::overriding::{
    CustomCaseSensitiveStyleOverrider, Environment, Overrider, SpringStyleOverrider,
};
use crate::properties_parser::{parse_line, Line};
use clap::Parser;
use model::Args;
use std::collections::HashSet;
use std::fs::File;
use std::io::{stdin, stdout, BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::{fs, path};

fn main_exec() -> Result<(), InternalError> {
    let configuration = Args::parse().validate_and_convert()?;
    let empty_buffer: &[u8] = &[];
    let input: Box<dyn BufRead> = if configuration.empty_input {
        Box::new(BufReader::new(empty_buffer))
    } else if configuration.file.is_none() {
        Box::new(BufReader::new(stdin()))
    } else {
        let f = File::open(configuration.file.clone().unwrap())?;
        Box::new(BufReader::new(f))
    };
    let same_input_output_file: bool =
        if configuration.file.is_some() && configuration.output_file.is_some() {
            let input_file = path::absolute(configuration.file.clone().unwrap())?;
            let output_file = path::absolute(configuration.output_file.clone().unwrap())?;
            input_file == output_file
        } else {
            false
        };
    let (mut output, path): (Box<dyn Write>, Option<PathBuf>) =
        if configuration.output_file.is_none() {
            (Box::new(BufWriter::new(stdout())), None)
        } else {
            let path = if same_input_output_file {
                let named = tempfile::NamedTempFile::new()?;
                named.into_temp_path().to_path_buf()
            } else {
                Path::new(configuration.output_file.clone().unwrap().as_str()).to_path_buf()
            };
            let f = File::options().create(true).write(true).open(&path)?;
            (Box::new(BufWriter::new(f)), Some(path))
        };
    let env: Environment = Environment::new(&std::env::vars().collect());
    let overrider: Box<dyn Overrider> = if configuration.spring {
        Box::new(SpringStyleOverrider::new(env))
    } else {
        Box::new(CustomCaseSensitiveStyleOverrider::new(
            configuration.replacement_map,
            env,
        ))
    };

    let mut defined_properties: HashSet<String> = HashSet::new();

    for (line_num, line_result) in input.lines().enumerate() {
        let line = line_result?;
        let parse_result = parse_line(line.as_str(), (line_num + 1) as i32)?;
        match parse_result {
            Line::Ignorable(line) => writeln!(output, "{}", line)?,
            Line::Prop(property) => {
                let overridden = overrider.resolve_substitution(
                    property.key.as_str(),
                    Some(configuration.prefix.as_str()),
                );
                if let Some(overridden_value) = overridden {
                    writeln!(output, "{}={}", property.key, overridden_value)?;
                } else {
                    writeln!(output, "{}={}", property.key, property.value)?;
                }
                defined_properties.replace(property.key);
            }
        }
    }
    for property in overrider.generate_additions(configuration.prefix.as_str()) {
        if !defined_properties.contains(property.key.as_str()) {
            writeln!(output, "{}={}", property.key, property.value)?;
        }
    }
    output.flush()?;
    if same_input_output_file {
        fs::copy(path.unwrap(), configuration.output_file.unwrap())?;
    }
    Ok(())
}

fn main() -> ExitCode {
    match main_exec() {
        Ok(_) => ExitCode::SUCCESS,
        Err(err) => {
            println!("{}", err);
            ExitCode::FAILURE
        }
    }
}
