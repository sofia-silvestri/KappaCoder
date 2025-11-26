use std::collections::HashMap;

#[repr(u8)]
#[derive(PartialEq, Eq, Hash, Clone)]
pub enum MainCoderParts {
    HeadMain,
    UsedDefinedCode,
    StreamProcessorCreation,
    StreamProcessorSetup,
    StreamProcessorConnection,
    StreamProcessorUserCode,
    StreamInit,
    StreamRun,
    StreamStop,
}

impl TryFrom<u8> for MainCoderParts {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(MainCoderParts::HeadMain),
            1 => Ok(MainCoderParts::UsedDefinedCode),
            2 => Ok(MainCoderParts::StreamProcessorCreation),
            3 => Ok(MainCoderParts::StreamProcessorSetup),
            4 => Ok(MainCoderParts::StreamProcessorConnection),
            5 => Ok(MainCoderParts::StreamProcessorUserCode),
            6 => Ok(MainCoderParts::StreamInit),
            7 => Ok(MainCoderParts::StreamRun),
            8 => Ok(MainCoderParts::StreamStop),
            _ => Err(()),
        }
    }
}
impl TryFrom<String> for MainCoderParts {
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
pub struct Connections {
    pub from_processor: String,
    pub from_output: String,
    pub to_processor: String,
    pub to_input: String,
}
#[derive(Clone)]
pub struct Settings {
    pub processor_name: String,
    pub settable_type: String,
    pub settable_name: String,
    pub value: String,
}

#[derive(Clone)]
pub struct TaskProcessor {
    pub name: String,
    pub stream_processors: Vec<String>,
}

#[derive(Clone)]
pub struct MainCoder {
    task_proc: HashMap<String, TaskProcessor>,
    stream_proc: HashMap<String, String>,
    connections: Vec<Connections>,
    settings: Vec<Settings>,
    user_codes: HashMap<MainCoderParts, String>,
    path: String,
}
impl MainCoder {
    pub fn new(path: String) -> Self {
        MainCoder {
            task_proc: HashMap::new(),
            stream_proc: HashMap::new(),
            connections: Vec::new(),
            settings: Vec::new(),
            user_codes: HashMap::new(),
            path,
        }
    }
    pub fn add_task_processor(&mut self, task_name: String) {
        self.task_proc.insert(task_name.clone(), TaskProcessor {
            name: task_name.clone(),
            stream_processors: Vec::new(),
        });
    }
    pub fn add_stream_processor(&mut self, proc_name: String, proc_type: String) {
        let split_name: Vec<&str> = proc_name.split(".").collect();
        let task_name = split_name[0].to_string();
        let stream_proc_name = split_name[1].to_string();
        let mut task_proc = self.task_proc.get_mut(&task_name).unwrap();
        task_proc.stream_processors.push(stream_proc_name);
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
        self.settings.push(Settings {
            processor_name: proc_name,
            settable_type,
            settable_name,
            value,
        });
    }
    pub fn add_code_section(&mut self, part: MainCoderParts, code: String) {
        self.user_codes.insert(part, code);
    }
    pub fn delete_object(&mut self, object_name: &String) {
        let split_name: Vec<&str> = object_name.split(".").collect();
        if split_name.len() == 2 {
            self.task_proc.retain(|k, _| k != object_name);
        } else if split_name.len() == 3 {
            let task_name = format!("{}.{}", split_name[0], split_name[1]);
            if let Some(task_proc) = self.task_proc.get_mut(&task_name) {
                task_proc.stream_processors.retain(|sp| sp != split_name[2]);
            }
        }
    }
    fn create_file_head_block(&self) -> String {
        let mut code_lines: Vec<String> = Vec::new();
        code_lines.join("\n");
        todo!();
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
            code_lines.push(format!("{}.connect::<_>(\"{}\", sender).unwrap();", connection.from_processor, connection.from_output));
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
        for task in self.task_proc.values() {
            // TODO: Add thread spawning 
            code_lines.push(format!(""));
        }
        code_lines.join("\n");
        todo!();
    }
    fn create_stream_stop_block(&self) -> String {
        let mut code_lines: Vec<String> = Vec::new();
        code_lines.push("processor_engine.stop().unwrap();".to_string());
        code_lines.join("\n")
    }

    pub fn generate(&self) -> Result<(), String> {
        let mut code_lines: Vec<String> = Vec::new();
        code_lines.push("// Auto-generated main.rs file".to_string());
        code_lines.push(self.create_file_head_block());
        code_lines.push("// User-defined code section".to_string());
        code_lines.push(self.create_stream_processor_creation_block());
        code_lines.push(self.create_stream_processor_setup_block());
        code_lines.push(self.create_stream_processor_connection_block());
        code_lines.push(self.create_stream_init_block());
        code_lines.push(self.create_stream_run_block());
        code_lines.push(self.create_stream_stop_block());
        code_lines.join("\n");
        Ok(())
    }
}

