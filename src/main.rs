use std::path::PathBuf;

use clap::{Parser, Subcommand};

/// Simple program to greet a person
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the program
    Run {
        /// Path to the program to run
        path: PathBuf,
    },
}

fn main() {
    let args = Args::parse();

    match args.cmd {
        Commands::Run { path } => {
            interpret(path);
        }
    }
}

#[derive(Debug, Clone)]
enum Instructions {
    Push(u8),
    Jump(String),
    IfJmp(String),
    Add,
    Sub,
    Mul,
    Div,
    Dup,
    Swap,
    Over,
    Rot,
    Drop,
    Print,
    Exit,
}

#[derive(Debug, Clone)]
struct SectionName(String);

#[derive(Debug, Clone)]
enum Program {
    Section(SectionName, Vec<Instructions>),
}

fn interpret(path: PathBuf) {
    let contents = std::fs::read_to_string(path).unwrap();
    let lines = contents.lines();

    let mut program: Vec<Program> = Vec::new();
    let mut current_section: Option<SectionName> = None;
    let mut instructions: Vec<Instructions> = Vec::new();

    for line in lines {
        if line.starts_with(&['/', '#']) || line.is_empty() {
            continue;
        }

        let mut parts = line.split(" ");

        let Some(line) = parts.next() else {
            panic!("how did we get here?");
        };

        // We have found a section
        if line.starts_with("::") && line.ends_with(':') {
            if !instructions.is_empty() {
                program.push(Program::Section(
                    current_section.take().unwrap_or_else(|| SectionName("main".to_string())),
                    instructions.drain(..).collect(),
                ));
            }

            current_section = Some(SectionName(line.trim_matches(':').to_string()));
            continue;
        }

        instructions.push(match line {
            "push" => {
                let Some(value) = parts.next() else {
                    panic!("push requires a value");
                };

                Instructions::Push(value.parse::<u8>().unwrap())
            }
            "add" => Instructions::Add,
            "sub" => Instructions::Sub,
            "mul" => Instructions::Mul,
            "div" => Instructions::Div,
            "drop" => Instructions::Drop,
            "dup" => Instructions::Dup,
            "swap" => Instructions::Swap,
            "over" => Instructions::Over,
            "rot" => Instructions::Rot,
            "print" => Instructions::Print,
            "exit" => Instructions::Exit,
            "jump" => {
                let Some(label) = parts.next() else {
                    panic!("jump requires a label");
                };

                Instructions::Jump(label.to_string())
            }
            "ifjmp" => {
                let Some(label) = parts.next() else {
                    panic!("ifjmp requires a label");
                };

                Instructions::IfJmp(label.to_string())
            }
            _ => {
                panic!("Unknown instruction: {line}");
            }
        });
    }

    if !instructions.is_empty() {

        if current_section.is_none() {
            current_section = Some(SectionName("main".to_string()));
        }

        program.push(Program::Section(
            current_section.take().unwrap(),
            instructions.drain(..).collect(),
        ));
    }

    let mut stack: Vec<u8> = Vec::new();

    let mut program_instructions: Vec<Instructions> = Vec::new();
    let mut ic = 0;

    for section in &program {
        match section {
            Program::Section(name, instructions) => {
                if name.0 == "main" {
                    program_instructions = instructions.to_vec();
                    break;
                }
            }
        }
    }

    if program_instructions.is_empty() {
        panic!("No main section found");
    }

    while ic < program_instructions.len() {
        let instruction = program_instructions[ic].clone();
        // println!("Stack: {:?}", stack);
        // println!("Running Instruction: {:?}", instruction);

        match instruction {
            Instructions::Push(value) => {
                stack.push(value);
            }
            Instructions::Add => {
                let (Some(a), Some(b)) = (stack.pop(), stack.pop()) else {
                    panic!("Not enough values on the stack to add");
                };

                stack.push(a + b);
            }
            Instructions::Sub => {
                let (Some(a), Some(b)) = (stack.pop(), stack.pop()) else {
                    panic!("Not enough values on the stack to subtract");
                };

                stack.push(a - b);
            }
            Instructions::Mul => {
                let (Some(a), Some(b)) = (stack.pop(), stack.pop()) else {
                    panic!("Not enough values on the stack to multiply");
                };

                stack.push(a * b);
            }
            Instructions::Div => {
                let (Some(a), Some(b)) = (stack.pop(), stack.pop()) else {
                    panic!("Not enough values on the stack to divide");
                };

                stack.push(a / b);
            }
            Instructions::Dup => {
                let Some(a) = stack.last().cloned() else {
                    panic!("Nothing to duplicate");
                };

                stack.push(a);
            }
            Instructions::Swap => {
                let (Some(a), Some(b)) = (stack.pop(), stack.pop()) else {
                    panic!("Not enough values on the stack to swap");
                };

                stack.push(a);
                stack.push(b);
            }
            Instructions::Over => {
                let Some(b) = stack.get(stack.len() - 2).cloned() else {
                    panic!("Not enough values on the stack to duplicate");
                };

                stack.push(b);
            }
            Instructions::Rot => {
                let (Some(a), Some(b), Some(c)) = (stack.pop(), stack.pop(), stack.pop()) else {
                    panic!("Not enough values on the stack to rotate");
                };

                stack.push(b);
                stack.push(a);
                stack.push(c);
            }
            Instructions::Drop => {
                stack.pop();
            }
            Instructions::Exit => {
                break;
            }
            Instructions::Jump(label) => {
                let mut found = false;

                for section in &program {
                    match section {
                        Program::Section(name, instructions) => {
                            if name.0 == label {
                                program_instructions = instructions.to_vec();
                                ic = 0;
                                found = true;
                                break;
                            }
                        }
                    }
                }

                if !found {
                    panic!("Unknown label: {label}");
                }

                continue;
            }
            Instructions::IfJmp(label) => {
                let Some(a) = stack.last() else {
                    panic!("Not enough values on the stack to compare");
                };

                if *a == 0 {
                    let mut found = false;

                    for section in &program {
                        match section {
                            Program::Section(name, instructions) => {
                                if name.0 == label {
                                    program_instructions = instructions.to_vec();
                                    ic = 0;
                                    found = true;
                                    break;
                                }
                            }
                        }
                    }

                    if !found {
                        panic!("Unknown label: {label}");
                    }

                    continue;
                }
            }
            Instructions::Print => {
                if stack.is_empty() {
                    panic!("Nothing to print");
                }

                println!("{}", stack.last().unwrap());
            }
        }
        ic += 1;
    }
}
