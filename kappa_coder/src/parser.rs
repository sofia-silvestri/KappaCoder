use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::path::Path;
use serde::{Serialize, Deserialize};
use coder::lib_coder::LibCoder;
use coder::main_coder::{MainCoderParts, MainCoder};
use coder::processor_coder::{ModCoderParts, ProcessorCoder};
use coder::coder::{Coder, to_snake_case};

use crate::cargo_interface::CargoInterface;
#[derive(PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum ObjectCategory {
    Crate,
    StreamProcBlock,
    Input,
    Output,
    State,
    Static,
    Parameter,
    Application,
    Task,
    StreamProc,
    Connection,
    Setting,
}

impl From<&String> for ObjectCategory {
    fn from(type_str: &String) -> Self {
        match type_str.as_str() {
            "crate" => ObjectCategory::Crate,
            "stream_proc_block" => ObjectCategory::StreamProcBlock,
            "input" => ObjectCategory::Input,
            "output" => ObjectCategory::Output,
            "state" => ObjectCategory::State,
            "static" => ObjectCategory::Static,
            "parameter" => ObjectCategory::Parameter,
            "application" => ObjectCategory::Application,
            "task" => ObjectCategory::Task,
            "stream_proc" => ObjectCategory::StreamProc,
            "connection" => ObjectCategory::Connection,
            "setting" => ObjectCategory::Setting,
            _ => panic!("Unknown object type: {}", type_str),
        }
    }
}

impl Into<String> for ObjectCategory {
    fn into(self) -> String {
        match self {
            ObjectCategory::Crate => "crate".to_string(),
            ObjectCategory::StreamProcBlock => "stream_proc_block".to_string(),
            ObjectCategory::Input => "input".to_string(),
            ObjectCategory::Output => "output".to_string(),
            ObjectCategory::State => "state".to_string(),
            ObjectCategory::Static => "static".to_string(),
            ObjectCategory::Parameter => "parameter".to_string(),
            ObjectCategory::Application => "application".to_string(),
            ObjectCategory::Task => "task".to_string(),
            ObjectCategory::StreamProc => "stream_proc".to_string(),
            ObjectCategory::Connection => "connection".to_string(),
            ObjectCategory::Setting => "setting".to_string(),
        }
    }
}
#[derive(Clone, Serialize, Deserialize)]
pub struct MemoryObject {
    pub parent: String,
    pub object_category: ObjectCategory,
    pub object_type: String,
    pub object_value: String,
    pub object_limits: String,
}

type ParserFunctionReturn = Result<(), String>;
type ParserFunction = fn(&mut Parser, &Vec<String>) -> ParserFunctionReturn;


pub struct Parser {
    commands_fn: HashMap<String, ParserFunction>,
    create_types_fn: HashMap<String, ParserFunction>,
    projects_map: HashMap<String, HashMap<String, MemoryObject>>,
    coder_map: HashMap<String, Box<dyn Coder>>,
    library_path: String,
    cargo_if: CargoInterface,
}

