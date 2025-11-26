use std::collections::HashMap;
use std::path::Path;
use rand::{Rng, rng, random_range};

#[derive(Clone, Eq, PartialEq, Hash)]
#[repr(u8)]
pub enum ModCoderParts {
    HeadMod,
    UsedDefinedCode,
    HeadStruct,
    UserDefinedStruct,
    EndStruct,
    HeadBuilder,
    UserDefinedBuilder,
    UserMemberCreation,
    UserDefinedImplStruct,
    InitBody,
    RunBody,
    ProcessBody,
    StopBody,
}

impl TryFrom<u8> for ModCoderParts {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ModCoderParts::HeadMod),
            1 => Ok(ModCoderParts::UsedDefinedCode),
            2 => Ok(ModCoderParts::HeadStruct),
            3 => Ok(ModCoderParts::UserDefinedStruct),
            5 => Ok(ModCoderParts::HeadBuilder),
            6 => Ok(ModCoderParts::UserDefinedBuilder),
            7 => Ok(ModCoderParts::UserMemberCreation),
            8 => Ok(ModCoderParts::UserDefinedImplStruct),
            9 => Ok(ModCoderParts::InitBody),
            10 => Ok(ModCoderParts::RunBody),
            11 => Ok(ModCoderParts::ProcessBody),
            12 => Ok(ModCoderParts::StopBody),
            _ => Err(()),
        }
    }
}
impl TryFrom<String> for ModCoderParts {
    type Error = ();
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let num_result: Result<u8, _> = value.parse();
        let int_code: u8 = match num_result {
            Ok(n) => n,
            Err(_) => {return Err(());},
        };
        Self::try_from(int_code)
    }
}
#[derive(Clone)]
pub struct Limits {
    min: String,
    max: String,
}
#[derive(Clone)]
pub struct Typed {
    category: String,
    name: String,
    data_type: String,
    default: String,
    limits: Option<Limits>,
}
#[derive(Clone)]
pub struct ProcessorCoder {
    processor_name: String,
    inputs: HashMap<String, String>,
    outputs: HashMap<String, String>,
    states: HashMap<String, String>,
    statics: HashMap<String, Typed>,
    parameters: HashMap<String, Typed>,
    user_codes: HashMap<ModCoderParts, String>,
    path: String,
    tmp_path: String,
}

