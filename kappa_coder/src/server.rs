use std::sync::mpsc;
use crate::library_manager::LibraryManager;
use processor_engine::stream_processor::{StreamProcessor, StreamBlock};
use interfaces::tcp_interface::TcpReceiver;
use crate::parser::Parser;
use crate::coder::Coder;
pub struct Server;

impl Server {
    pub fn start_coder_server(
        server_addr: String,
        server_port: u16,
        library_path: String,
        source_path: String,
    ) -> std::thread::JoinHandle<()> {
        let mut server = Server;
        Coder::get().lock().unwrap().set_code_path(source_path.clone());
        if server.init_library(library_path).is_ok() {
            std::thread::spawn(move || {
                server.run_server(server_port, server_addr).unwrap()
            })
        } else {
            panic!("Failed to initialize coder server.");
        }
    }

    pub fn init_library(&mut self, library_path: String) -> Result<(), String> {
        match LibraryManager::get().lock().unwrap().load_library(&library_path) {
            Ok(_) => println!("Libraries loaded successfully from {}", library_path),
            Err(e) => eprintln!("Error loading libraries: {}", e),
        }
        Ok(())
    }

    pub fn run_server(&mut self, port: u16, address: String) -> Result<(), String>{
        let mut tcp_receiver = TcpReceiver::<String>::new("coder_server");
        tcp_receiver.set_statics_value::<u16>("port", port).unwrap();
        tcp_receiver.set_statics_value::<String>("address", address).unwrap();
        match tcp_receiver.init() {
            Ok(_) => println!("kappa_coder server initialized tcp receiver."),
            Err(e) => return Err(format!("Error initializing tcp receiver: {}", e)),
        }
        println!("kappa_coder server initialized.");
        let (sender, receiver) = mpsc::sync_channel::<String>(1);
        match tcp_receiver.connect("received", sender) {
            Ok(_) => println!("kappa_coder server connected."),
            Err(e) => return Err(format!("Error connecting tcp receiver: {}", e)),
        }
        std::thread::spawn (move || {
            tcp_receiver.run()
        });
        loop {
            let mut command = receiver.recv().unwrap();
            command = command
            .chars()
            .filter(|c| {
                *c != '\n' && *c != '\r'
            })
            .collect();
            //print!("Received {}", command);
            Parser::parse_command(command.clone())?;
            
        }
        println!("kappa_coder server stopped.");
        Ok(())
    }
}