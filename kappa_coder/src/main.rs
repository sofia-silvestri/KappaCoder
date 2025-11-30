pub mod library_manager;
pub mod server;
pub mod parser;
pub mod cargo_interface;

use std::env;

use crate::server::Server;


fn print_usage() {
    println!("Usage: kappa_coder [help|[port=port_number] [addr=server_address]]");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let application_path = env::current_exe().unwrap();
    let mut server_port: u16 = 8080;
    let mut server_addr: String = "0.0.0.0".to_string();
    let mut dynamic_libraries_path: String = format!("{}/libraries", application_path.parent().unwrap().to_str().unwrap());
    let mut kappa_library_path: String = format!("{}/kappa_library", application_path.parent().unwrap().to_str().unwrap());
    for arg in args.into_iter().skip(1) {
        if arg == "help" {
            print_usage();
            return;
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
        if arg.contains("dynamic_lib") {
            arg.split('=').for_each(|part| {
                dynamic_libraries_path = part.to_string();
            });
        }
        if arg.contains("kappa_lib") {
            arg.split('=').for_each(|part| {
                kappa_library_path = part.to_string();
            });
        }
    }
    let join_handle = Server::start_coder_server(
        server_addr, 
        server_port, 
        dynamic_libraries_path,
        kappa_library_path);
    match join_handle {
        Ok(handle) => handle.join().unwrap(),
        Err(e) => {
            eprintln!("Failed to start coder server: {}", e);
            return;
        }
    };
}
