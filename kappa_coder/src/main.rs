pub mod library_manager;
pub mod server;
pub mod parser;

use std::env;

use crate::server::Server;


fn print_usage() {
    println!("Usage: kappa_coder [help|[port=port_number] [addr=server_address]]");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut server_port: u16 = 8080;
    let mut server_addr: String = "0.0.0.0".to_string();
    let mut library_path: String = "./libraries".to_string();
    let mut source_path: String = "./sources".to_string();
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
        if arg.contains("library_path") {
            arg.split('=').for_each(|part| {
                library_path = part.to_string();
            });
        }
        if arg.contains("source_path") {
            arg.split('=').for_each(|part| {
                source_path = part.to_string();
            });
        }
    }
    let join_handle = Server::start_coder_server(server_addr, server_port, library_path, source_path);
    join_handle.join().unwrap();
}