impl Parser {
    fn new() -> Self {
        let mut commands_fn: HashMap<String, ParserFunction> = HashMap::new();
        commands_fn.insert("create".to_string(), Parser::parse_create);
        commands_fn.insert("connect".to_string(), Parser::parse_connect);
        commands_fn.insert("set".to_string(), Parser::parse_set);
        commands_fn.insert("delete".to_string(), Parser::parse_delete);
        commands_fn.insert("code".to_string(), Parser::parse_code);
        commands_fn.insert("build".to_string(), Parser::parse_build);
        commands_fn.insert("import".to_string(), Parser::parse_import);

        let mut create_types_fn: HashMap<String, ParserFunction> = HashMap::new();
        create_types_fn.insert("crate".to_string(), Parser::create_crate);
        create_types_fn.insert("stream_proc_block".to_string(), Parser::create_stream_proc_block);
        create_types_fn.insert("input".to_string(), Parser::create_typed);
        create_types_fn.insert("output".to_string(), Parser::create_typed);
        create_types_fn.insert("state".to_string(), Parser::create_settable);
        create_types_fn.insert("static".to_string(), Parser::create_settable);
        create_types_fn.insert("parameter".to_string(), Parser::create_settable);
        create_types_fn.insert("application".to_string(), Parser::create_application);
        create_types_fn.insert("task".to_string(), Parser::create_task);
        create_types_fn.insert("stream_proc".to_string(), Parser::create_stream_proc);
        
        let cargo_path;
        if let Some(path) = std::env::var_os("HOME") {
            cargo_path = format!("{}/.cargo/bin/cargo", path.into_string().unwrap());
        } else {
            cargo_path = "cargo".to_string();
        }
        Self {
            commands_fn,
            create_types_fn,
            projects_map: HashMap::new(),
            coder_map: HashMap::new(),
            library_path: "".to_string(),
            cargo_if: CargoInterface {
                cargo_path: cargo_path,
                library_path: "".to_string(),
            },
        }
    }
    pub fn set_library_path(&mut self, path: String) -> Result<(), String> {
        let canonical_path = std::fs::canonicalize(&path);
        let canonical_path = match canonical_path {
            Err(_) => return Err("Library path does not exist.".to_string()),
            Ok(p) => p,
        };
        let canonical_path = canonical_path.to_str().unwrap().to_string();
        self.library_path = canonical_path.clone();
        self.cargo_if.library_path = canonical_path;
        Ok(())
    }
    pub fn get() -> &'static Mutex<Parser> {
        PARSER.get_or_init(|| Mutex::new(Parser::new()))
    }
    fn check_var(&self, var_name: &String, expected_type: &String) -> ParserFunctionReturn {
        let split_name = var_name.split(".").collect::<Vec<&str>>();
        let object_map = self.projects_map.get(&split_name[0].to_string()).ok_or_else(|| format!("Project {} not found.", split_name[0]))?;
        if object_map.contains_key(var_name) {
            let memory_object = object_map.get(var_name).unwrap();
            if memory_object.object_category != (&expected_type.clone()).into() {
                return Err(format!("Type mismatch for variable {}: expected {}, found {}.", var_name, expected_type, <ObjectCategory as Into<String>>::into(memory_object.object_category)));
            }
            Ok(())
        } else {
            Err(format!("Object {} not found.", var_name))
        }
    }
    fn insert_in_memory_map(&mut self, project_name: String, object_name: String, object: MemoryObject) -> ParserFunctionReturn {
        let object_map = self.projects_map.get_mut(&project_name).unwrap();
        if object_map.contains_key(&object_name) {
            return Err(format!("Object {} already exists.", object_name));
        }
        object_map.insert(object_name.clone(), object);
        let json_string = serde_json::to_string(&object_map).map_err(|e| format!("Error serializing object map: {}", e))?;
        let coder = self.coder_map.get(&project_name).ok_or_else(|| format!("Coder for project {} not found.", project_name))?;
        std::fs::write(format!("{}/.project/memory_map.json", coder.get_path()), json_string).map_err(|e| format!("Error writing object map file: {}", e))?;
        Ok(())
    }

    fn get_coder<T>(&mut self, coder_name: String) -> Result<&mut T, String>
    where
        T: Coder + 'static,
    {
        match self.coder_map.get_mut(&coder_name) {
            Some(some_coder) => {
                if let Some(coder_mut) = some_coder.as_any_mut().downcast_mut::<T>() {
                    Ok(coder_mut)
                } else {
                    Err(format!("Coder for {} is not of the expected type.", coder_name))
                }
            },
            None => Err(format!("Coder for {} not found.", coder_name)),
        }
    }
    fn create_crate(&mut self, tokens: &Vec<String>) -> ParserFunctionReturn {
        let crate_name = tokens.get(2).ok_or_else(|| "Missing crate name".to_string())?;
        if self.projects_map.contains_key(crate_name) {
            return Err(format!("Crate {} already exists.", crate_name));
        }
        if tokens.get(3) != Some(&"path".to_string()) {
            return Err(format!("Expected path keyword."));
        }
        let crate_folder = tokens.get(4).ok_or_else(|| "Missing crate path".to_string())?;
        if tokens.get(5) != Some(&"metadata".to_string()) {
            return Err(format!("Expected metadata keyword."));
        }
        let metadata = tokens.get(6).ok_or_else(|| "Missing metadata value".to_string())?;
        let crate_path = format!("{}/{}", crate_folder, crate_name);
        self.cargo_if.cargo_new_library(crate_path.to_string())?;
        self.cargo_if.cargo_add_commands(crate_path.to_string())?; 
        let memory_object = MemoryObject {
            parent: "".to_string(),
            object_category: ObjectCategory::Crate,
            object_type: crate_path.clone(),
            object_value: metadata.clone(),
            object_limits: "".to_string(),
        };
        self.projects_map.insert(crate_name.clone(), HashMap::new());
        self.insert_in_memory_map(crate_name.clone(), crate_name.clone(), memory_object)?;
        let mut lib_coder = LibCoder::new(crate_path.clone());
        lib_coder.generate()?;
        self.coder_map.insert(crate_name.clone(), Box::new(lib_coder));
        Ok(())
    }
    fn create_stream_proc_block(&mut self, tokens: &Vec<String>) -> ParserFunctionReturn {
        let block_name = tokens.get(2).ok_or_else(|| "Missing stream processor block name".to_string())?;
        let split_name = block_name.split(".").collect::<Vec<&str>>();
        if split_name.len() != 2 {
            return Err(format!("Stream processor block name must be in the format <crate_name>.<block_name>."));
        }
        self.check_var(&split_name[0].to_string(), &"crate".to_string())?;
        
        let mut lib_coder = self.get_coder::<LibCoder>(split_name[0].to_string())?.clone();
        
        let mut processor_coder = ProcessorCoder::new(lib_coder.get_path(), split_name[1].to_string());
        processor_coder.generate()?;
        self.coder_map.insert(block_name.clone(), Box::new(processor_coder));
        
        let memory_object = MemoryObject {
            parent: split_name[0].to_string(),
            object_category: ObjectCategory::StreamProcBlock,
            object_type: "".to_string(),
            object_value: "".to_string(),
            object_limits: "".to_string(),
        };
        self.insert_in_memory_map(split_name[0].to_string(), block_name.clone(), memory_object)?;
        
        lib_coder.add_module(split_name[1].to_string());
        lib_coder.generate()?;
        self.coder_map.insert(split_name[0].to_string(), Box::new(lib_coder));
        Ok(())
    }
    fn create_typed(&mut self, tokens: &Vec<String>) -> ParserFunctionReturn {
        let object_category = tokens.get(1).ok_or_else(|| "Missing object type".to_string())?;
        let object_name = tokens.get(2).ok_or_else(|| "Missing object name".to_string())?;
        let split_name = object_name.split(".").collect::<Vec<&str>>();
        if split_name.len() != 3 {
            return Err(format!("Settable name must be in the format <>.<>.<>."));
        }
        let parent_block = format!("{}.{}", split_name[0], split_name[1]);
        self.check_var(&parent_block.clone(), &"stream_proc_block".to_string())?;
        if tokens.get(3) != Some(&"type".to_string()) {
            return Err(format!("Expected type keyword."));
        }
        let object_type = tokens.get(4).ok_or_else(|| format!("Missing type"))?;
        let memory_object = MemoryObject {
            parent: parent_block.clone(),
            object_category: (&object_category.clone()).into(),
            object_type: object_type.clone(),
            object_value: "".to_string(),
            object_limits: "".to_string(),
        };
        self.insert_in_memory_map(split_name[0].to_string(), object_name.clone(), memory_object)?;
        let mut coder: ProcessorCoder = self.get_coder::<ProcessorCoder>(parent_block.clone())?.clone();
        coder.add_typed(&object_category.clone(), &split_name[2].to_string(), &object_type.clone());
        coder.generate()?;
        self.coder_map.insert(parent_block.clone(), Box::new(coder));
        Ok(())
    }
    fn create_settable(&mut self, tokens: &Vec<String>) -> ParserFunctionReturn {
        let object_category = tokens.get(1).ok_or_else(|| "Missing object type".to_string())?;
        let object_name = tokens.get(2).ok_or_else(|| "Missing crate name".to_string())?;
        let split_name = object_name.split(".").collect::<Vec<&str>>();
        if split_name.len() != 3 {
            return Err(format!("Input name must be in the format <>.<>.<>."));
        }
        let parent_block = format!("{}.{}", split_name[0], split_name[1]);
        self.check_var(&parent_block, &"stream_proc_block".to_string())?;
        if tokens.get(3) != Some(&"type".to_string()) {
            return Err(format!("Expected type keyword."));
        }
        let object_type = tokens.get(4).ok_or_else(|| format!("Missing type"))?;
        if tokens.get(5) != Some(&"value".to_string()) {
            return Err(format!("Expected value keyword."));
        }
        let object_value = tokens.get(6).ok_or_else(|| format!("Missing value"))?;
        let mut object_limits = None;
        if let Some(limits_key) = tokens.get(7) {
            if limits_key != "limits" {
                return Err(format!("Expected limits keyword."));
            }
            let value_limits = tokens.get(8).ok_or_else(|| format!("Missing limits"))?;
            object_limits = Some(value_limits);
        }
        
        let memory_object = MemoryObject {
            parent: parent_block.clone(),
            object_category: (&object_category.clone()).into(),
            object_type: object_type.clone(),
            object_value: object_value.clone(),
            object_limits: object_limits.unwrap_or(&"".to_string()).clone(),
        };
        self.insert_in_memory_map(split_name[0].to_string(), object_name.clone(), memory_object)?;

        let mut coder: ProcessorCoder = self.get_coder::<ProcessorCoder>(parent_block.clone())?.clone();
        coder.add_settable(&object_category.clone(), &split_name[2].to_string(), &object_type.clone(), &object_value.clone(), object_limits);
        coder.generate()?;
        self.coder_map.insert(parent_block.clone(), Box::new(coder));
        Ok(())
    }
    fn create_application(&mut self, tokens: &Vec<String>) -> ParserFunctionReturn {
        let application_name = tokens.get(2).ok_or_else(|| "Missing application name".to_string())?;
        if self.projects_map.contains_key(application_name) {
            return Err(format!("Application {} already exists.", application_name));
        }
        if tokens.get(3) != Some(&"path".to_string()) {
            return Err(format!("Expected path keyword."));
        }
        let application_folder = tokens.get(4).ok_or_else(|| "Missing application path".to_string())?;
        let metadata = tokens.get(6).ok_or_else(|| "Missing metadata value".to_string())?;
        let application_path = format!("{}/{}", application_folder, application_name);
        self.cargo_if.cargo_new_application(application_path.to_string())?;
        self.cargo_if.cargo_add_commands(application_path.to_string())?; 
        let memory_object = MemoryObject {
            parent: "".to_string(),
            object_category: ObjectCategory::Application,
            object_type: application_path.clone(),
            object_value: metadata.clone(),
            object_limits: "".to_string(),
        };
        self.projects_map.insert(application_name.clone(), HashMap::new());
        self.insert_in_memory_map(application_name.clone(), application_name.clone(), memory_object)?;
        let path = format!("{}/src/main.rs", application_path);
        let mut main_coder = MainCoder::new(path.clone());
        main_coder.generate()?;
        self.coder_map.insert(application_name.clone(), Box::new(main_coder));
        Ok(())
    }
    fn create_task(&mut self, tokens: &Vec<String>) -> ParserFunctionReturn {
        let task_name = tokens.get(2).ok_or_else(|| "Missing task name".to_string())?;
        let split_name = task_name.split(".").collect::<Vec<&str>>();
        if split_name.len() != 2 {
            return Err(format!("Task name must be in the format <>.<>."));
        }
        self.check_var(&split_name[0].to_string(), &"application".to_string())?;
        
        let memory_object = MemoryObject {
            parent: split_name[0].to_string(),
            object_category: ObjectCategory::Task,
            object_type: "".to_string(),
            object_value: "".to_string(),
            object_limits: "".to_string(),
        };
        self.insert_in_memory_map(split_name[0].to_string(), task_name.clone(), memory_object)?;

        let mut main_coder: MainCoder = self.get_coder::<MainCoder>(split_name[0].to_string())?.clone();
        main_coder.add_task_processor(task_name.clone());
        main_coder.generate()?;
        self.coder_map.insert(split_name[0].to_string(), Box::new(main_coder));
        Ok(())
    }
    fn create_stream_proc(&mut self, tokens: &Vec<String>) -> ParserFunctionReturn {
        let object_category = tokens.get(1).ok_or_else(|| "Missing object type".to_string())?;
        let object_name = tokens.get(2).ok_or_else(|| "Missing stream processor name".to_string())?;
        
        let split_name = object_name.split(".").collect::<Vec<&str>>();
        if split_name.len() != 3 {
            return Err(format!("Input name must be in the format <>.<>.<>."));
        }
        let parent_block = format!("{}.{}", split_name[0], split_name[1]);
        self.check_var(&parent_block, &"task".to_string())?;
        if tokens.get(3) != Some(&"type".to_string()) {
            return Err(format!("Expected type keyword."));
        }
        let object_type = tokens.get(4).ok_or_else(|| format!("Missing type"))?;
        let memory_object = MemoryObject {
            parent: split_name[0].to_string(),
            object_category: ObjectCategory::StreamProc,
            object_type: object_category.clone(),
            object_value: "".to_string(),
            object_limits: "".to_string(),
        };
        self.insert_in_memory_map(split_name[0].to_string(), object_name.clone(), memory_object)?;

        let mut main_coder: MainCoder = self.get_coder::<MainCoder>(split_name[0].to_string())?.clone();
        main_coder.add_stream_processor(object_name.clone(), object_type.clone());
        main_coder.generate()?;
        self.coder_map.insert(split_name[0].to_string(), Box::new(main_coder));
        Ok(())
    }
    fn parse_create(&mut self, tokens: &Vec<String>) -> ParserFunctionReturn {
        let key_type = tokens.get(1).ok_or_else(|| "Missing object type".to_string())?;
        let key_type_str: &str = key_type.as_str();
        let create_function: ParserFunction;
        if let Some((_, key_value)) = self.create_types_fn.get_key_value(key_type_str) {
            create_function = *key_value;
        } else {
            return Err(format!("Unknown command: {}", key_type_str));
        }
        create_function(self, tokens)?;
        Ok(())
    }
    fn parse_connect(&mut self, tokens: &Vec<String>) -> ParserFunctionReturn {
        let source_name = tokens.get(1).ok_or_else(|| "Missing source name".to_string())?;
        let target_name = tokens.get(2).ok_or_else(|| "Missing target name".to_string())?;
        let source_split_name = source_name.split(".").collect::<Vec<&str>>();
        if source_split_name.len() != 4 {
            return Err(format!("Connectable object name must be in the format <>.<>.<>.<>."));
        }
        let target_split_name = target_name.split(".").collect::<Vec<&str>>();
        if target_split_name.len() != 4 {
            return Err(format!("Connectable object name must be in the format <>.<>.<>.<>."));
        }
        let from_processor = format!("{}.{}.{}", source_split_name[0], source_split_name[1], source_split_name[2]);
        let to_processor = format!("{}.{}.{}", target_split_name[0], target_split_name[1], target_split_name[2]);
        self.check_var(&from_processor, &"stream_proc".to_string())?;
        self.check_var(&to_processor, &"stream_proc".to_string())?;

        let mut main_coder: MainCoder = self.get_coder::<MainCoder>(source_split_name[0].to_string())?.clone();
        main_coder.add_connection(from_processor.clone(), source_name.clone(), to_processor.clone(), target_name.clone());
        main_coder.generate()?;
        self.coder_map.insert(source_split_name[0].to_string(), Box::new(main_coder));
        Ok(())
    }
    fn parse_set(&mut self, tokens: &Vec<String>) -> ParserFunctionReturn {
        let object_category = tokens.get(1).ok_or_else(|| "Missing variable type".to_string())?;
        let object_name = tokens.get(2).ok_or_else(|| "Missing variable name".to_string())?;
        let split_name = object_name.split(".").collect::<Vec<&str>>();
        if split_name.len() != 4 {
            return Err(format!("Settable object name must be in the format <>.<>.<>."));
        }
        let parent_block = format!("{}.{}.{}", split_name[0], split_name[1], split_name[2]);
        self.check_var(&parent_block, &"stream_proc".to_string())?;
        let value = tokens.get(3).ok_or_else(|| "Missing variable value".to_string())?;

        let mut main_coder: MainCoder = self.get_coder::<MainCoder>(split_name[0].to_string())?.clone();
        main_coder.add_setting_value(parent_block.clone(), object_category.clone(), object_name.clone(), value.clone());
        main_coder.generate()?;
        self.coder_map.insert(split_name[0].to_string(), Box::new(main_coder));
        Ok(())
    }
    fn delete(&mut self, object_name: String) -> Result<(), String> {
        let split_name = object_name.split(".").collect::<Vec<&str>>();
        let object_map = self.projects_map.get_mut(&split_name[0].to_string()).unwrap();
        if split_name.len() == 1 {
            // Application or Crate deletion
            let children = object_map.iter()
                .filter(|(_, v)| v.parent == object_name)
                .map(|(k, _)| k.clone())
                .collect::<Vec<String>>();
            for child in children {
                object_map.retain(|_, v| v.parent != child);
                object_map.retain(|k, _| *k != child);
            }
            object_map.retain(|k, _| k != &object_name);
            self.cargo_if.delete_project(object_name.clone())?;
        } else if split_name.len() == 2 {
            //  Task or stream_proc_block deletion
            object_map.retain(|_, v| v.parent != object_name);
            object_map.retain(|k, _| k != &object_name);
        } else if split_name.len() == 3 {
            // Typed object or stream_proc deletion
            object_map.retain(|k, _| k != &object_name);
        }
        Ok(())
    }
    fn parse_delete(&mut self, tokens: &Vec<String>) -> ParserFunctionReturn {
        let object_name = tokens.get(1).ok_or_else(|| "Missing object name".to_string())?;
        let split_name = object_name.split(".").collect::<Vec<&str>>();
        if split_name.len() == 0 {
            return Err(format!("Invalid object name."));
        }
        let object_map = self.projects_map.get_mut(&split_name[0].to_string()).ok_or_else(|| format!("Project {} does not exist.", split_name[0]))?;
        if !object_map.contains_key(object_name) {
            return Err(format!("Object {} does not exist.", object_name));
        }
        if split_name.len() == 1 {
            self.projects_map.remove(&object_name.clone());
            self.coder_map.remove(object_name);
        } else {
            let object = object_map.get(object_name).unwrap();
            if object.object_category == ObjectCategory::StreamProcBlock {
                self.coder_map.remove(object_name);
            } 
            match object.object_category {
                ObjectCategory::StreamProcBlock => {
                    let mut coder = self.get_coder::<LibCoder>(split_name[0].to_string())?.clone();
                    coder.delete_object(&split_name[1].to_string());
                    coder.generate()?;
                    self.coder_map.insert(split_name[0].to_string(), Box::new(coder));
                }
                ObjectCategory::Task | ObjectCategory::StreamProc=> {
                    let parent_app = split_name[0].to_string();
                    let mut main_coder: MainCoder = self.get_coder::<MainCoder>(parent_app.clone())?.clone();
                    main_coder.delete_object(&object_name.clone());
                    main_coder.generate()?;
                    self.coder_map.insert(parent_app.clone(), Box::new(main_coder));
                }
                _ => {
                    let parent_block = format!("{}.{}", split_name[0], split_name[1]);
                    let mut coder: ProcessorCoder = self.get_coder::<ProcessorCoder>(parent_block.clone())?.clone();
                    coder.delete_object(&object_name.clone());
                    coder.generate()?;
                    self.coder_map.insert(parent_block.clone(), Box::new(coder));
                },
            }
        }
        self.delete(object_name.clone())?;
        Ok(())
    }
    fn parse_code(&mut self, tokens: &Vec<String>) -> ParserFunctionReturn {
        let object_name = tokens.get(1).ok_or_else(|| "Missing processor name".to_string())?;
        let code_string = tokens.get(2).ok_or_else(|| "Missing processor id block".to_string())?;
        match self.check_var(object_name, &"stream_proc".to_string()) {
            Ok(_) => {
                let code_id = ModCoderParts::try_from(code_string.clone())
                    .map_err(|_| format!("Invalid processor code part: {}", code_string))?;
                let code = tokens.get(3).ok_or_else(|| "Missing processor code".to_string())?;
                let mut coder: ProcessorCoder = self.get_coder::<ProcessorCoder>(object_name.clone())?.clone();
                coder.add_code_section(code_id, code.clone());
                coder.generate()?;
                self.coder_map.insert(object_name.clone(), Box::new(coder));
            },
            Err(_) => {
                match self.check_var(object_name, &"application".to_string()) {
                    Ok(_) => {
                        let code_id = MainCoderParts::try_from(code_string.clone())
                            .map_err(|_| format!("Invalid application code part: {}", code_string))?;
                        let code = tokens.get(3).ok_or_else(|| "Missing application code".to_string())?;
                        let mut coder: MainCoder = self.get_coder::<MainCoder>(object_name.clone())?.clone();
                        coder.add_code_section(code_id, code.clone());
                        coder.generate()?;
                        self.coder_map.insert(object_name.clone(), Box::new(coder));
                        
                    },
                    Err(_) => {return Err(format!("Object {} does not allow user code.", object_name));},
                }
            },
        }
        Ok(())
    }
    fn parse_build(&mut self, tokens: &Vec<String>) -> ParserFunctionReturn {
        let build_object_name = tokens.get(1).ok_or_else(|| "Missing artifact name".to_string())?;
        let build_path: String;
        match self.check_var(build_object_name, &"crate".to_string()) {
            Ok(_) => {
                match self.coder_map.get_mut(&build_object_name.clone()) {
                    Some(coder) => {
                        if let Some(lib_coder_mut) = coder.as_any_mut().downcast_mut::<LibCoder>() {
                            build_path = lib_coder_mut.get_path();
                        } else {
                            return Err(format!("Coder for crate {} is not a LibCoder.", build_object_name));
                        }
                    },
                    None => return Err(format!("Coder for crate {} not found.", build_object_name)),
                }
            },
            Err(_) => {
                match self.check_var(build_object_name, &"application".to_string()) {
                    Ok(_) => {
                        let main_coder: MainCoder;
                        match self.coder_map.get_mut(&build_object_name.clone()) {
                            Some(some_coder) => {
                                if let Some(main_coder_mut) = some_coder.as_any_mut().downcast_mut::<MainCoder>() {
                                    build_path = main_coder_mut.get_path();
                                } else {
                                    return Err(format!("Coder for crate {} is not a MainCoder.", build_object_name));
                                }
                            },
                            None => return Err(format!("Coder for crate {} not found.", build_object_name)),
                        }
                    },
                    Err(_) => {return Err(format!("Build target {} is neither a crate nor an application.", build_object_name));},
                }
            },
        }
        
        let build_type: String;
        if let Some(build_type_value) = tokens.get(2) {
            build_type = build_type_value.to_string();
        } else {
            build_type = "debug".to_string();
        }
        
        self.cargo_if.cargo_build(build_path.clone(), build_type.clone())?;
        Ok(())
    }
    pub fn parse_import(&mut self, tokens: &Vec<String>) -> ParserFunctionReturn {
        let import_path = tokens.get(1).ok_or_else(|| "Missing import path".to_string())?;
        let canonical_path = std::fs::canonicalize(&import_path).map_err(|_| "Import path does not exist.".to_string())?;
        let canonical_path_str = canonical_path.to_str().unwrap().to_string();
        let memory_map_file = format!("{}/.project/memory_map.json", canonical_path_str);
        let json_string = std::fs::read_to_string(&memory_map_file).map_err(|e| format!("Error reading import file: {}", e))?;
        let object_map: HashMap<String, MemoryObject> = serde_json::from_str(&json_string).map_err(|e| format!("Error deserializing import file: {}", e))?;
        let project_name = Path::new(&canonical_path_str)
            .parent()
            .and_then(|p| p.parent())
            .and_then(|p| p.file_name())
            .and_then(|f| f.to_str())
            .ok_or_else(|| "Could not determine project name from import path.".to_string())?
            .to_string();
        self.projects_map.insert(project_name.clone(), object_map);
        let main_coder_import_path = format!("{}/.project/main_coder.json", canonical_path_str);
        let lib_coder_import_path = format!("{}/.project/lib_coder.json", canonical_path_str);
        if std::path::Path::new(&main_coder_import_path).exists() {
            let main_coder = MainCoder::load(main_coder_import_path.clone())?;
            self.coder_map.insert(project_name.clone(), Box::new(main_coder));
        } else if std::path::Path::new(&lib_coder_import_path).exists() {
            let lib_coder = LibCoder::load(lib_coder_import_path.clone())?;
            let modules = lib_coder.get_modules();
            for module in modules.iter() {
                let processor_coder_import_path = format!("{}/.project/{}.json", canonical_path_str, module);
                if std::path::Path::new(&processor_coder_import_path).exists() {
                    let processor_coder = ProcessorCoder::load(processor_coder_import_path.clone())?;
                    let processor_coder_name = format!("{}.{}", project_name.clone(), module);
                    self.coder_map.insert(processor_coder_name, Box::new(processor_coder));
                }
            }
            self.coder_map.insert(project_name.clone(), Box::new(lib_coder.clone()));
        }
        Ok(())
    }
    pub fn parse_command(&mut self, command_string: String) -> ParserFunctionReturn {
        let commands: Vec<String> = command_string
            .split(';')
            .map(|s| s.trim().to_string())
            .collect();
        for cmd in commands.iter() {
            let mut tokens: Vec<String> = cmd
                .split(' ')
                .map(|s| s.trim().to_string())
                .collect();
            if tokens.get(0).is_none() {
                return Err("Invalid command format.".to_string());
            }
            while tokens.get(0) == Some(&"".to_string()) {
                // Remove empty strings at the start
                let _ = tokens.remove(0);
            }
            if tokens.is_empty() {
                return Err("Invalid command format.".to_string());
            }
            let key_command = tokens[0].clone();
            let parser_function: ParserFunction;
            if let Some((_, key_value)) = self.commands_fn.get_key_value(&key_command) {
                parser_function = *key_value;
            } else {
                return Err(format!("Unknown command: {}", key_command));
            }
            parser_function(self, &tokens)?;            
        }
        Ok(())
    }
}

static PARSER: OnceLock<Mutex<Parser>> = OnceLock::new();
