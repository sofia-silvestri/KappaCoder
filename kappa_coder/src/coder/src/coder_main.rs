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

pub struct MainCoder {
    stream_proc: HashMap<String, String>,
    connections: Vec<(String, String, String, String)>,
    settings: Vec<(String, String, String)>,
}
impl MainCoder {
    fn new() -> Self {
        MainCoder {
            stream_proc: HashMap::new(),
            connections: Vec::new(),
        }
    }
    fn add_stream_processor(&mut self, proc_name: String, proc_type: String) {
        self.stream_proc.insert(proc_name, proc_type);
    }
    fn add_connection(&mut self, from_proc: String, from_output: String, to_proc: String, to_input: String) {
        self.connections.push((from_proc, from_output, to_proc, to_input));
    }
    fn add_setting_value(&mut self, proc_name: String, param_name: String, value: String) {
        if let Some(blocks) = self.stream_proc.get_mut(&proc_name) {
            self.settings.push((proc_name, param_name, value));
        }
    }
    fn create_file_head_block(&self) -> String {
        let mut code_lines: Vec<String> = Vec::new();
        code_lines.join("\n")
    }

    fn create_stream_processor_creation_block(&self) -> String {
        let mut code_lines: Vec<String> = Vec::new();
        for (proc_name, proc_type) in self.stream_proc.iter() {
            code_lines.push(format!("let mut {} = {}::new(\"{}\");", proc_name, proc_type, proc_name));
        }
        code_lines.join("\n")
    }

    fn create_stream_processor_setup_block(&self) -> String {
        let mut code_lines: Vec<String> = Vec::new();
        for (proc_name, proc_varia)
        code_lines.join("\n")
    }

    fn create_stream_processor_connection_block(&self) -> String {
        let mut code_lines: Vec<String> = Vec::new();
        code_lines.join("\n")
    }

    fn create_stream_init_block(&self) -> String {
        let mut code_lines: Vec<String> = Vec::new();
        code_lines.join("\n")
    }
    fn create_stream_run_block(&self) -> String {
        let mut code_lines: Vec<String> = Vec::new();
        code_lines.join("\n")
    }
    fn create_stream_stop_block(&self) -> String {
        let mut code_lines: Vec<String> = Vec::new();
        code_lines.join("\n")
    }

    fn create_main_rs_file(&self, user_defined_code: Vec<String>) -> Vec<String> {
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
        code_lines
    }
}

