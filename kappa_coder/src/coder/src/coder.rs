use std::io::Write;
use rand::{Rng, rng, random_range};
pub fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c.is_ascii_uppercase() {
            if !result.is_empty() {
                let next_char_is_lowercase = chars.peek().map_or(false, |&next| next.is_ascii_lowercase());
                if next_char_is_lowercase {
                    result.push('_');
                } else if result.chars().last().map_or(false, |last| !last.is_ascii_uppercase() && last != '_') {
                     result.push('_');
                }
            }
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c);
        }
    }
    result
}

pub trait Coder: Send + Sync + std::any::Any {
    fn generate(&mut self) -> Result<(), String>;

    fn get_tmp_file(&self) -> String {
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        let mut rng = rng();
        let random_string: String = (0..16)
            .map(|_| {
                let idx = rng.random_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect();
        
        format!("/tmp/processor_coder_{}.rs", random_string)
    }

    fn file_write(&self, path: String, content: String) -> Result<(), String> {
        let mut file = match std::fs::File::create(&path) {
            Ok(file) => file,
            Err(e) => return Err(format!("Error creating file {}: {}", path, e)),
        };
        match file.write_all(content.as_bytes()) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Error writing to file {}: {}", path, e)),
        }
    }

    fn file_move(&self, src: &String, dest: &String) -> Result<(), String> {
        match std::fs::rename(src, dest) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Error moving file from {} to {}: {}", src, dest, e)),
        }
    }
    fn as_any(&self) -> &dyn std::any::Any;

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}