use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use std::path::Path;
use std::io::Write;
use std::fs;
use std::process::Command;

type CoderFunctionReturn = Result<(), String>;
type CoderFunction = fn(Vec<String>) -> CoderFunctionReturn;

pub struct Coder {
    code_path: String,
    library_path: String,
    library_code: HashMap<String, Vec<String>>,
    processor_block_code: HashMap<String, Vec<String>>
}

impl Coder {
    fn new() -> Self {
        Coder {
            code_path: "".to_string(),
            library_path: "".to_string(),
            library_code: HashMap::new(),
            processor_block_code: HashMap::new(),
        }
    }
    pub fn get() -> Arc<Mutex<Coder>> {
        CODER.get_or_init(|| Arc::new(Mutex::new(Coder::new()))).clone()
    }
    pub fn set_code_path(&mut self, _path: String) -> Result<(), String> {
        if _path.is_empty() {
            return Err("Source path cannot be empty.".to_string());
        }
        self.code_path = _path;
        if !Path::new(&self.code_path).exists() {
            fs::create_dir_all(&self.code_path).map_err(|e| format!("Failed to create source path directory: {}", e))?;
        }
        Ok(())
    }
    
    pub fn set_library_path(&mut self, path: String) -> Result<(), String> {
        if path.is_empty() {
            return Err("Library path cannot be empty.".to_string());
        }
        if !Path::new(&path).exists() {
            return Err("Library path does not exist.".to_string());
        }
        self.library_path = path;
        Ok(())
    }
    fn generate_lib_rs_file(&self, crate_name: String, lib_rs_path: String) -> Result<(), String> {
        let file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(lib_rs_path);
        match file {
            Ok(file) => {
                for (part, code_lines) in self.library_code.iter() {
                    let code_string = code_lines.join("\n");
                    match writeln!(&file, "// {} section\n{}", part, code_string) {
                        Ok(_) => {}
                        Err(e) => return Err(format!("Error writing to lib.rs file: {}", e)),
                    }
                }
            }
            Err(e) => {return Err(format!("Error! {}",e))}
        }
        Ok(())
    }
    fn cargo_commands(&self, crate_path: String) -> Result<(), String> {
        Command::new("cargo").arg(format!("new {}", crate_path));
        if std::env::set_current_dir(&crate_path).is_err() {
            return Err(format!("Something went wrong in crate creation"));
        }
        Command::new("cargo").arg(format!("add num-traits"));
        Command::new("cargo").arg(format!("add serde_json"));
        Command::new("cargo").arg(format!("add serde --features derive"));
        Command::new("cargo").arg(format!("add processor_engine --path {}/processor_engine", self.library_path));
        Command::new("cargo").arg(format!("add stream_proc_macro --path {}/processor_engine/src/stream_proc_macro", self.library_path));
        Command::new("cargo").arg(format!("add data_model --path {}/data_model", self.library_path));
        Command::new("cargo").arg(format!("add utils --path {}/utils", self.library_path));
        Ok(())
    }
    fn crate_code_create(&self, command_parts: Vec<String>) -> CoderFunctionReturn {
        let crate_name = command_parts.get(1).ok_or(format!("Error! Missing crate name"))?;
        let crate_path = format!("{}/{}", self.code_path, crate_name);
        if Path::new(&crate_path).exists() {
            return Err(format!("Crate path already exists!"));
        }
        self.cargo_commands(crate_path.clone())?;
        self.generate_lib_rs_file(crate_name.to_string(), format!("{}/src/lib.rs", crate_path))?;
        Ok(())
    }
    fn processor_block_create(&self, command_parts: Vec<String>) -> CoderFunctionReturn {
        let block_name = command_parts.get(1).ok_or(format!("Error! Missing processor_block name"))?;
        if Some(&"in".to_string()) != command_parts.get(2) {
            return Err(format!("Error. Missing 'in' keyword!"));
        }
        let crate_name = command_parts.get(3).ok_or(format!("Error! Missing crate name."))?;
        let block_path = format!("{}/{}/{}.rs",self.code_path, crate_name, block_name);
        if Path::new(&block_path).exists() {
            return Err(format!("Error! Processor block {} source file {} already exists!", block_name, block_path));
        }
        // TODO: Code of processor_block implementation
        Ok(())
    }
    fn code_create(&self, command_parts: Vec<String>) -> CoderFunctionReturn {
        let _type = command_parts.get(1).ok_or(format!("Error! Missing type."))?;
        match _type.as_str() {
            "crate" => { self.crate_code_create(command_parts)?; },
            "task" => { todo!(); },
            "processor_block" => { todo!(); },
            "processor" => { todo!(); },
            "input" => { todo!(); },
            "output" => { todo!(); },
            "state"  => { todo!(); },
            "static" => { todo!(); },
            "parameter" => { todo!(); },
            "code" => { todo!(); },
            _ => {
                return Err(format!("Error! Unkown type {}", _type))
            }
        }
        Ok(())
    }
    pub fn generate(&self, command_parts: Vec<String>) -> CoderFunctionReturn {
        // Placeholder for code generation logic
        println!("Generating code for command parts: {:?}", command_parts);
        match command_parts.get(0) {
            Some(command) => {
                match command.as_str() {
                    "create" => { self.code_create(command_parts)?;},
                    "add" => todo!(),
                    "delete" => todo!(),
                    "connect" => todo!(),
                    "set" => todo!(),
                    "build" => todo!(),
                    _ => {return Err(format!("Command {} unkown", command));},
                }
            }
            None => {return Err("Empty command".to_string())},
        }
        Ok(())
    }
}

static CODER: OnceLock<Arc<Mutex<Coder>>> = OnceLock::new();