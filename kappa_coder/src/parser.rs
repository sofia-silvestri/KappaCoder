use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use crate::coder::Coder;

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

type ParserFunction = fn(&mut Parser, Vec<String>) -> Result<(), String>;

pub struct Parser {
    pub commands: HashMap<String, ParserFunction>,
    types : Vec<String>,
    memory_objects: HashMap<String, MemoryObject>,
    connections_list: Vec<(String, String)>,
    typed: HashMap<String, String>,
    settable: HashMap<String, String>,
}

impl Parser {
    fn new() -> Self {
        let mut commands: HashMap<String, ParserFunction> = HashMap::new();
        commands.insert("create".to_string(), Parser::parse_create);
        commands.insert("add".to_string(), Parser::parse_add);
        commands.insert("delete".to_string(), Parser::parse_delete);
        commands.insert("connect".to_string(), Parser::parse_connect);
        commands.insert("set".to_string(), Parser::parse_set);
        commands.insert("build".to_string(), Parser::parse_build);
        let mut types = Vec::<String>::new();
        types.push("crate".to_string());
        types.push("task".to_string());
        types.push("processor".to_string());
        types.push("code".to_string());
        types.push("input".to_string());
        types.push("output".to_string());
        types.push("parameter".to_string());
        types.push("static".to_string());
        types.push("state".to_string());
        Parser {
            commands,
            types,
            memory_objects: HashMap::new(),
            connections_list: Vec::new(),
            typed: HashMap::new(),
            settable: HashMap::new(),
        }
    }
    pub fn get() -> &'static Mutex<Parser> {
        PARSER.get_or_init(|| Mutex::new(Parser::new()))
    }
    fn parse_create_crate(&mut self, name: &String) -> Result<(), String> {
        println!("Creating crate with name {}", name);
        let new_object = MemoryObject {
            parent: "root".to_string(),
            object_type: "crate".to_string(),
        };
        self.memory_objects.insert(name.to_string(), new_object);
        Ok(())
    }
    fn parse_create_task(&mut self, name: &String) {
        println!("Creating task with name {}", name);
        let new_object = MemoryObject {
            parent: "root".to_string(),
            object_type: "task".to_string(),
        };
        self.memory_objects.insert(name.to_string(), new_object);
    }
    fn parse_create_processor(&mut self, name: &String, args: &Vec<String>) -> Result<(), String> {
        let mut parent_name = "root";
        if Some(&"in".to_string()) == args.get(2) {
            parent_name = args.get(3).ok_or("Missing parent name argument.".to_string())?;
            if !self.memory_objects.contains_key(parent_name) {
                return Err(format!("Parent object {} does not exist.", parent_name));
            }
            let (_, value) = self.memory_objects.get_key_value(parent_name).unwrap();
            if value.object_type != "crate" || value.object_type != "task" {
                return Err(format!("Processor can be added only to task or library"));
            }
        }
        println!("Creating processor with name {} in parent {}", name, parent_name);
        let new_object = MemoryObject {
            parent: parent_name.to_string(),
            object_type: "processor".to_string(),
        };
        self.memory_objects.insert(name.to_string(), new_object);
        Ok(())
    }
    pub fn parse_create_code(&mut self, name: &String, args: &Vec<String>) -> Result<(), String> {
        if args.get(1) != Some(&"in".to_string()) {
            return Err("Missing in keyword".to_string());
        }
        let parent_name = args.get(2).ok_or("Missing parent name argument.".to_string())?;
        if !self.memory_objects.contains_key(parent_name) {
            return Err(format!("Parent object {} does not exist.", parent_name));
        }
        let (_, value) = self.memory_objects.get_key_value(parent_name).unwrap();
        if value.object_type != "processor" && value.object_type != "taask" {
            return Err(format!("Cannot add code to non-processor object {}.", parent_name));
        }
        println!("Creating code in parent {}", parent_name);
        let new_object = MemoryObject {
            parent: parent_name.to_string(),
            object_type: "processor".to_string(),
        };
        self.memory_objects.insert(name.to_string(), new_object);
        Ok(())
    }
    fn parse_create_typed(&mut self, name: &String, args: &Vec<String>, _type: &String) -> Result<(), String> {
        if args.get(2) != Some(&"type".to_string()) {
            return Err("Expected 'type' keyword.".to_string());
        }
        let type_name = args.get(3).ok_or("Missing type name argument.".to_string())?;
        self.typed.insert(name.to_string(), type_name.to_string());
        if args.get(4) == Some(&"in".to_string()) {
            let parent_name = args.get(5).ok_or("Missing parent name argument.".to_string())?;
            if !self.memory_objects.contains_key(parent_name) {
                return Err(format!("Parent object {} does not exist.", parent_name));
            }
            let (_, value) = self.memory_objects.get_key_value(parent_name).unwrap();
            if value.object_type != "processor" {
                return Err(format!("Cannot add {} to non-processor object {}.", _type, parent_name));
            }
            println!("Creating {} with name {} of type {} in parent {}", _type, name, type_name, parent_name);
            let new_object = MemoryObject {
                parent: parent_name.to_string(),
                object_type: _type.to_string(),
            };
            self.memory_objects.insert(name.to_string(), new_object);
        } else {
            println!("Creating {} with name {} of type {} with no parent", _type, name, type_name);
            let new_object = MemoryObject {
                parent: "root".to_string(),
                object_type: _type.to_string(),
            };
            self.memory_objects.insert(name.to_string(), new_object);
        }
        Ok(())
    }
    pub fn parse_create_settable(&mut self, name: &String, args: &Vec<String>, _type: &String) -> Result<(), String> {
        if args.get(2) != Some(&"type".to_string()) {
            return Err("Expected 'type' keyword.".to_string());
        }
        let type_name = args.get(3).ok_or("Missing type name argument.".to_string())?;
        let default_flag = args.get(4).ok_or("Missing 'default' argument.".to_string())?;
        if default_flag != "default" {
            return Err("Expected 'default' keyword.".to_string());
        }
        let default_value = args.get(5).ok_or("Missing default value argument.".to_string())?;
        self.settable.insert(name.to_string(), default_value.to_string());
        if args.get(6) == Some(&"in".to_string()) {
            let parent_name = args.get(7).ok_or("Missing parent name argument.".to_string())?;
            if !self.memory_objects.contains_key(parent_name) {
                return Err(format!("Parent object {} does not exist.", parent_name));
            }
            let (_, value) = self.memory_objects.get_key_value(parent_name).unwrap();
            if value.object_type != "processor" {
                return Err(format!("Cannot add {} to non-processor object {}.", _type, parent_name));
            }
            println!("Creating {} with name {} of type {} with default {} in parent {}", _type, name, type_name, default_value, parent_name);
            let new_object = MemoryObject {
                parent: parent_name.to_string(),
                object_type: _type.to_string(),
            };
            self.memory_objects.insert(name.to_string(), new_object);
        } else {
            println!("Creating {} with name {} of type {} with default {} with no parent", _type, name, type_name, default_value);
            let new_object = MemoryObject {
                parent: "root".to_string(),
                object_type: _type.to_string(),
            };
            self.memory_objects.insert(name.to_string(), new_object);
        }
        Ok(())
    }
    pub fn parse_create(&mut self, args: Vec<String>) -> Result<(), String> {
        // Implement create command parsing logic here
        if self.types.contains(&args[0]) {
            let _type = args.get(0).ok_or("Missing type argument.".to_string())?;
            let name = args.get(1).ok_or("Missing name argument.".to_string())?;
            if self.memory_objects.contains_key(name) {
                return Err(format!("Object with name {} already exists.", name));
            }
            match _type.as_str() {
                "crate" => {
                    if args.len() > 2 {
                        return Err("Invalid argument for crate creation".to_string());
                    }
                    self.parse_create_crate(name)?;
                }
                "task" => {
                    self.parse_create_task(name);
                    return Err("Task creation not implemented yet.".to_string());
                }
                "processor" => {
                    self.parse_create_processor(name, &args)?;
                }
                "input" | "output" | "state" => {
                    self.parse_create_typed(name, &args, _type)?;
                }
                "parameter" | "static" => {
                    self.parse_create_settable(name, &args, _type)?;
                }
                "code" => {
                    self.parse_create_code(name, &args)?;
                }
                _ => {
                    // Dosen't need but rust wants it
                    return Err(format!("Unknown type: {}", _type));
                }
            }
        } else {
            return Err(format!("Unknown type: {}", args[0]));
        }
        Ok(())
    }
    pub fn parse_add(&mut self, args: Vec<String>) -> Result<(), String> {
        let name = args.get(0).ok_or("Missing name argument.".to_string())?;
        if !self.memory_objects.contains_key(name) {
            return Err(format!("Object with name {} does not exists.", name));
        }
        let in_flag = args.get(1).ok_or("Missing 'in' argument.".to_string())?;
        if in_flag != "in" {
            return Err("Expected 'in' keyword.".to_string());
        }
        let parent_name = args.get(2).ok_or("Missing parent name argument.".to_string())?;
        if !self.memory_objects.contains_key(parent_name) {
            return Err(format!("Parent object {} does not exist.", parent_name));
        }
        let (_, value) = self.memory_objects.get_key_value(parent_name).unwrap();
        let (_, child_value) = self.memory_objects.get_key_value(name).unwrap();
        match child_value.object_type.as_str() {
            "input" | "output" | "parameter" | "static" | "state" => {
                if value.object_type != "processor" {
                    return Err(format!("Cannot add {} to non-processor object {}.", child_value.object_type, parent_name));
                }
            }
            "processor" => {
                if value.object_type != "crate" {
                    return Err(format!("Cannot add processor to non-crate object {}.", parent_name));
                }
            }
            "crate" => {
                return Err("Cannot add crate inside another object.".to_string());
            }
            _ => {}
        }
        println!("Adding {} to parent {}", name, parent_name);
        let updated_object = MemoryObject {
            parent: parent_name.to_string(),
            object_type: child_value.object_type.clone(),
        };
        self.memory_objects.insert(name.to_string(), updated_object);
        Ok(())
    }
    pub fn parse_delete(&mut self, args: Vec<String>) -> Result<(), String> {
        let name = args.get(0).ok_or("Missing name argument.".to_string())?;
        if !self.memory_objects.contains_key(name) {
            return Err(format!("Object with name {} does not exists.", name));
        }
        let (_, value) = self.memory_objects.get_key_value(name).unwrap();
        if value.object_type == "crate" || value.object_type == "processor" {
            let children: Vec<String> = self.memory_objects.iter()
                .filter(|(_, obj)| obj.parent == *name)
                .map(|(k, _)| k.clone())
                .collect();
            if !children.is_empty() {
                return Err(format!("Cannot delete crate {} because it has child objects.", name));
            }
        }
        println!("Deleting object {}", name);
        self.memory_objects.remove(name);
        Ok(())
    }
    pub fn parse_connect(&mut self, args: Vec<String>) -> Result<(), String> {
        let name = args.get(0).ok_or("Missing name argument.".to_string())?;
        if !self.memory_objects.contains_key(name) {
            return Err(format!("Object with name {} does not exists.", name));
        }
        let (_, value) = self.memory_objects.get_key_value(name).unwrap();
        if value.object_type != "output" {
            return Err(format!("Object {} is not an output.", name));
        }
        let to_flag = args.get(1).ok_or("Missing 'to' argument.".to_string())?;
        if to_flag != "to" {
            return Err("Expected 'to' keyword.".to_string());
        }
        let target_name = args.get(2).ok_or("Missing target name argument.".to_string())?;
        if !self.memory_objects.contains_key(target_name) {
            return Err(format!("Target object {} does not exist.", target_name));
        }
        let (_, target_value) = self.memory_objects.get_key_value(target_name).unwrap();
        if target_value.object_type != "input" {
            return Err(format!("Target object {} is not an input.", target_name));
        }
        println!("Connecting output {} to input {}", name, target_name);
        self.connections_list.push((name.to_string(), target_name.to_string()));
        Ok(())
    }
    pub fn parse_set(&mut self, args: Vec<String>) -> Result<(), String> {
        let name = args.get(0).ok_or("Missing name argument.".to_string())?;
        if !self.memory_objects.contains_key(name) {
            return Err(format!("Object with name {} does not exists.", name));
        }
        let (_, value) = self.memory_objects.get_key_value(name).unwrap();
        if value.object_type != "parameter" && value.object_type != "static" {
            return Err(format!("Object {} is not an settable.", name));
        }
        println!("Setting {} to value {}", name, args[1]);
        self.settable.insert(name.to_string(), args[1].clone());
        Ok(())
    }
    pub fn parse_build(&mut self, args: Vec<String>) -> Result<(), String> {
        // Implement set command parsing logic here
        Ok(())
    }
    pub fn parse_command(data: String) -> Result<(), String> {
        let parts: Vec<String> = data
            .split(';')
            .map(|s| s.trim().to_string())
            .collect();
        for part in parts.iter() {
            let mut sub_parts: Vec<String> = part
                .split(' ')
                .map(|s| s.trim().to_string())
                .collect();
            
            if sub_parts.get(0).is_none() {
                return Err("Invalid command format.".to_string());
            }
            while sub_parts.get(0) == Some(&"".to_string()) {
                // Remove empty strings at the start
                let _ = sub_parts.remove(0);
            }
            if sub_parts.is_empty() {
                return Err("Invalid command format.".to_string());
            }
            let command_name = sub_parts[0].clone();
            let parser_function : ParserFunction;
            {
                let parser = Parser::get().lock().unwrap();
                if let Some((_key, key_value)) = parser.commands.get_key_value(&command_name) {
                    parser_function = *key_value;
                } else {
                    return Err(format!("Unknown command: {}", command_name));
                }
            }
            let args = sub_parts[1..].to_vec();
            parser_function(&mut *Parser::get().lock().unwrap(), args)?;
            Coder::get().lock().unwrap().generate(sub_parts.clone())?;
        }
        Ok(())
    }
}

static PARSER: OnceLock<Mutex<Parser>> = OnceLock::new();