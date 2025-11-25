use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

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
type ParserFunction = fn(&mut Parser, Vec<String>) -> ParserFunctionReturn;


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
        create_types_fn.insert("input".to_string(), Parser::create_input);
        create_types_fn.insert("output".to_string(), Parser::create_output);
        create_types_fn.insert("state".to_string(), Parser::create_state);
        create_types_fn.insert("static".to_string(), Parser::create_static);
        create_types_fn.insert("parameter".to_string(), Parser::create_parameter);
        create_types_fn.insert("application".to_string(), Parser::create_application);
        create_types_fn.insert("task".to_string(), Parser::create_task);
        create_types_fn.insert("stream_proc".to_string(), Parser::create_stream_proc);

        Self {
            commands,
            create_types_fn,
            object_map: HashMap::new(),
            typed_objects: HashMap::new()
        }
    }
    fn create_crate(&mut self, tokens: &Vec<String>) -> ParserFunctionReturn {
        Ok(())
    }
    fn create_stream_proc_block(&mut self, tokens: &Vec<String>) -> ParserFunctionReturn {
        Ok(())
    }
    fn create_input(&mut self, tokens: &Vec<String>) -> ParserFunctionReturn {
        Ok(())
    }
    fn create_output(&mut self, tokens: &Vec<String>) -> ParserFunctionReturn {
        Ok(())
    }
    fn create_state(&mut self, tokens: &Vec<String>) -> ParserFunctionReturn {
        Ok(())
    }
    fn create_static(&mut self, tokens: &Vec<String>) -> ParserFunctionReturn {
        Ok(())
    }
    fn create_parameter(&mut self, tokens: &Vec<String>) -> ParserFunctionReturn {
        Ok(())
    }
    fn create_application(&mut self, tokens: &Vec<String>) -> ParserFunctionReturn {
        Ok(())
    }
    fn create_task(&mut self, tokens: &Vec<String>) -> ParserFunctionReturn {
        Ok(())
    }
    fn create_stream_proc(&mut self, tokens: &Vec<String>) -> ParserFunctionReturn {
        Ok(())
    }
    fn parse_create(&mut self, tokens: &Vec<String>) -> ParserFunctionReturn {
        let key_type = tokens.get(1).ok_or("Missing object type".to_string());
        let create_function: ParserFunction;
        if let (_, key_value) = self.create_types_fn.get_key_value(key_type) {
            create_function = *key_value;
        } else {
            return Err(format!("Unknown command: {}", command_name));
        }
        create_function(&self, tokens)?;
    }
    fn parse_delete(&mut self, tokens: &Vec<String>) -> ParserFunctionReturn {
        Ok(())
    }
    fn parse_connect(&mut self, tokens: &Vec<String>) -> ParserFunctionReturn {
        Ok(())
    }
    fn parse_set(&mut self, tokens: &Vec<String>) -> ParserFunctionReturn {
        Ok(())
    }
    fn parse_build(&mut self, tokens: &Vec<String>) -> ParserFunctionReturn {
        Ok(())
    }
    fn parse_command(&self, command_string: String) -> ParserFunctionReturn {
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
                let _ = sub_parts.remove(0);
            }
            if tokens.is_empty() {
                return Err("Invalid command format.".to_string());
            }
            let key_command = tokens[0].clone();
            let parser_function: ParserFunction;
            if let (_, key_value) = self.commands_fn.get_key_value(key_command) {
                parser_function = *key_value;
            } else {
                return Err(format!("Unknown command: {}", command_name));
            }
            parser_function(&self, tokens)?;
        }
    }
}

static PARSER: OnceLock<Mutex<Parser>> = OnceLock::new();
