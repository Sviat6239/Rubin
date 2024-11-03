use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::{Command, exit};

struct Shell {
    current_dir: PathBuf,
    history: Vec<PathBuf>,
    history_index: usize,
    custom_commands: Vec<CustomCommand>, // Store custom commands in a vector
    env_vars: HashMap<String, String>,   // Store custom environment variables
}

#[derive(Debug)]
struct CustomCommand {
    name: String,
    definition: String,
    description: String,
}

impl Shell {
    fn new() -> Self {
        let current_dir = env::current_dir().unwrap();
        Shell {
            current_dir: current_dir.clone(),
            history: vec![current_dir],
            history_index: 0,
            custom_commands: Vec::new(),
            env_vars: HashMap::new(),
        }
    }

    fn run(&mut self) {
        loop {
            print!("{} $> ", self.current_dir.display());
            io::stdout().flush().unwrap();

            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            let command = input.trim();
            self.execute_command(command);
        }
    }

    fn execute_command(&mut self, command: &str) {
        let args: Vec<&str> = command.split_whitespace().collect();

        if let Some(first_arg) = args.get(0).map(|&s| s) {
            match first_arg {
                "dir" => self.list_dir(),
                "mkdir" => self.make_dir(args.get(1).copied()),
                "rmdir" => self.remove_dir(args.get(1).copied()),
                "help" => self.display_help(),
                "<-" => self.go_backward(),
                "->" => self.go_forward(),
                "clear" => self.clear_screen(),
                "rename" => self.rename_dir(args.get(1).copied(), args.get(2).copied()),
                "move" => self.move_file(args.get(1).copied(), args.get(2).copied()),
                "copy" => self.copy_file(args.get(1).copied(), args.get(2).copied()),
                "type" => self.type_file(args.get(1).copied()),
                "exit" => self.exit_shell(),
                "cc" => self.handle_custom_command(&args[1..]),
                "run" => self.run_script(args.get(1).copied()),      // New: run a script
                "source" => self.source_env_file(args.get(1).copied()), // New: source environment variables
                "setenv" => self.set_env_var(args.get(1).copied(), args.get(2).copied()), // Fix: use copied()
                _ => self.handle_file_commands(first_arg, &args[1..]),
            }
        }
    }

    fn run_script(&self, script_path: Option<&str>) {
        if let Some(path) = script_path {
            let script_full_path = self.current_dir.join(path);
            if script_full_path.exists() {
                let status = Command::new("sh")
                    .arg(script_full_path)
                    .status();
                if let Err(e) = status {
                    println!("Failed to run script: {}", e);
                }
            } else {
                println!("Script not found: {}", path);
            }
        } else {
            println!("Usage: run <script_path>");
        }
    }

    fn source_env_file(&mut self, file_path: Option<&str>) {
        if let Some(path) = file_path {
            let full_path = self.current_dir.join(path);
            match fs::read_to_string(full_path) {
                Ok(contents) => {
                    for line in contents.lines() {
                        if let Some((key, value)) = line.split_once('=') {
                            self.env_vars.insert(key.trim().to_string(), value.trim().to_string());
                        }
                    }
                    println!("Environment variables sourced.");
                }
                Err(_) => println!("Failed to read env file."),
            }
        } else {
            println!("Usage: source <env_file_path>");
        }
    }

    fn set_env_var(&mut self, key: Option<&str>, value: Option<&str>) {
        if let (Some(k), Some(v)) = (key, value) {
            self.env_vars.insert(k.to_string(), v.to_string());
            println!("Environment variable set: {}={}", k, v);
        } else {
            println!("Usage: setenv <key> <value>");
        }
    }

    fn handle_custom_command(&mut self, args: &[&str]) {
        if let Some(action) = args.get(0) {
            match *action {
                "create" => self.create_custom_command(
                    args.get(1).map(|v| *v),
                    args.get(2).map(|v| *v),
                    args.get(3).map(|v| *v)
                ),
                "list" => self.list_custom_commands(),
                "delete" => self.delete_custom_command(args.get(1).map(|v| *v)),
                "refactor" => self.refactor_custom_command(
                    args.get(1).map(|v| *v),
                    args.get(2).map(|v| *v),
                    args.get(3).map(|v| *v)
                ),
                _ => println!("Unknown custom command action: {}", action),
            }
        } else {
            println!("Usage: cc <create/list/delete/refactor>");
        }
    }

    fn create_custom_command(&mut self, cmd_name: Option<&str>, cmd_definition: Option<&str>, cmd_description: Option<&str>) {
        if let (Some(name), Some(definition), Some(description)) = (cmd_name, cmd_definition, cmd_description) {
            let command = CustomCommand {
                name: name.to_string(),
                definition: definition.to_string(),
                description: description.to_string(),
            };
            self.custom_commands.push(command);
            println!("Custom command '{}' created.", name);
        } else {
            println!("Usage: cc create <command_name> <command_definition> <command_description>");
        }
    }