impl ProcessorCoder {
    pub fn new(path: String, processor_name: String) -> Self {
        ProcessorCoder {
            processor_name,
            inputs: HashMap::new(),
            outputs: HashMap::new(),
            states: HashMap::new(),
            statics: HashMap::new(),
            parameters: HashMap::new(),
            user_codes: HashMap::new(),
            path,
            tmp_path: "".to_string(),
        }
    }
    fn get_code_file(&mut self) -> Result<std::fs::File, String> {
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        let mut rng = rng();
        let random_string: String = (0..16)
            .map(|_| {
                let idx = rng.random_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect();
        
        self.tmp_path = format!("/tmp/processor_coder_{}.rs", random_string);
        match std::fs::File::create(&self.tmp_path) {
            Ok(file) => Ok(file),
            Err(e) => Err(format!("Error creating file {}: {}", self.path, e)),
        }
    }
    fn file_write(&mut self, content: String, mut code_file: std::fs::File) -> Result<(), String> {
        use std::io::Write;
        match code_file.write_all(content.as_bytes()) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Error writing to file {}: {}", self.path, e)),
        }
    }
    pub fn add_code_section(&mut self, part: ModCoderParts, code: String) {
        self.user_codes.insert(part, code);
    }
    pub fn add_typed(&mut self, category: &String, name: &String, data_type: &String) {
        match category.as_str() {
            "input" => {
                self.inputs.insert(name.clone(), data_type.clone());
            },
            "output" => {
                self.outputs.insert(name.clone(), data_type.clone());
            },
            "state" => {
                self.states.insert(name.clone(), data_type.clone());
            },
            _ => {},
        }
    }
    pub fn add_settable(&mut self, category: &String, name: &String, data_type: &String, default: &String, limits: Option<&String>) {
        let settable = Typed {
            category: category.clone(),
            name: name.clone(),
            data_type: data_type.clone(),
            default: default.clone(),
            limits: limits.map(|lim_str| {
                let parts: Vec<&str> = lim_str.split(",").collect();
                Limits {
                    min: parts.get(0).unwrap_or(&"").to_string(),
                    max: parts.get(1).unwrap_or(&"").to_string(),
                }
            }),
        };
        match category.as_str() {
            "static" => {
                self.statics.insert(name.clone(), settable);
            },
            "parameter" => {
                self.parameters.insert(name.clone(), settable);
            },
            _ => {},
        }
    }
    pub fn delete_object(&mut self, object_name: &String) {
        self.inputs.retain(|k, _| k != object_name);
        self.outputs.retain(|k, _| k != object_name);
        self.states.retain(|k, _| k != object_name);
        self.statics.retain(|k, _| k != object_name);
        self.parameters.retain(|k, _| k != object_name);

    }
    fn generate_head_mod(&self) -> String {
        let mut code_lines: Vec<String> = Vec::new();
        code_lines.push(format!("use std::collections::HashMap;"));
        code_lines.push(format!("use std::any::Any;"));
        code_lines.push(format!("use std::fmt::Display;"));
        code_lines.push(format!("use std::sync::mpsc::SyncSender;"));
        code_lines.push(format!("use std::sync::{{Arc, Mutex}};"));
        code_lines.push(format!("use serde::Serialize;"));
        code_lines.push(format!("use stream_proc_macro::{{StreamBlockMacro}};"));
        code_lines.push(format!("use data_model::streaming_data::{{StreamingError, StreamingState}};"));
        code_lines.push(format!("use data_model::memory_manager::{{DataTrait, StaticsTrait, State, Parameter, Statics}};"));
        code_lines.push(format!("use crate::stream_processor::{{StreamBlock, StreamBlockDyn, StreamProcessor}};"));
        code_lines.push(format!("use crate::connectors::{{ConnectorTrait, Input, Output}};"));
        code_lines.join("\n")
    }
    fn generate_user_defined_code(&self) -> String {
        if let Some(code) = self.user_codes.get(&ModCoderParts::UsedDefinedCode) {
            return code.clone();
        }
        "".to_string()
    }
    fn generate_head_struct(&self) -> String {
        let mut code_lines: Vec<String> = Vec::new();
        code_lines.push(format!("#[derive(StreamBlockMacro)]"));
        code_lines.push(format!("pub struct {} {{", self.processor_name));
        code_lines.push(format!("    name:       &'static str,"));
        code_lines.push(format!("    inputs:     HashMap<&'static str, Box<dyn ConnectorTrait>>,"));
        code_lines.push(format!("    outputs:    HashMap<&'static str, Box<dyn ConnectorTrait>>,"));
        code_lines.push(format!("    parameters: HashMap<&'static str, Box<dyn DataTrait>>,"));
        code_lines.push(format!("    statics:    HashMap<&'static str, Box<dyn StaticsTrait>>,"));
        code_lines.push(format!("    state:      HashMap<&'static str, Box<dyn DataTrait>>,"));
        code_lines.push(format!("    lock:       Arc<Mutex<()>>,"));
        code_lines.push(format!("    proc_state: Arc<Mutex<StreamingState>>,"));
        code_lines.join("\n")
    }
    fn generate_user_defined_struct(&self) -> String {
        let mut code_lines: Vec<String> = Vec::new();
        if let Some(code) = self.user_codes.get(&ModCoderParts::UserDefinedStruct) {
            code_lines.push(code.clone());
        }
        code_lines.push(format!("}}"));
        code_lines.join("\n")
    }
    fn generate_head_builder(&self) -> String {
        let mut code_lines: Vec<String> = Vec::new();
        code_lines.push(format!("impl {} {{", self.processor_name));
        code_lines.push(format!("    pub fn new(name: &'static str) -> Self {{"));
        code_lines.push(format!("        let mut ret = Self {{"));
        code_lines.push(format!("            name,"));
        code_lines.push(format!("            inputs: HashMap::new(),"));
        code_lines.push(format!("            outputs: HashMap::new(),"));
        code_lines.push(format!("            parameters: HashMap::new(),"));
        code_lines.push(format!("            statics: HashMap::new(),"));
        code_lines.push(format!("            state: HashMap::new(),"));
        code_lines.push(format!("            lock: Arc::new(Mutex::new(())),"));
        code_lines.push(format!("            proc_state: Arc::new(Mutex::new(StreamingState::Null)),"));
        code_lines.join("\n")
    }
    fn generate_user_defined_builder(&self) -> String {
        let mut code_lines: Vec<String> = Vec::new();
        if let Some(code) = self.user_codes.get(&ModCoderParts::UserDefinedBuilder) {
            code_lines.push(code.clone());
        }
        code_lines.push(format!("    }};"));
        code_lines.join("\n")
    }
    fn generate_member_creation(&self) -> String {
        let mut code_lines: Vec<String> = Vec::new();
        for (input_name, input_type) in self.inputs.iter() {
            code_lines.push(format!("        ret.new_input::<{}>(\"{}\");", input_type, input_name));
        }
        for (output_name, output_type) in self.outputs.iter() {
            code_lines.push(format!("        ret.new_output::<{}>(\"{}\");", output_type, output_name));
        }
        for (state_name, state_type) in self.states.iter() {
            code_lines.push(format!("        ret.new_state::<{}>(\"{}\");", state_type, state_name));
        }
        for (static_name, static_typed) in self.statics.iter() {
            if let Some(limits) = &static_typed.limits {
                code_lines.push(format!("        ret.new_statics::<{}>(\"{}\", {}, Some(({}, {})));", static_typed.data_type, static_name, static_typed.default, limits.min, limits.max));
            } else {
                code_lines.push(format!("        ret.new_statics::<{}>(\"{}\", {}, None);", static_typed.data_type, static_name, static_typed.default));
            }
        }
        for (param_name, param_typed) in self.parameters.iter() {
            if let Some(limits) = &param_typed.limits {
                code_lines.push(format!("        ret.new_parameter::<{}>(\"{}\", {}, Some(({}, {})));", param_typed.data_type, param_name, param_typed.default, limits.min, limits.max));
            } else {
                code_lines.push(format!("        ret.new_parameter::<{}>(\"{}\", {}, None);", param_typed.data_type, param_name, param_typed.default));
            }
        }
        code_lines.join("\n")
    }
    fn generate_user_member_creation(&self) -> String {
        let mut code_lines: Vec<String> = Vec::new();
        if let Some(code) = self.user_codes.get(&ModCoderParts::UserMemberCreation) {
            code_lines.push(code.clone());
        }
        code_lines.push(format!("        ret"));
        code_lines.push(format!("    }}"));
        code_lines.join("\n")
    }
    fn generate_user_defined_impl_struct(&self) -> String {
        let mut code_lines: Vec<String> = Vec::new();
        if let Some(code) = self.user_codes.get(&ModCoderParts::UserDefinedImplStruct) {
            code_lines.push(code.clone());
        }
        code_lines.push(format!("}}"));
        code_lines.join("\n")
    }
    fn generate_init_body(&self) -> String {
        let mut code_lines: Vec<String> = Vec::new();
        code_lines.push(format!("impl StreamProcessor for {} {{", self.processor_name));
        code_lines.push(format!("    pub fn init(&mut self) -> Result<(), StreamingError> {{"));
        if let Some(code) = self.user_codes.get(&ModCoderParts::InitBody) {
            return code.clone();
        }
        code_lines.push(format!("    }}"));
        code_lines.join("\n")
    }
    fn generate_run_body(&self) -> String {
        let mut code_lines: Vec<String> = Vec::new();
        code_lines.push(format!("    pub fn run(&mut self) -> Result<(), StreamingError> {{"));
        if let Some(code) = self.user_codes.get(&ModCoderParts::RunBody) {
            return code.clone();
        }
        code_lines.push(format!("    }}"));
        code_lines.join("\n")
    }
    fn generate_process_body(&self) -> String {
        let mut code_lines: Vec<String> = Vec::new();
        code_lines.push(format!("    pub fn processes(&mut self) -> Result<(), StreamingError> {{"));
        if let Some(code) = self.user_codes.get(&ModCoderParts::ProcessBody) {
            return code.clone();
        }
        code_lines.push(format!("    }}"));
        code_lines.join("\n")
    }
    fn generate_stop_body(&self) -> String {
        let mut code_lines: Vec<String> = Vec::new();
        code_lines.push(format!("    pub fn stop(&mut self) -> Result<(), StreamingError> {{"));
        if let Some(code) = self.user_codes.get(&ModCoderParts::StopBody) {
            return code.clone();
        }
        code_lines.push(format!("    }}"));
        code_lines.join("\n")
    }
    pub fn generate(&mut self) -> Result<(), String> {
        let mut code_file = self.get_code_file()?;
        let mut code_lines: Vec<String> = Vec::new();
        code_lines.push(self.generate_head_mod());
        code_lines.push(self.generate_user_defined_code());
        code_lines.push(self.generate_head_struct());
        code_lines.push(self.generate_user_defined_struct());
        code_lines.push(self.generate_head_builder());
        code_lines.push(self.generate_user_defined_builder());
        code_lines.push(self.generate_member_creation());
        code_lines.push(self.generate_user_member_creation());
        code_lines.push(self.generate_user_defined_impl_struct());
        code_lines.push(self.generate_init_body());
        code_lines.push(self.generate_run_body());
        code_lines.push(self.generate_process_body());
        code_lines.push(self.generate_stop_body());
        let full_code = code_lines.join("\n");
        self.file_write(full_code, code_file)?;
        std::fs::rename(&self.tmp_path, &self.path).map_err(|e| format!("Error renaming temp file to {}: {}", self.path, e))?;
        Ok(())
    }
}