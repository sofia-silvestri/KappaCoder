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
        Command::new("cargo").arg(format!("add num-traits")).status().expect("Failed to create the project");
        Command::new("cargo").arg(format!("add serde_json")).status().expect("Failed to create the project");
        Command::new("cargo").arg(format!("add serde --features derive")).status().expect("Failed to create the project");
        Command::new("cargo").arg(format!("add processor_engine --path {}/processor_engine", self.library_path)).status().expect("Failed to create the project");
        Command::new("cargo").arg(format!("add stream_proc_macro --path {}/processor_engine/src/stream_proc_macro", self.library_path)).status().expect("Failed to create the project");
        Command::new("cargo").arg(format!("add data_model --path {}/data_model", self.library_path)).status().expect("Failed to create the project");
        Command::new("cargo").arg(format!("add utils --path {}/utils", self.library_path)).status().expect("Failed to create the project");
        let _ = std::env::set_current_dir(curr_dir);
        Ok(())
    }
    pub fn create_crate(&mut self, crate_name: &String, metadata: &String) -> CoderFunctionReturn {
        let crate_path = format!("{}/{}", self.code_path, to_snake_case(crate_name));
        if Path::new(&crate_path).exists() {
            return Err(format!("Crate directory {} already exists", crate_path));
        }
        Command::new("cargo").arg(format!("new --lib {}", crate_path)).status().expect("Failed to create the project");
        self.cargo_commands(crate_path)?;
        let mut lib_coder = match &self.lib_coder {
            Some(coder) => coder.clone(),
            None => {
                return Err(format!("Library coder not initialized"));
            },
        };
        lib_coder.generate()?;
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
        lib_coder.generate()?;
        let mut processor_coder = ProcessorCoder::new(processor_path.clone(), processor_name.to_string());
        processor_coder.generate()?;
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
        processor_coder.generate()?;
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
        Command::new("cargo").arg(format!("new {}", application_path)).status().expect("Failed to create the project");
        self.cargo_commands(application_path)?;
        let mut main_coder = match &self.main_coder {
            Some(coder) => coder.clone(),
            None => {
                return Err(format!("Main coder not initialized"));
            },
        };
        main_coder.generate()?;
        Ok(())
    }
    pub fn create_task(&mut self, task_name: &String) -> CoderFunctionReturn {
        let mut main_coder = match &self.main_coder {
            Some(coder) => coder.clone(),
            None => {
                return Err(format!("Main coder not initialized"));
            },
        };
        main_coder.add_task_processor(task_name.clone());
        main_coder.generate()?;
        Ok(())
    }
    pub fn create_processor(&mut self, processor_name: &String, processor_code: &String, processor_type: &String) -> CoderFunctionReturn {
        let mut main_coder = match &self.main_coder {
            Some(coder) => coder.clone(),
            None => {
                return Err(format!("Main coder not initialized"));
            },
        };
        main_coder.add_stream_processor(processor_name.clone(), processor_type.clone());
        main_coder.generate()?;
        Ok(())
    }
    pub fn delete_object(&mut self, object_name: &String) -> CoderFunctionReturn {
        let split_name = object_name.split(".").collect::<Vec<&str>>();
        if split_name.len() == 1 {
            return Err(format!("Invalid object name format"));
        }
        if let Some(processor_coder) = self.module_coders.get_mut(&object_name.clone()) {
            processor_coder.delete_object(object_name);
            processor_coder.generate()
        } else if let Some(main_coder) = &mut self.main_coder {
            main_coder.delete_object(object_name);
            main_coder.generate()
        } else if let Some(lib_coder) = &mut self.lib_coder {
            lib_coder.delete_object(object_name);
            lib_coder.generate()
        } else {
            Err(format!("Object {} not found", object_name))
        }
    }
    pub fn connect(&mut self, source_name: &String, target_name: &String) -> CoderFunctionReturn {
        let mut main_coder = match &self.main_coder {
            Some(coder) => coder.clone(),
            None => {
                return Err(format!("Main coder not initialized"));
            },
        };
        let split_source: Vec<&str> = source_name.split(".").collect();
        let split_target: Vec<&str> = target_name.split(".").collect();
        if split_source.len() != 4 || split_target.len() != 4 {
            return Err(format!("Invalid source or target name format"));
        }
        let from_processor = format!("{}.{}.{}", split_source[0], split_source[1], split_source[2]);
        let to_processor = format!("{}.{}.{}", split_target[0], split_target[1], split_target[2]);
        main_coder.add_connection(from_processor, split_source[3].to_string(), to_processor, split_target[3].to_string());
        main_coder.generate()?;
        Ok(())
    }
    pub fn set_value(&mut self, object_type: &String, object_name: &String, value: &String) -> CoderFunctionReturn {
        let mut main_coder = match &self.main_coder {
            Some(coder) => coder.clone(),
            None => {
                return Err(format!("Main coder not initialized"));
            },
        };
        let split_name: Vec<&str> = object_name.split(".").collect();
        if split_name.len() != 4 {
            return Err(format!("Invalid source name format"));
        }
        let processor_name = format!("{}.{}.{}", split_name[0], split_name[1], split_name[2]);
        main_coder.add_setting_value(processor_name, object_type.clone(), split_name[3].to_string(), value.clone());
        main_coder.generate()?;
        Ok(())
    }
    pub fn create_processor_code(&mut self, processor_name: &String, code_id: ModCoderParts, code: &String) -> CoderFunctionReturn {
        let mut processor_coder = match self.module_coders.get_mut(&processor_name.clone()) {
            Some(coder) => coder,
            None => {
                return Err(format!("Processor coder for {} not found", processor_name.clone()));
            },
        };
        processor_coder.add_code_section(code_id, code.clone());
        processor_coder.generate()?;
        Ok(())
    }
    pub fn create_application_code(&mut self, code_id: MainCoderParts, code: &String) -> CoderFunctionReturn {
        let mut main_coder = match &self.main_coder {
            Some(coder) => coder.clone(),
            None => {
                return Err(format!("Main coder not initialized"));
            },
        };
        main_coder.add_code_section(code_id, code.clone());
        main_coder.generate()?;
        Ok(())
    }
    pub fn build(&mut self, build_object_name: &String, build_type: &String) -> CoderFunctionReturn {
        let build_path = format!("{}/{}", self.code_path, to_snake_case(build_object_name));
        if !Path::new(&build_path).exists() {
            return Err(format!("Source directory {} does not exists", build_path));
        }
        let curr_dir = std::env::current_dir().unwrap();
        if std::env::set_current_dir(&build_path).is_err() {
            return Err(format!("Something went wrong in building"));
        }
        if build_type == "release" {
            Command::new("cargo").arg("build").arg("--release").status().expect("Failed to build the project");
        } else {
            Command::new("cargo").arg("build").status().expect("Failed to build the project");
        }
        let _ = std::env::set_current_dir(curr_dir);
        Ok(())
    }
}
    
