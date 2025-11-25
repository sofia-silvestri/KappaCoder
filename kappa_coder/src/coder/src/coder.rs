use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use std::path::Path;
use std::io::Write;
use std::process::Command;
use crate::lib_coder::LibCoder;
use crate::processor_coder::{ProcessorCoder, ModCoderParts};
use crate::main_coder::{MainCoder, MainCoderParts};
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
#[derive(Clone)]
pub struct Coder {
    code_path: String,
    library_path: String,
    lib_coder: Option<LibCoder>,
    main_coder: Option<MainCoder>,
    module_coders: HashMap<String, ProcessorCoder>,
}

pub enum BuildType {
    Library,
    Application,
}

impl Coder {
    pub fn new(code_path: String, library_path: String, build_type: BuildType) -> Self {
        let mut lib_coder: Option<LibCoder> = None;
        let mut main_coder: Option<MainCoder> = None;
        match build_type {
            BuildType::Library => {
                let lib_path = code_path.clone();
                lib_coder = Some(LibCoder::new(code_path.clone()));
            },
            BuildType::Application => {
                let main_path = format!("{}/src/main.rs", code_path.clone());
                main_coder = Some(MainCoder::new(main_path));
            },
        }
        Coder {
            code_path: code_path,
            library_path: library_path,
            lib_coder,
            main_coder,
            module_coders: HashMap::new(),
        }
    }
    
    fn cargo_commands(&self, path: String) -> Result<(), String> {
        let curr_dir = std::env::current_dir().unwrap();
        if std::env::set_current_dir(&path).is_err() {
            return Err(format!("Something went wrong in crate creation"));
        }
        Command::new("cargo").arg(format!("add num-traits"));
        Command::new("cargo").arg(format!("add serde_json"));
        Command::new("cargo").arg(format!("add serde --features derive"));
        Command::new("cargo").arg(format!("add processor_engine --path {}/processor_engine", self.library_path));
        Command::new("cargo").arg(format!("add stream_proc_macro --path {}/processor_engine/src/stream_proc_macro", self.library_path));
        Command::new("cargo").arg(format!("add data_model --path {}/data_model", self.library_path));
        Command::new("cargo").arg(format!("add utils --path {}/utils", self.library_path));
        let _ = std::env::set_current_dir(curr_dir);
        Ok(())
    }
    pub fn create_crate(&mut self, crate_name: &String, metadata: &String) -> CoderFunctionReturn {
        let crate_path = format!("{}/{}", self.code_path, to_snake_case(crate_name));
        if Path::new(&crate_path).exists() {
            return Err(format!("Crate directory {} already exists", crate_path));
        }
        Command::new("cargo").arg(format!("new --lib {}", crate_path));
        self.cargo_commands(crate_path)?;
        let mut lib_coder = match &self.lib_coder {
            Some(coder) => coder.clone(),
            None => {
                return Err(format!("Library coder not initialized"));
            },
        };
        lib_coder.generate();
        Ok(())
    }
    pub fn create_processor_block(&mut self, block_name: String) -> CoderFunctionReturn {
        let split_name = block_name.split(".").collect::<Vec<&str>>();
        let crate_name = split_name[0];
        let processor_name = split_name[1];
        let processor_path = format!("{}/{}/src/{}.rs", self.code_path, to_snake_case(&crate_name), to_snake_case(&processor_name));
        if Path::new(&processor_path).exists() {
            return Err(format!("Processor file {} already exists", processor_path));
        }
        let mut lib_coder = match &self.lib_coder {
            Some(coder) => coder.clone(),
            None => {
                return Err(format!("Library coder not initialized"));
            },
        };
        lib_coder.add_module(processor_name.to_string());
        lib_coder.generate();
        let mut processor_coder = ProcessorCoder::new(processor_path.clone());
        processor_coder.generate();
        self.module_coders.insert(block_name.clone(), processor_coder.clone());
        Ok(())
    }
    pub fn create_typed(&mut self, object_kind: &String, object_name: &String, object_type: &String) -> CoderFunctionReturn {
        let split_name = object_name.split(".").collect::<Vec<&str>>();
        let crate_name = split_name[0];
        let processor_name = split_name[1];
        let mut processor_coder = match self.module_coders.get_mut(&object_name.clone()) {
            Some(coder) => coder,
            None => {
                return Err(format!("Processor coder for {} not found", object_name));
            },
        };
        processor_coder.add_typed(object_kind, object_name, object_type);
        processor_coder.generate();
        Ok(())
    }
    pub fn create_settable(&mut self, object_kind: &String, object_name: &String, object_type: &String, object_default: &String, object_limits: Option<&String>) -> CoderFunctionReturn {
        let split_name = object_name.split(".").collect::<Vec<&str>>();
        let crate_name = split_name[0];
        let processor_name = split_name[1];
        let mut processor_coder = match self.module_coders.get_mut(&object_name.clone()) {
            Some(coder) => coder,
            None => {
                return Err(format!("Processor coder for {} not found", object_name));
            },
        };
        processor_coder.add_settable(object_kind, object_name, object_type, object_default, object_limits);
        processor_coder.generate()?;
        Ok(())
    }
    pub fn create_application(&mut self, application_name: &String) -> CoderFunctionReturn {
        let application_path = format!("{}/{}", self.code_path, to_snake_case(application_name));
        if Path::new(&application_path).exists() {
            return Err(format!("Applicatio path {} already exists", application_path));
        }
        Command::new("cargo").arg(format!("new {}", application_path));
        self.cargo_commands(application_path)?;
        let mut main_coder = match &self.main_coder {
            Some(coder) => coder.clone(),
            None => {
                return Err(format!("Main coder not initialized"));
            },
        };
        main_coder.generate();
        Ok(())
    }
    pub fn create_task(&mut self, task_name: &String) -> CoderFunctionReturn {
        todo!();
    }
    pub fn create_processor(&mut self, processor_name: &String, processor_code: &String, processor_type: &String) -> CoderFunctionReturn {
        todo!();
    }
    pub fn delete_object(&mut self, object_name: &String) -> CoderFunctionReturn {
        todo!();
    }
    pub fn connect(&mut self, source_name: &String, target_name: &String) -> CoderFunctionReturn {
        todo!();
    }
    pub fn set_value(&mut self, object_type: &String, object_name: &String, value: &String) -> CoderFunctionReturn {
        todo!();
    }
    pub fn create_processor_code(&mut self, processor_name: &String, code_id: ModCoderParts, code: &String) -> CoderFunctionReturn {
        todo!();
    }
    pub fn create_application_code(&mut self, processor_name: &String, code_id: MainCoderParts, code: &String) -> CoderFunctionReturn {
        todo!();
    }
    pub fn build(&mut self, build_object_name: &String, build_type: &String) -> CoderFunctionReturn {
        todo!();
    }
}
    
