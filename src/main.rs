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

        #[arg(short, long, default_value_t = false)]
        debug: bool,
    },
}

fn main() {
    let args = Args::parse();

    match args.cmd {
        Commands::Run { path, debug } => {
            interpret(path, debug);
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
enum DataType {
    Bool(bool),
    Int(usize),
    Float(f64),
    String(String),
}

#[derive(Debug, Clone)]
enum Instructions {
    Push(DataType),
    Jump(String),
    IfJmp(String),
    EQ,
    NE,
    And,
    Or,
    Not,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
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

fn interpret(path: PathBuf, debug: bool) {
    let contents = std::fs::read_to_string(path).unwrap();
    let lines = contents.lines();

    let mut program: Vec<Program> = Vec::new();
    let mut current_section: Option<SectionName> = None;
    let mut instructions: Vec<Instructions> = Vec::new();

    for line in lines {
        if line.starts_with(&['/', '#']) || line.is_empty() {
            continue;
        }

        // We have found a section
        if line.starts_with("::") && line.ends_with(':') {
            if !instructions.is_empty() {
                program.push(Program::Section(
                    current_section
                        .take()
                        .unwrap_or_else(|| SectionName("main".to_string())),
                    instructions.drain(..).collect(),
                ));
            }

            current_section = Some(SectionName(line.trim_matches(':').to_string()));
            continue;
        }

        let (instruction, value) = line.split_once(" ").unwrap_or_else(|| (line, ""));

        instructions.push(match instruction.to_lowercase().as_str() {
            "push" => {
                if value == "" {
                    panic!("push requires a value");
                };

                if value.starts_with('"') && value.ends_with('"') {
                    Instructions::Push(DataType::String(value.trim_matches('"').replace("\\n", "\n").replace("\\r", "\r").to_string()))
                } else if value.contains('.') {
                    Instructions::Push(DataType::Float(value.parse::<f64>().unwrap()))
                } else if value == "true" || value == "false" {
                    Instructions::Push(DataType::Bool(value.parse::<bool>().unwrap()))
                } else {
                    Instructions::Push(DataType::Int(value.parse::<usize>().unwrap()))
                }
            }
            "eq" => Instructions::EQ,
            "ne" => Instructions::NE,
            "and" => Instructions::And,
            "or" => Instructions::Or,
            "not" => Instructions::Not,
            "add" => Instructions::Add,
            "sub" => Instructions::Sub,
            "mul" => Instructions::Mul,
            "div" => Instructions::Div,
            "mod" => Instructions::Mod,
            "drop" => Instructions::Drop,
            "dup" => Instructions::Dup,
            "swap" => Instructions::Swap,
            "over" => Instructions::Over,
            "rot" => Instructions::Rot,
            "print" => Instructions::Print,
            "exit" => Instructions::Exit,
            "jump" => {
                if value == "" {
                    panic!("jump requires a label");
                };

                Instructions::Jump(value.to_string())
            }
            "ifjmp" => {
                if value == "" {
                    panic!("ifjmp requires a label");
                };

                Instructions::IfJmp(value.to_string())
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

    let mut stack: Vec<DataType> = Vec::new();

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

        if debug {
            println!("Stack: {:?}", stack);
            println!("Running Instruction: {:?}", instruction);
        }

        match instruction {
            Instructions::Push(value) => {
                stack.push(value);
            }
            Instructions::Add => {
                let (Some(a), Some(b)) = (stack.pop(), stack.pop()) else {
                    panic!("Not enough values on the stack to add");
                };

                match (&a, &b) {
                    (DataType::Int(a), DataType::Int(b)) => {
                        stack.push(DataType::Int(a + b));
                    }
                    (DataType::Float(a), DataType::Float(b)) => {
                        stack.push(DataType::Float(a + b));
                    }
                    _ => {
                        panic!("Cannot add non-numeric values {:?} and {:?}", a, b);
                    }
                }
            }
            Instructions::Sub => {
                let (Some(a), Some(b)) = (stack.pop(), stack.pop()) else {
                    panic!("Not enough values on the stack to subtract");
                };

                match (&a, &b) {
                    (DataType::Int(a), DataType::Int(b)) => {
                        stack.push(DataType::Int(a - b));
                    }
                    (DataType::Float(a), DataType::Float(b)) => {
                        stack.push(DataType::Float(a - b));
                    }
                    _ => {
                        panic!("Cannot subtract non-numeric values {:?} and {:?}", a, b);
                    }
                }
            }
            Instructions::Mul => {
                let (Some(a), Some(b)) = (stack.pop(), stack.pop()) else {
                    panic!("Not enough values on the stack to multiply");
                };

                match (&a, &b) {
                    (DataType::Int(a), DataType::Int(b)) => {
                        stack.push(DataType::Int(a * b));
                    }
                    (DataType::Float(a), DataType::Float(b)) => {
                        stack.push(DataType::Float(a * b));
                    }
                    _ => {
                        panic!("Cannot multiply non-numeric values {:?} and {:?}", a, b);
                    }
                }
            }
            Instructions::Div => {
                let (Some(a), Some(b)) = (stack.pop(), stack.pop()) else {
                    panic!("Not enough values on the stack to divide");
                };

                match (&a, &b) {
                    (DataType::Int(a), DataType::Int(b)) => {
                        if b == &0 {
                            panic!("Cannot divide by zero");
                        }

                        stack.push(DataType::Int(a / b));
                    }
                    (DataType::Float(a), DataType::Float(b)) => {
                        if b == &0.0 {
                            panic!("Cannot divide by zero");
                        }

                        stack.push(DataType::Float(a / b));
                    }
                    _ => {
                        panic!("Cannot divide non-numeric values {:?} and {:?}", a, b);
                    }
                };
            }
            Instructions::Mod => {
                let (Some(a), Some(b)) = (stack.pop(), stack.pop()) else {
                    panic!("Not enough values on the stack to modulo");
                };

                match (&a, &b) {
                    (DataType::Int(a), DataType::Int(b)) => {
                        stack.push(DataType::Int(a % b));
                    }
                    _ => {
                        panic!("Cannot modulo non-numeric values {:?} and {:?}", a, b);
                    }
                }
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
            Instructions::EQ => {
                let (Some(a), Some(b)) = (stack.pop(), stack.pop()) else {
                    panic!("Not enough values on the stack to compare");
                };

                stack.push(DataType::Bool(a == b));
            }
            Instructions::NE => {
                let (Some(a), Some(b)) = (stack.pop(), stack.pop()) else {
                    panic!("Not enough values on the stack to compare");
                };

                stack.push(DataType::Bool(a != b));
            }
            Instructions::And => {
                let (Some(a), Some(b)) = (stack.pop(), stack.pop()) else {
                    panic!("Not enough values on the stack to compare");
                };

                match (&a, &b) {
                    (DataType::Bool(a), DataType::Bool(b)) => {
                        stack.push(DataType::Bool(*a && *b));
                    }
                    _ => {
                        panic!("Cannot compare non-boolean values {:?} and {:?}", a, b);
                    }
                }
            }
            Instructions::Or => {
                let (Some(a), Some(b)) = (stack.pop(), stack.pop()) else {
                    panic!("Not enough values on the stack to compare");
                };

                match (&a, &b) {
                    (DataType::Bool(a), DataType::Bool(b)) => {
                        stack.push(DataType::Bool(*a || *b));
                    }
                    _ => {
                        panic!("Cannot compare non-boolean values {:?} and {:?}", a, b);
                    }
                }
            }
            Instructions::Not => {
                let Some(a) = stack.pop() else {
                    panic!("Not enough values on the stack to compare");
                };

                match a {
                    DataType::Bool(a) => stack.push(DataType::Bool(!a)),
                    _ => {
                        panic!("Cannot compare non-boolean value {:?}", a);
                    }
                }
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
                                program_instructions.splice(ic..ic+1, instructions.to_vec());
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

                let should_jump = match a {
                    DataType::Bool(a) => *a,
                    DataType::Int(a) => *a == 0,
                    _ => {
                        panic!("Cannot compare non-numeric values {:?}", a);
                    }
                };

                if should_jump {
                    let mut found = false;

                    for section in &program {
                        match section {
                            Program::Section(name, instructions) => {
                                if name.0 == label {
                                    program_instructions
                                        .splice(ic..ic+1, instructions.to_vec());
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

                match stack.last().unwrap() {
                    DataType::Bool(a) => print!("{}", a),
                    DataType::Int(a) => print!("{}", a),
                    DataType::Float(a) => print!("{}", a),
                    DataType::String(a) => print!("{}", a),
                }
            }
        }
        ic += 1;
    }
}