    fn list_custom_commands(&self) {
        if self.custom_commands.is_empty() {
            println!("No custom commands defined.");
        } else {
            for (index, command) in self.custom_commands.iter().enumerate() {
                println!("{}: {} - {} (Definition: {})", index + 1, command.name, command.description, command.definition);
            }
        }
    }

    fn delete_custom_command(&mut self, cmd_number: Option<&str>) {
        if let Some(num_str) = cmd_number {
            if let Ok(index) = num_str.parse::<usize>() {
                if index > 0 && index <= self.custom_commands.len() {
                    let removed = self.custom_commands.remove(index - 1);
                    println!("Custom command '{}' deleted.", removed.name);
                } else {
                    println!("Command number out of range.");
                }
            } else {
                println!("Invalid command number.");
            }
        } else {
            println!("Usage: cc delete <command_number>");
        }
    }

    fn refactor_custom_command(&mut self, cmd_number: Option<&str>, new_definition: Option<&str>, new_description: Option<&str>) {
        if let Some(num_str) = cmd_number {
            if let Ok(index) = num_str.parse::<usize>() {
                if index > 0 && index <= self.custom_commands.len() {
                    let command = &mut self.custom_commands[index - 1];
                    if let Some(definition) = new_definition {
                        command.definition = definition.to_string();
                    }
                    if let Some(description) = new_description {
                        command.description = description.to_string();
                    }
                    println!("Custom command '{}' updated.", command.name);
                } else {
                    println!("Command number out of range.");
                }
            } else {
                println!("Invalid command number.");
            }
        } else {
            println!("Usage: cc refactor <command_number> <new_definition> <new_description>");
        }
    }

    fn list_dir(&self) {
        if let Ok(entries) = fs::read_dir(&self.current_dir) {
            for entry in entries.filter_map(Result::ok) {
                println!("{}", entry.file_name().to_string_lossy());
            }
        }
    }

    fn make_dir(&self, dir_name: Option<&str>) {
        if let Some(name) = dir_name {
            let path = self.current_dir.join(name);
            if fs::create_dir_all(path).is_err() {
                println!("Failed to create directory: {}", name);
            }
        } else {
            println!("Usage: mkdir <directory_name>");
        }
    }

    fn remove_dir(&self, dir_name: Option<&str>) {
        if let Some(name) = dir_name {
            let path = self.current_dir.join(name);
            if fs::remove_dir(path).is_err() {
                println!("Failed to remove directory: {}", name);
            }
        } else {
            println!("Usage: rmdir <directory_name>");
        }
    }

    fn go_backward(&mut self) {
        if self.history_index > 0 {
            self.history_index -= 1;
            self.current_dir = self.history[self.history_index].clone();
        }
    }

    fn go_forward(&mut self) {
        if self.history_index < self.history.len() - 1 {
            self.history_index += 1;
            self.current_dir = self.history[self.history_index].clone();
        }
    }

    fn clear_screen(&self) {
        Command::new("cmd").arg("/C").arg("cls").status().unwrap();
    }

    fn rename_dir(&self, old_name: Option<&str>, new_name: Option<&str>) {
        if let (Some(old), Some(new)) = (old_name, new_name) {
            let old_path = self.current_dir.join(old);
            let new_path = self.current_dir.join(new);
            if fs::rename(old_path, new_path).is_err() {
                println!("Failed to rename directory.");
            }
        } else {
            println!("Usage: rename <old_name> <new_name>");
        }
    }

    fn move_file(&self, source: Option<&str>, destination: Option<&str>) {
        if let (Some(src), Some(dest)) = (source, destination) {
            let src_path = self.current_dir.join(src);
            let dest_path = self.current_dir.join(dest);
            if fs::rename(src_path, dest_path).is_err() {
                println!("Failed to move file.");
            }
        } else {
            println!("Usage: move <source> <destination>");
        }
    }

    fn copy_file(&self, source: Option<&str>, destination: Option<&str>) {
        if let (Some(src), Some(dest)) = (source, destination) {
            let src_path = self.current_dir.join(src);
            let dest_path = self.current_dir.join(dest);
            if fs::copy(src_path, dest_path).is_err() {
                println!("Failed to copy file.");
            }
        } else {
            println!("Usage: copy <source> <destination>");
        }
    }

    fn type_file(&self, file_name: Option<&str>) {
        if let Some(name) = file_name {
            let file_path = self.current_dir.join(name);
            match fs::read_to_string(file_path) {
                Ok(contents) => println!("{}", contents),
                Err(_) => println!("Failed to read file."),
            }
        } else {
            println!("Usage: type <file_name>");
        }
    }

    fn exit_shell(&self) {
        exit(0);
    }

    fn handle_file_commands(&self, file_name: &str, args: &[&str]) {
        println!("Unknown command: {}", file_name);
    }
}

fn main() {
    let mut shell = Shell::new();
    shell.run();
}
