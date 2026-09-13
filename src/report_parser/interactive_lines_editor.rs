use crate::model::Money;
use crate::model::{GenericKeys, ParsedLineInfo};
use color_print::{cprint, cprintln};
use eyre::Result;
use std::io;
use std::str::FromStr;

pub struct InteractiveLinesEditor<Keys: GenericKeys> {
    parsed_lines: Vec<ParsedLineInfo<Keys>>,
}

#[derive(PartialEq)]
enum ProcessCommandResult {
    FinishEditing,
    Edited,
    WrongCommand,
}

fn read_string() -> Result<String> {
    let stdin = io::stdin();
    let mut buffer = String::new();
    stdin.read_line(&mut buffer)?;
    Ok(buffer.trim().to_owned())
}

impl<Keys: GenericKeys> InteractiveLinesEditor<Keys> {
    pub fn new(parsed_lines: Vec<ParsedLineInfo<Keys>>) -> Self {
        Self { parsed_lines }
    }

    fn print_lines(&self) {
        for i in 0..self.parsed_lines.len() {
            cprint!("Line <yellow>{}</yellow>: ", i);
            Self::print_line(&self.parsed_lines[i]);
        }
    }

    fn print_line(line: &ParsedLineInfo<Keys>) {
        cprintln!(
            "Key <green>{:?}</green> Value: <red>{}</red>",
            line.key,
            line.value
        );
        println!("    Raw line: {}", line.original_line);
    }

    pub fn result_lines(&self) -> &Vec<ParsedLineInfo<Keys>> {
        &self.parsed_lines
    }

    pub fn run_editor(&mut self) -> Result<()> {
        cprintln!("<yellow>Interactive editint started</yellow>");
        loop {
            println!(
                "p - print lines. i <number> - add line before specified line. e <number> - edit line. d <number> - delete line. Enter - finish"
            );
            let cmd = read_string()?;
            match self.process_command(&cmd)? {
                ProcessCommandResult::FinishEditing => {
                    return Ok(());
                }
                ProcessCommandResult::Edited => {
                    cprintln!("<green>Edits applied</green>");
                }
                ProcessCommandResult::WrongCommand => {
                    cprintln!("<red>Wrong input</red>");
                }
            }
        }
    }

    fn process_command(&mut self, cmd: &str) -> Result<ProcessCommandResult> {
        if cmd.is_empty() {
            return Ok(ProcessCommandResult::FinishEditing);
        }
        let parts = cmd.split_whitespace().collect::<Vec<&str>>();
        match parts[0] {
            "i" => self.process_insert(&parts),
            "e" => self.process_edit(&parts),
            "d" => Ok(self.process_delete(&parts)),
            "p" => {
                self.print_lines();
                Ok(ProcessCommandResult::Edited)
            }
            _ => Ok(ProcessCommandResult::WrongCommand),
        }
    }

    fn try_get_single_number(cmd_parts: &[&str]) -> Option<usize> {
        if cmd_parts.len() != 2 {
            return None;
        }
        usize::from_str(cmd_parts[1]).ok()
    }

    fn process_insert(&mut self, cmd_parts: &[&str]) -> Result<ProcessCommandResult> {
        let number = match Self::try_get_single_number(cmd_parts) {
            Some(v) => v,
            None => {
                return Ok(ProcessCommandResult::WrongCommand);
            }
        };
        if number > self.parsed_lines.len() {
            return Ok(ProcessCommandResult::WrongCommand);
        }
        let mut new_line = ParsedLineInfo {
            key: Keys::default(),
            value: Money::zero(),
            original_line: String::new(),
        };
        if self.edit_line(&mut new_line)? == ProcessCommandResult::Edited {
            self.parsed_lines.insert(number, new_line);
            return Ok(ProcessCommandResult::Edited);
        }
        Ok(ProcessCommandResult::WrongCommand)
    }

    fn process_edit(&mut self, cmd_parts: &[&str]) -> Result<ProcessCommandResult> {
        let number = match Self::try_get_single_number(cmd_parts) {
            Some(v) => v,
            None => {
                return Ok(ProcessCommandResult::WrongCommand);
            }
        };
        if number >= self.parsed_lines.len() {
            return Ok(ProcessCommandResult::WrongCommand);
        }
        let mut edited_line = self.parsed_lines[number].clone();
        let edit_result = self.edit_line(&mut edited_line)?;
        if edit_result == ProcessCommandResult::Edited {
            self.parsed_lines[number] = edited_line;
        }
        Ok(edit_result)
    }

    fn process_delete(&mut self, cmd_parts: &[&str]) -> ProcessCommandResult {
        let number = match Self::try_get_single_number(cmd_parts) {
            Some(v) => v,
            None => {
                return ProcessCommandResult::WrongCommand;
            }
        };
        if number >= self.parsed_lines.len() {
            return ProcessCommandResult::WrongCommand;
        }
        self.parsed_lines.remove(number);
        ProcessCommandResult::Edited
    }

    fn edit_line(&self, line: &mut ParsedLineInfo<Keys>) -> Result<ProcessCommandResult> {
        loop {
            Self::print_line(line);
            println!("k - change key. v - change value. Enter - Continue");
            let cmd = read_string()?;
            if Self::process_edit_line_command(&cmd, line)? == ProcessCommandResult::FinishEditing {
                return Ok(ProcessCommandResult::Edited);
            }
        }
    }

    fn process_edit_line_command(
        cmd: &str,
        line: &mut ParsedLineInfo<Keys>,
    ) -> Result<ProcessCommandResult> {
        match cmd {
            "" => Ok(ProcessCommandResult::FinishEditing),
            "v" => {
                println!("Enter next value, in roubles:");
                let sum_str = read_string()?;
                let sum = if let Ok(v) = i64::from_str(&sum_str) {
                    v
                } else {
                    cprintln!("<red>Failed parse {}</red>", sum_str);
                    return Ok(ProcessCommandResult::WrongCommand);
                };
                line.value = Money::from_roubles(sum);
                Ok(ProcessCommandResult::Edited)
            }
            "k" => {
                println!("Enter next key (allowed {:?}):", Keys::VARIANTS);
                let key_str = read_string()?;
                let key = if let Ok(v) = Keys::from_str(&key_str) {
                    v
                } else {
                    cprintln!("<red>Failed parse {}</red>", key_str);
                    return Ok(ProcessCommandResult::WrongCommand);
                };
                line.key = key;
                Ok(ProcessCommandResult::Edited)
            }
            _ => Ok(ProcessCommandResult::WrongCommand),
        }
    }
}
