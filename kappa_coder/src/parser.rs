use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use coder::coder::Coder;

pub struct MemoryObject {
    parent: String,
    object_type: String,
}

impl MemoryObject {
    pub fn get_parent(&self) -> &String {
        &self.parent
    }
    pub fn get_object_type(&self) -> &String {
        &self.object_type
    }
}
type ParserFunctionReturn = Result<(), String>;
type ParserFunction = fn(&mut Parser, &Vec<String>) -> ParserFunctionReturn;


pub struct Parser {
    commands_fn: HashMap<String, ParserFunction>,
    create_types_fn: HashMap<String, ParserFunction>,
    object_map: HashMap<String, MemoryObject>,
    typed_objects: HashMap<String, String>,
}

impl Parser {
    fn new() -> Self {
        let mut commands_fn: HashMap<String, ParserFunction> = HashMap::new();
        commands_fn.insert("create".to_string(), Parser::parse_create);
        commands_fn.insert("delete".to_string(), Parser::parse_delete);
        commands_fn.insert("connect".to_string(), Parser::parse_connect);
        commands_fn.insert("set".to_string(), Parser::parse_set);
        commands_fn.insert("build".to_string(), Parser::parse_build);

        let mut create_types_fn: HashMap<String, ParserFunction> = HashMap::new();
        create_types_fn.insert("crate".to_string(), Parser::create_crate);
        create_types_fn.insert("stream_proc_block".to_string(), Parser::create_stream_proc_block);
        create_types_fn.insert("input".to_string(), Parser::create_typed);
        create_types_fn.insert("output".to_string(), Parser::create_typed);
        create_types_fn.insert("state".to_string(), Parser::create_typed);
        create_types_fn.insert("static".to_string(), Parser::create_settable);
        create_types_fn.insert("parameter".to_string(), Parser::create_settable);
        create_types_fn.insert("application".to_string(), Parser::create_application);
        create_types_fn.insert("task".to_string(), Parser::create_task);
        create_types_fn.insert("stream_proc".to_string(), Parser::create_stream_proc);

        Self {
            commands_fn,
            create_types_fn,
            object_map: HashMap::new(),
            typed_objects: HashMap::new()
        }
    }
    pub fn get() -> &'static Mutex<Parser> {
        PARSER.get_or_init(|| Mutex::new(Parser::new()))
    }
    fn check_var(&self, var_name: &String, expected_type: &String) -> ParserFunctionReturn {
        if self.object_map.contains_key(var_name) {
            let memory_object = self.object_map.get(var_name).unwrap();
            if &memory_object.object_type != expected_type {
                return Err(format!("Type mismatch for variable {}: expected {}, found {}.", var_name, expected_type, memory_object.object_type));
            }
            Ok(())
        } else {
            Err(format!("Object {} not found.", var_name))
        }
    }
    fn create_crate(&mut self, tokens: &Vec<String>) -> ParserFunctionReturn {
        let crate_name = tokens.get(2).ok_or_else(|| "Missing crate name".to_string())?;
        if self.object_map.contains_key(crate_name) {
            return Err(format!("Crate {} already exists.", crate_name));
        }
        if tokens.get(3) != Some(&"metadata".to_string()) {
            return Err(format!("Expected metadata keyword."));
        }
        let metadata = tokens.get(4).ok_or_else(|| "Missing metadata value".to_string())?;
        let memory_object = MemoryObject {
            parent: "".to_string(),
            object_type: "crate".to_string(),
        };
        self.object_map.insert(crate_name.clone(), memory_object);
        let binding = Coder::get();
        let mut coder = binding.lock().unwrap();
        coder.create_crate(crate_name, metadata)?;
        Ok(())
    }
    fn create_stream_proc_block(&mut self, tokens: &Vec<String>) -> ParserFunctionReturn {
        let block_name = tokens.get(2).ok_or_else(|| "Missing stream processor block name".to_string())?;
        if self.object_map.contains_key(block_name) {
            return Err(format!("Stream processor block {} already exists.", block_name));
        }
        let split_name = block_name.split(".").collect::<Vec<&str>>();
        if split_name.len() != 2 {
            return Err(format!("Stream processor block name must be in the format <crate_name>.<block_name>."));
        }
        self.check_var(&split_name[0].to_string(), &"crate".to_string())?;
        let memory_object = MemoryObject {
            parent: split_name[0].to_string(),
            object_type: "stream_proc_block".to_string(),
        };
        self.object_map.insert(block_name.clone(), memory_object);
        let binding = Coder::get();
        let mut coder = binding.lock().unwrap();
        coder.create_processor_block(block_name.clone())?;
        Ok(())
    }
    fn create_typed(&mut self, tokens: &Vec<String>) -> ParserFunctionReturn {
        let object_type = tokens.get(1).ok_or_else(|| "Missing object type".to_string())?;
        let object_name = tokens.get(2).ok_or_else(|| "Missing crate name".to_string())?;
        if self.object_map.contains_key(object_name) {
            return Err(format!("{} {} already exists.", object_type,object_name));
        }
        let split_name = object_name.split(".").collect::<Vec<&str>>();
        if split_name.len() != 3 {
            return Err(format!("Input name must be in the format <>.<>.<>."));
        }
        let parent_block = format!("{}.{}", split_name[0], split_name[1]);
        self.check_var(&parent_block, &"stream_proc_block".to_string())?;
        if tokens.get(3) != Some(&"type".to_string()) {
            return Err(format!("Expected type keyword."));
        }
        let object_type = tokens.get(4).ok_or_else(|| format!("Missing {} type", object_type))?;
        let memory_object = MemoryObject {
            parent: parent_block,
            object_type: object_type.clone(),
        };
        let binding = Coder::get();
        let mut coder = binding.lock().unwrap();
        coder.create_typed(&object_type.clone(), &object_name.clone(), &object_type.clone())?;
        self.object_map.insert(object_name.clone(), memory_object);
        Ok(())
    }
    fn create_settable(&mut self, tokens: &Vec<String>) -> ParserFunctionReturn {
        let object_type = tokens.get(1).ok_or_else(|| "Missing object type".to_string())?;
        let object_name = tokens.get(2).ok_or_else(|| "Missing crate name".to_string())?;
        if self.object_map.contains_key(object_name) {
            return Err(format!("{} {} already exists.", object_type,object_name));
        }
        let split_name = object_name.split(".").collect::<Vec<&str>>();
        if split_name.len() != 3 {
            return Err(format!("Input name must be in the format <>.<>.<>."));
        }
        let parent_block = format!("{}.{}", split_name[0], split_name[1]);
        self.check_var(&parent_block, &"stream_proc_block".to_string())?;
        if tokens.get(3) != Some(&"type".to_string()) {
            return Err(format!("Expected type keyword."));
        }
        let object_type = tokens.get(4).ok_or_else(|| format!("Missing {} type", object_type))?;
        if tokens.get(5) != Some(&"value".to_string()) {
            return Err(format!("Expected value keyword."));
        }
        let object_value = tokens.get(6).ok_or_else(|| format!("Missing {} value", object_type))?;
        let mut object_limits = None;
        if let Some(limits_key) = tokens.get(7) {
            if limits_key != "limits" {
                return Err(format!("Expected limits keyword."));
            }
            let value_limits = tokens.get(8).ok_or_else(|| format!("Missing {} limits", object_type))?;
            object_limits = Some(value_limits);
        }
        let memory_object = MemoryObject {
            parent: parent_block,
            object_type: object_type.clone(),
        };
        let binding = Coder::get();
        let mut coder = binding.lock().unwrap();
        coder.create_settable(&object_type.clone(), &object_name.clone(), &object_type.clone(), object_limits)?;
        self.object_map.insert(object_name.clone(), memory_object);
        Ok(())
    }
    fn create_application(&mut self, tokens: &Vec<String>) -> ParserFunctionReturn {
        let application_name = tokens.get(2).ok_or_else(|| "Missing application name".to_string())?;
        if self.object_map.contains_key(application_name) {
            return Err(format!("Application {} already exists.", application_name));
        }
        let memory_object = MemoryObject {
            parent: "".to_string(),
            object_type: "application".to_string(),
        };
        self.object_map.insert(application_name.clone(), memory_object);
        let binding = Coder::get();
        let mut coder = binding.lock().unwrap();
        coder.create_application(application_name)?;
        Ok(())
    }
    fn create_task(&mut self, tokens: &Vec<String>) -> ParserFunctionReturn {
        let task_name = tokens.get(2).ok_or_else(|| "Missing task name".to_string())?;
        if self.object_map.contains_key(task_name) {
            return Err(format!("Task {} already exists.", task_name));
        }
        let split_name = task_name.split(".").collect::<Vec<&str>>();
        if split_name.len() != 2 {
            return Err(format!("Task name must be in the format <>.<>."));
        }
        self.check_var(&split_name[0].to_string(), &"application".to_string())?;
        let memory_object = MemoryObject {
            parent: split_name[0].to_string(),
            object_type: "task".to_string(),
        };
        self.object_map.insert(task_name.clone(), memory_object);
        let binding = Coder::get();
        let mut coder = binding.lock().unwrap();
        coder.create_task(&task_name)?;
        Ok(())
    }
    fn create_stream_proc(&mut self, tokens: &Vec<String>) -> ParserFunctionReturn {
        let object_type = tokens.get(1).ok_or_else(|| "Missing object type".to_string())?;
        let object_name = tokens.get(2).ok_or_else(|| "Missing stream processor name".to_string())?;
        if self.object_map.contains_key(object_name) {
            return Err(format!("{} {} already exists.", object_type,object_name));
        }
        let split_name = object_name.split(".").collect::<Vec<&str>>();
        if split_name.len() != 3 {
            return Err(format!("Input name must be in the format <>.<>.<>."));
        }
        let parent_block = format!("{}.{}", split_name[0], split_name[1]);
        self.check_var(&parent_block, &"task".to_string())?;
        if tokens.get(3) != Some(&"type".to_string()) {
            return Err(format!("Expected type keyword."));
        }
        let object_type = tokens.get(4).ok_or_else(|| format!("Missing {} type", object_type))?;
        let memory_object = MemoryObject {
            parent: parent_block,
            object_type: object_type.clone(),
        };
        let binding = Coder::get();
        let mut coder = binding.lock().unwrap();
        coder.create_processor(&object_type.clone(), &object_name.clone(), &object_type.clone())?;
        self.object_map.insert(object_name.clone(), memory_object);
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
    fn delete(&mut self, object_name: String) {
        let split_name = object_name.split(".").collect::<Vec<&str>>();
        if split_name.len() == 1 {
            // Application or Crate deletion
            self.object_map.retain(|_, v| v.parent != object_name);
        } else if split_name.len() == 2 {
            //  Task or stream_proc_block deletion
            let parent_name = split_name[0].to_string();
            self.object_map.retain(|_, v| v.parent != parent_name);
        } else if split_name.len() == 3 {
            // Typed object or stream_proc deletion
            let parent_name = format!("{}.{}", split_name[0], split_name[1]);
            self.object_map.retain(|_, v| v.parent != parent_name);
        }
    }
    fn parse_delete(&mut self, tokens: &Vec<String>) -> ParserFunctionReturn {
        let object_name = tokens.get(1).ok_or_else(|| "Missing object name".to_string())?;
        if !self.object_map.contains_key(object_name) {
            return Err(format!("Object {} does not exist.", object_name));
        }
        self.delete(object_name.clone());
        let binding = Coder::get();
        let mut coder = binding.lock().unwrap();
        coder.delete_object(&object_name)?;
        Ok(())
    }
    fn parse_connect(&mut self, tokens: &Vec<String>) -> ParserFunctionReturn {
        let source_name = tokens.get(1).ok_or_else(|| "Missing source name".to_string())?;
        let target_name = tokens.get(2).ok_or_else(|| "Missing target name".to_string())?;
        self.check_var(source_name, &"output".to_string())?;
        self.check_var(target_name, &"input".to_string())?;
        let binding = Coder::get();
        let mut coder = binding.lock().unwrap();
        coder.connect(source_name, target_name)?;
        Ok(())
    }
    fn parse_set(&mut self, tokens: &Vec<String>) -> ParserFunctionReturn {
        let object_type = tokens.get(1).ok_or_else(|| "Missing variable type".to_string())?;
        let object_name = tokens.get(2).ok_or_else(|| "Missing variable name".to_string())?;
        let split_name = object_name.split(".").collect::<Vec<&str>>();
        if split_name.len() != 4 {
            return Err(format!("Settable object name must be in the format <>.<>.<>."));
        }
        let parent_block = format!("{}.{}.{}", split_name[0], split_name[1], split_name[2]);
        self.check_var(&parent_block, &"stream_proc".to_string())?;
        let value = tokens.get(3).ok_or_else(|| "Missing variable value".to_string())?;
        let binding = Coder::get();
        let mut coder = binding.lock().unwrap();
        coder.set_value(object_type, object_name, value)?;
        Ok(())
    }
    fn parse_build(&mut self, tokens: &Vec<String>) -> ParserFunctionReturn {
        let build_object_name = tokens.get(1).ok_or_else(|| "Missing artifact name".to_string())?;
        let build_object_type;
        match self.check_var(build_object_name, &"crate".to_string()) {
            Ok(_) => {build_object_type = "crate".to_string();},
            Err(_) => {
                match self.check_var(build_object_name, &"application".to_string()) {
                    Ok(_) => {build_object_type = "application".to_string();},
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
        let binding = Coder::get();
        let mut coder = binding.lock().unwrap();
        coder.build(&build_object_name, &build_type)?;
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
