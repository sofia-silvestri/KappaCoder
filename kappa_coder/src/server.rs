use std::sync::mpsc;
use crate::library_manager::LibraryManager;
use processor_engine::stream_processor::{StreamProcessor, StreamBlock};
use data_model::streaming_data::StreamingError;
use interfaces::tcp_interface::TcpReceiver;
use crate::parser::Parser;

pub struct Server {
    code_path: String,
}

impl Server {
    pub fn start_coder_server(
        server_addr: String,
        server_port: u16,
        library_path: String,
        source_path: String,
    ) -> std::thread::JoinHandle<()> {
        let mut coder = Server {
            code_path: source_path,
        };
        if coder.init_library(library_path).is_ok() {
            std::thread::spawn(move || {
                coder.run_server(server_port, server_addr).unwrap()
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

    pub fn run_server(&mut self, port: u16, address: String) -> Result<(), StreamingError>{
        let mut tcp_receiver = TcpReceiver::<String>::new("coder_server");
        tcp_receiver.set_statics_value::<u16>("port", port)?;
        tcp_receiver.set_statics_value::<String>("address", address)?;
        tcp_receiver.init()?;
        println!("kappa_coder server initialized.");
        let (sender, receiver) = mpsc::sync_channel::<String>(1);
        tcp_receiver.connect("received", sender)?;
        std::thread::spawn (move || {
            tcp_receiver.run()
        });
        loop {
            let command = receiver.recv().unwrap();
            if command.contains("exit") {
                break;
            }
            print!("Received {}", command);
            let parsed_commands = Parser::parse_command(command.clone());
            match parsed_commands {
                Ok(commands) => { },
                Err(e) => {
                    eprintln!("Error parsing command: {}", e);
                    continue;
                }
            }
            
        }
        self.stop();
        Ok(())
    }

    pub fn stop(&mut self) {
        println!("kappa_coder server stopped.");
    }
}