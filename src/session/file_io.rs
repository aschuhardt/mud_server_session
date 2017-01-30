//file_io.rs

use std::fs;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

pub struct FileIO { }

impl FileIO {
	pub fn write_string(file_path: &str, contents: &str) {
		let mut file = FileIO::create_file(file_path);
		if let Err(why) = file.write_all(contents.as_bytes()) {
	            panic!("couldn't write to {}: {}", file_path,
	                                               why.description());
	    }
	}

	pub fn read_string(file_path: &str) -> String {
		let mut file = FileIO::open_file(file_path);
	    let mut s = String::new();
	    match file.read_to_string(&mut s) {
	        Err(why) => panic!("couldn't read {}: {}", file_path,
	                                                   why.description()),
	        Ok(_) => s,
	    }
	}

	pub fn delete_file(file_path: &str) {
		if let Err(why) = fs::remove_file(file_path) {
			panic!("couldn't delete file {}: {}", file_path,
												  why.description());
		}
	}

	fn open_file(file_path: &str) -> File {
		let path = Path::new(file_path);
		let display = path.display();
		match File::open(&path) {
        	Err(why) => {
        		panic!("couldn't open {}: {}", display,
            	                               why.description())
        	},
        	Ok(file) => file,
    	}
	}

	fn create_file(file_path: &str) -> File {
		let path = Path::new(file_path);
		let display = path.display();
		match File::create(&path) {
	    	Err(why) => panic!("couldn't create {}: {}", display,
	        	                                       why.description()),
	    	Ok(file) => file,
		}
	}
}
