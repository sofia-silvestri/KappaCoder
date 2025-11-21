use std::sync::{Arc, Mutex, OnceLock};

type CoderFunction = fn(Vec<String>) -> Result<(), String>;

pub struct Coder {
    code_path: String,
}

impl Coder {
    fn new() -> Self {
        Coder {code_path: "".to_string(),}
    }
    pub fn get() -> Arc<Mutex<Coder>> {
        CODER.get_or_init(|| Arc::new(Mutex::new(Coder::new()))).clone()
    }
    pub fn set_code_path(&mut self, _path: String) -> Result<(), String> {
        self.code_path = _path;
        if self.code_path.is_empty() {
            return Err("Source path cannot be empty.".to_string());
        }
        if !std::path::Path::new(&self.code_path).exists() {
            std::fs::create_dir_all(&self.code_path).map_err(|e| format!("Failed to create source path directory: {}", e))?;
        }
        Ok(())
    }
    pub fn generate(&self, command_parts: Vec<String>) -> Result<(), String> {
        // Placeholder for code generation logic
        println!("Generating code for command parts: {:?}", command_parts);
        match command_parts.get(0) {
            Some(_) | None => todo!(),
        }
        Ok(())
    }
}

static CODER: OnceLock<Arc<Mutex<Coder>>> = OnceLock::new();