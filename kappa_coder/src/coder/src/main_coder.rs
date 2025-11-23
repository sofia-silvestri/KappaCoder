use std::collections::HashMap;

enum MainCoderParts {
    HeadMain,
    UsedDefinedCode,
    StreamProcessorCreation,
    StreamProcessorSetup,
    StreamProcessorConnection,
    StreamInit,
    StreamRun,
    StreamStop,
}

pub struct Connections {
    from_processor: String,
    from_output: String,
    to_processor: String,
    to_input: String,
}

pub struct Settings {
    processor_name: String,
    settable_type: String,
    settable_name: String,
    value: String,
}
pub struct MainCoder {
    stream_proc: HashMap<String, String>,
    connections: Vec<Connections>,
    settings: Vec<Settings>,
}
impl MainCoder {
    pub fn new() -> Self {
        MainCoder {
            stream_proc: HashMap::new(),
            connections: Vec::new(),
            settings: Vec::new(),
        }
    }
    pub fn add_stream_processor(&mut self, proc_name: String, proc_type: String) {
        self.stream_proc.insert(proc_name, proc_type);
    }
    pub fn add_connection(&mut self, from_proc: String, from_output: String, to_proc: String, to_input: String) {
        self.connections.push(Connections {
            from_processor: from_proc,
            from_output,
            to_processor: to_proc,
            to_input,
        });
    }
    pub fn add_setting_value(&mut self, proc_name: String, settable_type: String, settable_name: String, value: String) {
        if let Some(blocks) = self.stream_proc.get_mut(&proc_name) {
            self.settings.push(Settings {
                processor_name: proc_name,
                settable_type,
                settable_name,
                value,
            });
        }
    }
    fn create_file_head_block(&self) -> String {
        let mut code_lines: Vec<String> = Vec::new();
        code_lines.join("\n")
    }

    fn create_stream_processor_creation_block(&self) -> String {
        let mut code_lines: Vec<String> = Vec::new();
        code_lines.push("// Stream Processor Creation Section".to_string());
        code_lines.push("let mut processor_engine = ProcessorEngine::get().lock().unwrap();".to_string());
        for (proc_name, proc_type) in self.stream_proc.iter() {
            code_lines.push(format!("let mut {} = {}::new(\"{}\");", proc_name, proc_type, proc_name));
            code_lines.push(format!("processor_engine.register_processor(\"{}\", Box::new({})).unwrap();", proc_name, proc_name));
        }
        code_lines.join("\n")
    }

    fn create_stream_processor_setup_block(&self) -> String {
        let mut code_lines: Vec<String> = Vec::new();
        for setting in self.settings.iter() {
            if setting.settable_type == "parameter" {
                code_lines.push(format!("{}.set_parameter_value::<{}>(\"{}\", \"{}\").unwrap();", setting.processor_name, setting.settable_type, setting.settable_name, setting.value));
            } else if setting.settable_type == "statics" {
                code_lines.push(format!("{}.set_statics_value::<{}>(\"{}\", \"{}\").unwrap();", setting.processor_name, setting.settable_type, setting.settable_name, setting.value));
            }
        }
        code_lines.join("\n")
    }

    fn create_stream_processor_connection_block(&self) -> String {
        let mut code_lines: Vec<String> = Vec::new();
        for connection in self.connections.iter() {
            code_lines.push(format!("let sender = {}.get_input::<_>(\"{}\").unwrap().sender;", connection.to_processor, connection.to_input));
            code_lines.push(format!("{}.connect::<_>({}, sender).unwrap();", connection.from_processor, connection.from_output));
        }
        code_lines.join("\n")
    }

    fn create_stream_init_block(&self) -> String {
        let mut code_lines: Vec<String> = Vec::new();
        code_lines.push("processor_engine.init().unwrap();".to_string());
        code_lines.join("\n")
    }
    fn create_stream_run_block(&self) -> String {
        let mut code_lines: Vec<String> = Vec::new();
        // TODO: Add thread spawning 
        code_lines.join("\n")
    }
    fn create_stream_stop_block(&self) -> String {
        let mut code_lines: Vec<String> = Vec::new();
        code_lines.push("processor_engine.stop().unwrap();".to_string());
        code_lines.join("\n")
    }

    pub fn create_main_rs_file(&self, user_defined_code: Vec<String>) -> String {
        let mut code_lines: Vec<String> = Vec::new();
        code_lines.push("// Auto-generated main.rs file".to_string());
        code_lines.push(self.create_file_head_block());
        code_lines.push("// User-defined code section".to_string());
        code_lines.extend(user_defined_code);
        code_lines.push(self.create_stream_processor_creation_block());
        code_lines.push(self.create_stream_processor_setup_block());
        code_lines.push(self.create_stream_processor_connection_block());
        code_lines.push(self.create_stream_init_block());
        code_lines.push(self.create_stream_run_block());
        code_lines.push(self.create_stream_stop_block());
        code_lines.join("\n")
    }
}

