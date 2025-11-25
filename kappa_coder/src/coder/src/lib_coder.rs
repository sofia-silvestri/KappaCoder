use data_model::modules::{ModuleStruct, Version};

enum LibCoderParts {
    ModulesSection,
    ModuleStructSection,
    StartGetModule,
    BodyGetModule,
    EndGetModule,
}

#[derive(Clone)]
pub struct LibCoder {
    modules: Vec<String>,
    module_structs: ModuleStruct,
    path: String,
}

impl LibCoder {
    pub fn new(path: String) -> Self {
        LibCoder {
            modules: Vec::new(),
            module_structs: ModuleStruct {
                name: String::new(),
                description: String::new(),
                authors: String::new(),
                release_date: String::new(),
                version: Version { major: 0, minor: 0, build: 0 },
                dependencies: Vec::new(),
                provides: Vec::new(),
            },
            path,
        }
    }
    pub fn add_module(&mut self, module_name: String) {
        self.modules.push(module_name);
    }
    fn add_module_section(&self) -> String {
        let mut code_lines: Vec<String> = Vec::new();
        for module in self.modules.iter() {
            code_lines.push(format!("pub mod {};", module));
        }
        code_lines.join("\n")
    }
    fn add_module_struct_section(&self) -> String {
        let mut code_lines: Vec<String> = Vec::new();
        code_lines.push(format!("use std::ffi::c_char;"));
        code_lines.push(format!("use data_model::modules::{{Version,ModuleStructFFI}};"));
        code_lines.push(format!("use processor_engine::stream_processor::StreamProcessor;"));
        code_lines.push(format!("use processor_engine::ffi::{{TraitObjectRepr, export_stream_processor, get_error_return}};"));
        code_lines.push(format!("#[unsafe(no_mangle)]"));
        code_lines.push(format!("pub static MODULE: ModuleStructFFI  = ModuleStructFFI {{"));
        code_lines.push(format!("    name: b\"{}\\0\".as_ptr() as *const c_char,", self.module_structs.name));
        code_lines.push(format!("    description: b\"{}\\0\".as_ptr() as *const c_char,", self.module_structs.description));
        code_lines.push(format!("    authors: b\"{}\\0\".as_ptr() as *const c_char,", self.module_structs.authors));
        code_lines.push(format!("    release_date: b\"{}\\0\".as_ptr() as *const c_char,", self.module_structs.release_date));
        code_lines.push(format!("    version: Version{{ major: {},minor: {},build: {}}},", self.module_structs.version.major, self.module_structs.version.minor, self.module_structs.version.build));
        code_lines.push(format!("    dependencies: ["),);
        for dependency in self.module_structs.dependencies.iter() {
            code_lines.push(format!("        b\"{}\\0\".as_ptr() as *const c_char,", dependency));
        }
        code_lines.push(format!("    ],"));
        code_lines.push(format!("    dependency_number: {},", self.module_structs.dependencies.len()));
        code_lines.push(format!("    provides: ["));
        for provide in self.module_structs.provides.iter() {
            code_lines.push(format!("        b\"{}\\0\".as_ptr() as *const c_char,", provide));
        }
        code_lines.push(format!("    ],"));
        code_lines.push(format!("    provides_lengths: {},", self.module_structs.provides.len()));
        code_lines.push(format!("}};"));
        code_lines.join("\n")
    }
    fn add_start_get_module_section(&self) -> String {
        let mut code_lines: Vec<String> = Vec::new();
        code_lines.push(format!("#[unsafe(no_mangle)]"));
        code_lines.push(format!("pub extern \"C\" fn get_processor_modules(proc_block: *const u8, "));
        code_lines.push(format!("    proc_block_len: usize, "));
        code_lines.push(format!("    block_name: *const u8, "));
        code_lines.push(format!("    block_name_len: usize) -> TraitObjectRepr {{"));
        code_lines.push(format!("    let proc_block_str = unsafe {{"));
        code_lines.push(format!("        std::str::from_utf8(std::slice::from_raw_parts(proc_block, proc_block_len)).unwrap()"));
        code_lines.push(format!("    }};"));
        code_lines.push(format!("    let block_name_str = unsafe {{"));
        code_lines.push(format!("        std::str::from_utf8(std::slice::from_raw_parts(block_name, block_name_len)).unwrap()"));
        code_lines.push(format!("    }};"));
        code_lines.push(format!("    let proc: Box<dyn StreamProcessor>;"));
        code_lines.push(format!("    match proc_block_str {{"));
        code_lines.join("\n")
    }

    fn add_body_get_module_section(&self) -> String {
        let mut code_lines: Vec<String> = Vec::new();
        for module in self.modules.iter() {
            code_lines.push(format!("        \"{}\" => {{", module));
            code_lines.push(format!("            proc = Box::new({}::new(block_name_str));", module));
            code_lines.push(format!("            export_stream_processor(proc)"));
            code_lines.push(format!("        }}"));
        }
        code_lines.join("\n")
    }

    fn add_end_get_module_section(&self) -> String {
        let mut code_lines: Vec<String> = Vec::new();
        code_lines.push(format!("        _ => {{"));
        code_lines.push(format!("            eprintln!(\"Processor block {{}} not found\", proc_block_str);"));
        code_lines.push(format!("            get_error_return(1)"));
        code_lines.push(format!("        }}"));
        code_lines.push(format!("}}"));
        code_lines.join("\n")
    }
    pub fn generate(&self) -> String {
        let mut code_lines: Vec<String> = Vec::new();
        code_lines.push(self.add_module_section());
        code_lines.push(self.add_module_struct_section());
        code_lines.push(self.add_start_get_module_section());
        code_lines.push(self.add_body_get_module_section());
        code_lines.push(self.add_end_get_module_section());
        code_lines.join("\n")
    }
}