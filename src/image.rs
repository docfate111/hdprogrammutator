pub use crate::fileobject::*;
pub use std::fs::File;
//use std::path::Path;

pub struct Image {
    pub file_objs: Vec<FileObject>,
}

impl fmt::Display for Image {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut output = String::from("Image\n");
        for obj in self.file_objs.iter() {
            output.push_str(&format!("{} ", *obj));
        }
        write!(f, "{}", output)
    }
}

/*
Image stat format:
[ fileobjs_num ]
    [ len | relative path
    | type
    | xattr_num
        [ name_len | name ]
    ]
*/
impl Image {
    pub fn new() -> Self {
        Self {
            file_objs: Vec::<FileObject>::new(),
        }
    }
    /*pub fn add_obj(&mut self, f: FileObject) {
        self.file_objs.push(f);
    }
    pub fn from_file<P: AsRef<Path>>(&mut self, path: P) -> std::io::Result<()> {
        //let file = File::open(path)?;
        println!("TODO implement later for filesystem image mutation");
        // hydra python script parses image into image stat format specificed above
        // then it is read into image struct to mutate
        Ok(())
    }*/
}
