use std::ffi::c_void;
use std::env;
use libloading::{Library, Symbol};
use data_model::modules::ModuleStruct;
use processor_engine::stream_processor::StreamProcessor;
use processor_engine::ffi::{TraitObjectRepr, import_stream_processor, c_char_to_string};

fn print_usage() {
    println!("Usage: kappa_coder [help|[port=port_number] [addr=server_address]]");
}
fn main() {
    let args: Vec<String> = env::args().collect();
    let mut server_port: u16 = 8080;
    let mut server_addr: String = "0.0.0.0".to_string();
    for arg in args.into_iter().skip(1) {
        match arg.as_str() {
            "help" => print_usage(),
            _ => print_usage(),
        }
        if arg.contains("port") {
            arg.split('=').for_each(|part| {
                if let Ok(port) = part.parse::<u16>() {
                    server_port = port;
                }
            });
        }
        if arg.contains("addr") {
            arg.split('=').for_each(|part| {
                server_addr = part.to_string();
            });
        }
    }

    let lib = unsafe { Library::new("../../KappaLibrary/target/debug/libdigital_transform.so").unwrap() };
    let module_handle: Symbol<*mut ModuleStruct> = unsafe { lib.get(b"MODULE\0").unwrap() };
    let get_processor_modules: Symbol<unsafe extern "C" fn(*const u8, usize, *const u8, usize) -> TraitObjectRepr> =
        unsafe { lib.get(b"get_processor_modules").unwrap() };
    let a: ModuleStruct = unsafe { **module_handle };

    println!("Loaded module: {}", c_char_to_string(a.name).unwrap());
    unsafe {
        let ptr_proc: TraitObjectRepr = get_processor_modules(b"fft_f32".as_ptr(), 7, b"fft_block".as_ptr(), 9);
        if ptr_proc.vtable.is_null() {
            println!("Processor {} not found", "fft_f32");
            return;
        }
        let processor: Box<dyn StreamProcessor> = import_stream_processor(ptr_proc);
    }
}
