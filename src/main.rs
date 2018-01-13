#[macro_use] extern crate nickel;
extern crate hyper;
extern crate regex;

use std::env;
use std::error::Error;
use std::fmt;
use std::fs::{File, OpenOptions, create_dir_all};
use std::io::{Read, Write};
use std::path::Path;

use nickel::{Nickel, HttpRouter};
use regex::Regex;



#[derive(Debug)]
struct VooError(&'static str);

const MAX_SIZE: usize = 1024 * 1024;


fn main() {
    match voorhees() {
        Ok(_) => (),
        Err(error) => eprintln!("Failed: {}", error),
    }
}


fn voorhees() -> Result<(), Box<Error>> {
    let mut args = env::args();

    args.next().ok_or("No arguments")?;

    let store_directory = args.next().unwrap_or_else(|| "store".to_string());
    let restore_directory = store_directory.clone();

    let bind_to = args.next().unwrap_or_else(|| "localhost:6767".to_string());

    let id_pattern = Regex::new(r#"\A[-_\da-zA-Z]+\z"#)?;

    let mut serv = Nickel::new();

    serv.get("/json/:id", middleware! { |request|
        let path = Path::new(&restore_directory);
        let id = request.param("id").unwrap();
        restore(&path, id).unwrap()
    });

    serv.post("/json/:id", middleware! { |request, response| {
        let mut buf = String::new();
        {
            request.origin.read_to_string(&mut buf).unwrap();
        }

        let id = request.param("id").unwrap();
        if id_pattern.is_match(id) {
            let path = Path::new(&store_directory);
            store(&path, id, &buf).unwrap();
            format!("OK: {} bytes", buf.len())
        } else {
            format!("Invalid id")
        }
    }});

    serv.listen(bind_to)?;
    Ok(())
}


fn store(dir: &Path, id: &str, content: &str) -> Result<(), Box<Error>> {
    if MAX_SIZE < content.len() {
        return Err(VooError("Too large content"))?;
    }

    let mut path = dir.to_path_buf();
    create_dir_all(&path)?;
    path.push(id);

    let mut file = OpenOptions::new().read(false).write(true).append(false).create(true).open(path)?;
    file.write(content.as_bytes())?;

    Ok(())
}


fn restore(dir: &Path, id: &str) -> Result<String, Box<Error>> {
    let mut path = dir.to_path_buf();
    path.push(id);

    let mut result = String::new();
    let mut file = File::open(path)?;
    file.read_to_string(&mut result)?;

    Ok(result)
}


impl fmt::Display for VooError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for VooError {
    fn description(&self) -> &str {
        self.0
    }

    fn cause(&self) -> Option<&Error> {
        None
    }
}
