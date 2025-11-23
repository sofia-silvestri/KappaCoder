use std::sync::mpsc;
use crate::library_manager::LibraryManager;
use processor_engine::stream_processor::{StreamProcessor, StreamBlock};
use interfaces::tcp_interface::{TcpReceiver, TcpMessage};
use crate::parser::Parser;
use coder::coder::Coder;
pub struct Server;

impl Server {
    pub fn start_coder_server(
        server_addr: String,
        server_port: u16,
        dynamic_libraries: String,
        kappa_library: String,
        source_path: String,
    ) -> Result<std::thread::JoinHandle<()>, String> {
        let mut server = Server;
        Coder::get().lock().unwrap().set_code_path(source_path.clone())?;
        Coder::get().lock().unwrap().set_library_path(kappa_library.clone())?;
        if server.init_library(dynamic_libraries).is_ok() {
            Ok(std::thread::spawn(move || {
                server.run_server(server_port, server_addr).unwrap()
            }))
        } else {
            panic!("Failed to initialize coder server.");
        }
    }

    pub fn init_library(&mut self, dynamic_libraries: String) -> Result<(), String> {
        match LibraryManager::get().lock().unwrap().load_library(&dynamic_libraries) {
            Ok(_) => println!("Libraries loaded successfully from {}", dynamic_libraries),
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
        let (sender, receiver) = mpsc::sync_channel::<TcpMessage<String>>(1);
        match tcp_receiver.connect("received", sender) {
            Ok(_) => println!("kappa_coder server connected."),
            Err(e) => return Err(format!("Error connecting tcp receiver: {}", e)),
        }
        let sender = tcp_receiver.get_input::<TcpMessage<String>>("response").unwrap().sender.clone();
        std::thread::spawn (move || {
            tcp_receiver.run()
        });
        
        loop {
            let command = receiver.recv().unwrap();
            let id_stream = command.id_stream;
            let mut command = command.message;
            command = command
            .chars()
            .filter(|c| {
                *c != '\n' && *c != '\r'
            })
            .collect();
            //print!("Received {}", command);
            let answer: TcpMessage<String>;
            match Parser::parse_command(command.clone()) {
                
                Ok(_) => {
                    answer = TcpMessage {
                        id_stream,
                        message: format!("Ok\n"),
                    }
                },
                Err(e) => {
                    answer = TcpMessage {
                        id_stream,
                        message: format!("Error: {}\n", e),
                    }
                }
            }
            sender.send(answer).unwrap();            
        }
    }
}