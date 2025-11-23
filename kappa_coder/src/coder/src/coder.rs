use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use std::path::Path;
use std::io::Write;
use std::fs;
use std::process::Command;
use crate::lib_coder::LibCoder;
use crate::processor_coder::ProcessorCoder;
use crate::main_coder::MainCoder;
type CoderFunctionReturn = Result<(), String>;
type CoderFunction = fn(Vec<String>) -> CoderFunctionReturn;

fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c.is_ascii_uppercase() {
            if !result.is_empty() {
                let next_char_is_lowercase = chars.peek().map_or(false, |&next| next.is_ascii_lowercase());
                if next_char_is_lowercase {
                    result.push('_');
                } else if result.chars().last().map_or(false, |last| !last.is_ascii_uppercase() && last != '_') {
                     result.push('_');
                }
            }
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c);
        }
    }
    result
}

pub struct LibMaps {
    pub path: String,
    pub coder: LibCoder,
}

pub struct ModuleMaps {
    pub path: String,
    pub coder: ProcessorCoder,
}

pub struct Coder {
    code_path: String,
    library_path: String,
    module_library: HashMap<String, LibMaps>,
    processor_modules: HashMap<String, ModuleMaps>,
}


impl Coder {
    fn new() -> Self {
        Coder {
            code_path: "".to_string(),
            library_path: "".to_string(),
            module_library: HashMap::new(),
            processor_modules: HashMap::new(),
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
                if writeln!(&file, "// Auto-generated lib.rs for crate {}", crate_name).is_err() {
                    return Err("Error writing to lib.rs file".to_string());
                }
                let lib = self.module_library.get(&crate_name);
                match lib {
                    Some(lib) => {
                        let code = lib.coder.generate_lib_code();
                        if writeln!(&file, "{}", code).is_err() {
                            return Err("Error writing lib code to lib.rs file".to_string());
                        }
                    }
                    None => {
                        return Err("LibCoder not found for crate".to_string());
                    }
                }
            }
            Err(e) => {return Err(format!("Error! {}",e))}
        }
        Ok(())
    }
    fn generate_processor_rs_file(&self, crate_name: String, processor_name: String, processor_path: String) -> Result<(), String> {
        let file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(processor_path);
        match file {
            Ok(file) => {
                if writeln!(&file, "// Auto-generated processor block {} for crate {}", processor_name, crate_name).is_err() {
                    return Err("Error writing to processor block file".to_string());
                }
                // TODO: Generate processor block code
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
    fn crate_code_create(&mut self, command_parts: Vec<String>) -> CoderFunctionReturn {
        let crate_name = command_parts.get(1).ok_or(format!("Error! Missing crate name"))?;
        let crate_path = format!("{}/{}", self.code_path, crate_name);
        if Path::new(&crate_path).exists() {
            return Err(format!("Crate path already exists!"));
        }
        
        self.module_library.insert(
            crate_name.clone(), 
            LibMaps {
                path: crate_path.clone(),
                coder: LibCoder::new(),
            }
        );
        self.cargo_commands(crate_path.clone())?;
        self.generate_lib_rs_file(crate_name.to_string(), format!("{}/src/lib.rs", crate_path))?;
        Ok(())
    }
    fn processor_block_create(&mut self, command_parts: Vec<String>) -> CoderFunctionReturn {
        let block_name = command_parts.get(1).ok_or(format!("Error! Missing processor_block name"))?;
        if Some(&"in".to_string()) != command_parts.get(2) {
            return Err(format!("Error. Missing 'in' keyword!"));
        }
        let crate_name = command_parts.get(3).ok_or(format!("Error! Missing crate name."))?;
        match self.module_library.get(crate_name) {
            Some(lib) => {lib.coder.clone().add_module(crate_name.to_string());},
            None => {
                return Err(format!("Error! Crate {} not found. Create crate first.", crate_name));
            }
        }
        self.generate_lib_rs_file(crate_name.to_string(), self.module_library.get(crate_name).ok_or(format!("Error! Crate path not found."))?.path.clone())?;
        let block_path = format!("{}/{}/{}.rs",self.code_path, crate_name, to_snake_case(block_name));
        if Path::new(&block_path.clone()).exists() {
            return Err(format!("Error! Processor block {} source file {} already exists!", block_name, block_path.clone()));
        }
        self.generate_processor_rs_file(crate_name.to_string(), block_name.to_string(), block_path.clone())?;
        self.processor_modules.insert(
            block_name.to_string(),
            ModuleMaps {
                path: block_path,
                coder: ProcessorCoder::new(),
            }
        );
        Ok(())
    }
    fn code_create(&mut self, command_parts: Vec<String>) -> CoderFunctionReturn {
        let _type = command_parts.get(1).ok_or(format!("Error! Missing type."))?;
        match _type.as_str() {
            "crate" => { self.crate_code_create(command_parts)?; },
            "task" => { todo!(); },
            "processor_block" => { self.processor_block_create(command_parts)?; },
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
    pub fn generate(&mut self, command_parts: Vec<String>) -> CoderFunctionReturn {
        // Placeholder for code generation logic
        println!("Generating code for command parts: {:?}", command_parts);
        match command_parts.get(0) {
            Some(command) => {
                match command.as_str() {
                    "create" => { self.code_create(command_parts)?;},
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