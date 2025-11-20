use std::collections::HashMap;
use std::fs;
use std::sync::{Mutex, OnceLock};
use processor_engine::ffi::ModuleHandle;

pub struct LibraryManager<'a> {
    library_handles: HashMap<String, ModuleHandle<'a>>,
}

impl<'a> LibraryManager<'a> {
    fn new() -> Self {
        LibraryManager {
            library_handles: HashMap::new(),
        }
    }
    pub fn get() -> &'static Mutex<LibraryManager<'a>> {
        LIBRARY_MANAGER.get_or_init(|| {
            Mutex::new(LibraryManager::new())
        })
    }

    pub fn load_library(&mut self, path: &str) -> Result<(), String> {
        // Implementation for loading librariwa
        let entries: fs::ReadDir = fs::read_dir(path).map_err(|e| e.to_string())?;

        for entry in entries {
            let entry = entry.map_err(|e| e.to_string())?; 
            let path = entry.path();

            if path.is_file() {
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if ext == "so" {
                        match ModuleHandle::new(path.display().to_string()) {
                            Ok(handle) => {
                                let module_name = handle.clone().module.name;
                                self.library_handles.insert(module_name.clone(), handle);
                                println!("Loaded module: {}", module_name);
                            }
                            Err(e) => {
                                eprintln!("Failed to load module from {:?}: {:?}", path, e);
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

static LIBRARY_MANAGER: OnceLock<Mutex<LibraryManager>> = OnceLock::new();