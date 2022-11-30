#![allow(dead_code)]
use hdrepresentation::*;
use randomutils::*;
mod randomutils;
//use nix::fcntl::OFlag;
use std::fs::write;
use std::path::Path;

#[derive(Clone)]
pub struct ProgramMutator {
    p: Program,
}

impl ProgramMutator {
    pub fn new() -> Self {
        let mut p = Program::new();
        p.prepare_buffers();
        Self { p }
    }

    pub fn get_fd_index(&self, fobj: &FileObject) -> i64 {
        for f in self.p.avail_files.iter() {
            if *f == *fobj {
                return f.fd_index;
            }
        }
        -1
    }

    pub fn add_random_syscall(&mut self) {
        match thread_rng().gen_range(0..19) {
            0 => {
                self.add_random_read();
            }
            1 => {
                self.add_random_write();
            }
            2 => {
                self.add_random_lseek();
            }
            3 => {
                self.add_random_pread();
            }
            4 => {
                self.add_random_pwrite();
            }
            5 => {
                self.add_random_getdents();
            }
            /*6 => { not yet implemented in penguincrab
                self.add_random_stat();
            }*/
            6 => {
                self.add_random_fstat();
            }
            /*8 => {
                self.add_random_lstat();
            }*/
            7 => {
                self.add_random_rename();
            }
            8 => {
                self.add_random_fdatasync();
            }
            9 => {
                self.add_random_fsync();
            }
            10 => {
                self.add_random_access();
            }
            11 => {
                self.add_random_truncate();
            }
            12 => {
                self.add_random_ftruncate();
            }
            13 => {
                self.add_random_mkdir();
            }
            14 => {
                self.add_random_rmdir();
            }
            15 => {
                self.add_random_link();
            }
            16 => {
                self.add_random_unlink();
            }
            17 => {
                self.add_random_symlink();
            }
            18 => {
                self.add_random_setxattr();
            }
            19 => {
                self.add_random_removexattr();
            }
            _ => {
                self.add_random_open();
            }
        }
    }
    // just adds syscall open with whatever arguments
    pub fn add_open(&mut self, path: &str, flags: i64, mode: i64) {
        // later o_directory
        let mut open_sys = Syscall::new(SysNo::Open);
        // if file already exists look for it in avail_files
        // TODO: go back to avail_dirs and avail_non_dirs instead of just avail_files

        let var_idx: i64 = match self.p.avail_files.iter().find(|fobj| fobj.rel_path == path) {
            None => {
                let idx = self.p.create_str(path);
                let obj = FileObject::new(&path, FileType::File, idx);
                self.p.add_file(obj, idx);
                idx
            }
            Some(f) => f.fd_index,
        };
        open_sys.add_arg(var_idx, true);
        open_sys.add_arg(flags, false);
        open_sys.add_arg(mode, false);
        // save returned fd
        let fd_index = self
            .p
            .create_file_variable(VariableType::Long(0), FileType::File);
        open_sys.ret_index = fd_index;
        self.p.add_syscall(open_sys);
    }

    // uses arguments that make sense
    pub fn add_random_open(&mut self) {
        // late change if does not exist use O_CREAT
        // if it does exist and is a directory: O_RDONLY | O_DIRECTORY
        let mut r = thread_rng();
        // create new file:
        // TODO: avail_non_dirs and avail_dirs
        if r.gen_range(0..11) < 9 || self.p.avail_files.len() == 0 {
            self.add_open(
                &self.rand_path(),
                (r.gen_range(0..0x2FF + 1) | 64) as i64, // 64 is LKL_O_CREAT
                0,                                       //r.gen_range(0..0o777 + 1),
            );
        } else {
            // open existing file
            self.add_open(
                &(self.get_random_filename()),
                r.gen_range(0..0x2FF + 1),
                r.gen_range(0..0o777 + 1),
            );
        }
    }

    /// returns tuple of directory names and file names
    pub fn get_file_names(&self) -> (Vec<String>, Vec<String>) {
        let mut dir_names = Vec::<String>::new();
        let mut file_names = Vec::<String>::new();
        for fobj in self.p.avail_files.iter() {
            let path = fobj.rel_path.clone();
            match fobj.ftype {
                FileType::Dir => {
                    dir_names.push(path.clone());
                }
                FileType::File | FileType::Symlink => {
                    file_names.push(path.clone());
                }
                _ => {}
            }
        }
        (dir_names, file_names)
    }

    pub fn get_random_dirname(&self) -> String {
        let (dir_names, _) = self.get_file_names();
        if dir_names.len() == 0 {
            return String::from("");
        }
        dir_names
            .choose(&mut rand::thread_rng())
            .expect("get_random_dirname failed to choose? check length of dir_names")
            .to_string()
    }

    pub fn get_random_filename(&self) -> String {
        let (_, file_names) = self.get_file_names();
        if file_names.len() == 0 {
            return String::from("");
        }
        file_names
            .choose(&mut rand::thread_rng())
            .expect("get_random_filename failed to choose? check length of file_names")
            .to_string()
    }

    /// generate a random path by either creating a new one or appending to a directory
    /// or getting a past file path
    pub fn rand_path(&self) -> String {
        let r = thread_rng().gen_range(0..10);
        let (file_names, dir_names) = self.get_file_names();
        let num_dirs = dir_names.len();
        let num_files = file_names.len();
        if (num_dirs == 0 && num_files > 0 && r < 3) || (r < 3 && num_files > 0) {
            self.get_random_filename()
        } else if (num_dirs > 0 && r > 8) || (num_dirs > 0 && num_files == 0 && r > 8) {
            let mut path = self.get_random_dirname();
            path.push_str("/");
            path.push_str(&rand_string(thread_rng().gen_range(1..20)));
            path
        } else {
            let mut x = String::from("/");
            x.push_str(&random_len_string());
            String::from(x)
        }
    }

    /// add read syscall to the program
    pub fn add_read(&mut self, fd_index: i64) {
        let mut read_sys = Syscall::new(SysNo::Read);
        read_sys.add_arg(fd_index, true);
        read_sys.add_arg(Program::SRC8192, true);
        read_sys.add_arg(rand_size(), false);
        let ret_index = self.p.create_variable(VariableType::Long(0));
        read_sys.ret_index = ret_index;
        self.p.add_syscall(read_sys);
    }

    // pub fn mutate_open
    // pub fn mutate_read
    pub fn add_random_read(&mut self) {
        let file_fd_count = self.p.active_file_fds.len();
        if file_fd_count < 1 {
            println!("No valid fds for random read so opening some files");
            // create, open, fcntl, dup and pipe.
            self.add_random_open();
        }
        let dir_fd_count = self.p.active_dir_fds.len();
        if dir_fd_count > 0 {
            if thread_rng().gen_range(0..2) == 1 {
                let idx = *self.get_random_dir_fd().unwrap();
                self.add_read(idx);
                return;
            }
        }
        let idx = *self
            .get_random_file_fd_index()
            .expect("random_read could not find random fd index");
        self.add_read(idx);
    }

    pub fn get_random_file_fd_index(&self) -> Option<&i64> {
        self.p.active_file_fds.choose(&mut rand::thread_rng())
    }

    pub fn get_random_dir_fd(&self) -> Option<&i64> {
        self.p.active_dir_fds.choose(&mut rand::thread_rng())
    }

    pub fn get_random_fd(&self) -> Option<&i64> {
        self.p.active_fds.choose(&mut rand::thread_rng())
    }

    pub fn add_write(&mut self, fd_index: i64) {
        let mut write_sys = Syscall::new(SysNo::Write);
        let sz = rand_size();
        write_sys.add_arg(fd_index, true);
        let var_idx = self.p.create_str(&rand_string(sz as usize));
        write_sys.add_arg(var_idx, true);
        write_sys.add_arg(sz, false);
        let ret_index = self.p.create_variable(VariableType::Long(0));
        write_sys.ret_index = ret_index;
        self.p.add_syscall(write_sys);
    }

    pub fn add_random_write(&mut self) {
        if self.p.active_fds.len() > 0 {
            self.add_write(*self.get_random_fd().unwrap());
        } else {
            println!("No valid fds for random write so opening some files");
            // create, open, fcntl, dup and pipe.
            self.add_random_open();
        }
    }

    pub fn add_lseek(&mut self, fd_index: i64) {
        let mut lseek_sys = Syscall::new(SysNo::Lseek);
        lseek_sys.add_arg(fd_index, true);
        lseek_sys.add_arg(
            thread_rng().gen_range(0..Program::PAGE_SIZE + 3) as i64 - 1,
            false,
        );
        // set 0, cur 1, end 2, data 3, hole 4
        lseek_sys.add_arg(thread_rng().gen_range(0..100) as i64 - 1, false);
        let ret_index = self.p.create_variable(VariableType::Long(0));
        lseek_sys.ret_index = ret_index;
        self.p.add_syscall(lseek_sys);
    }

    pub fn add_random_lseek(&mut self) {
        if self.p.active_fds.len() > 0 {
            self.add_lseek(*self.get_random_fd().unwrap());
        } else {
            println!("No valid fds for random lseek so opening some files");
            // create, open, fcntl, dup and pipe.
            self.add_random_open();
        }
    }

    pub fn add_random_getdents(&mut self) {
        let mut getdents_sys = Syscall::new(SysNo::Getdents);
        if self.p.active_fds.len() > 0 {
            getdents_sys.add_arg(*self.get_random_fd().unwrap(), true);
            getdents_sys.add_arg(Program::DEST8192, true);
            getdents_sys.add_arg(rand_size(), false);
            self.p.add_syscall(getdents_sys);
        } else {
            println!("No valid fds for random getdents so opening some files");
            self.add_random_open();
        }
    }

    pub fn add_random_pread(&mut self) {
        let mut pread_sys = Syscall::new(SysNo::Pread);
        if self.p.active_fds.len() > 0 {
            pread_sys.add_arg(*self.get_random_fd().unwrap(), true);
            pread_sys.add_arg(Program::SRC8192, true);
            pread_sys.add_arg(rand_size(), false);
            pread_sys.add_arg(rand_size(), false);
            self.p.add_syscall(pread_sys);
        } else {
            println!("No valid fds for random pread so opening some files");
            self.add_random_open();
        }
    }

    pub fn add_random_pwrite(&mut self) {
        let mut pwrite_sys = Syscall::new(SysNo::Pwrite);
        if self.p.active_fds.len() > 0 {
            pwrite_sys.add_arg(*self.get_random_fd().unwrap(), true);
            pwrite_sys.add_arg(Program::DEST8192, true);
            pwrite_sys.add_arg(rand_size(), false);
            pwrite_sys.add_arg(rand_size(), false);
            self.p.add_syscall(pwrite_sys);
        } else {
            println!("No valid fds for random pwrite so opening some files");
            self.add_random_open();
        }
    }

    pub fn add_random_fstat(&mut self) {
        let mut fstat_sys = Syscall::new(SysNo::Fstat);
        if self.p.active_fds.len() > 0 {
            fstat_sys.add_arg(*self.get_random_fd().unwrap(), true);
            fstat_sys.add_arg(Program::DEST8192, true);
            self.p.add_syscall(fstat_sys);
        } else {
            println!("No valid fds for random fstat so opening some files");
            self.add_random_open();
        }
    }

    pub fn get_random_filename_index(&self) -> i64 {
        let paths: Vec<i64> = self
            .p
            .avail_files
            .iter()
            .map(|fobj| fobj.fd_index)
            .clone()
            .collect();
        *paths
            .choose(&mut rand::thread_rng())
            .expect("get_random_filename_index does not have enough values in avail_files hashmap")
    }

    pub fn add_random_stat(&mut self) {
        let mut stat_sys = Syscall::new(SysNo::Stat);
        if self.p.avail_files.len() > 0 {
            stat_sys.add_arg(self.get_random_filename_index(), true);
            stat_sys.add_arg(Program::DEST8192, true);
            self.p.add_syscall(stat_sys);
        } else {
            eprintln!("No valid fds for random stat so opening some files");
            self.add_random_open();
        }
    }

    pub fn add_random_lstat(&mut self) {
        let mut lstat_sys = Syscall::new(SysNo::Lstat);
        if self.p.avail_files.len() > 0 {
            lstat_sys.add_arg(self.get_random_filename_index(), true);
            lstat_sys.add_arg(Program::DEST8192, true);
            self.p.add_syscall(lstat_sys);
        } else {
            println!("No valid fds for random stat so opening some files");
            self.add_random_open();
        }
    }

    pub fn add_random_rename(&mut self) {
        let mut rename_sys = Syscall::new(SysNo::Rename);
        while self.p.avail_files.len() < 2 {
            self.add_random_open();
        }
        // can be directory: deal with this later
        // get random file_obj
        let fobj = self.get_random_fobj();
        rename_sys.add_arg(self.get_fd_index(&fobj), true);
        self.p.remove_file(fobj.clone());
        // remove the past one and replace it
        match thread_rng().gen_range(0..6) {
            0 | 1 | 2 => {
                // set new file name to an existing one
                // get index for random fobj
                let replaced_fobj = self.get_random_fobj();
                let index_of_replaced = self.get_fd_index(&replaced_fobj);
                self.p.remove_file(replaced_fobj.clone());
                self.p.add_file(replaced_fobj, index_of_replaced);
                rename_sys.add_arg(index_of_replaced, true);
            }
            _ => {
                // generate new random path
                let path = self.rand_path();
                let var_idx = self.p.create_str(&path);
                rename_sys.add_arg(var_idx, true);
                self.p.add_file(fobj.clone(), var_idx);
            }
        }
        self.p.add_syscall(rename_sys);
    }

    pub fn get_random_fobj(&self) -> FileObject {
        let fobjs: Vec<FileObject> = self.p.avail_files.iter().cloned().collect();
        fobjs
            .choose(&mut rand::thread_rng())
            .expect("get_random_fobj does not have enough keys in avail_files hashmap")
            .clone()
    }

    pub fn get_random_fobj_with_xattrs(&self) -> Option<FileObject> {
        match self
            .p
            .avail_files
            .iter()
            .cloned()
            .filter(|x| x.xattrs.len() > 0)
            .collect::<Vec<FileObject>>()
            .choose(&mut rand::thread_rng())
        {
            None => None,
            Some(v) => Some(v.clone()),
        }
    }

    pub fn add_random_fsync(&mut self) {
        self.add_fsync(SysNo::Fsync);
    }

    pub fn add_random_fdatasync(&mut self) {
        self.add_fsync(SysNo::Fdatasync);
    }

    pub fn add_random_syncfs(&mut self) {
        self.add_fsync(SysNo::Syncfs);
    }

    pub fn add_fsync(&mut self, nr: SysNo) {
        let mut sys = Syscall::new(nr);
        while self.p.active_fds.len() < 2 {
            self.add_random_open();
        }
        sys.add_arg(
            *self
                .get_random_fd()
                .expect("not enough file descriptors for add_fsync"),
            true,
        );
        self.p.add_syscall(sys);
    }

    pub fn add_random_sendfile(&mut self) {
        let mut sys = Syscall::new(SysNo::Sendfile);
        while self.p.active_fds.len() < 2 {
            self.add_random_open();
        }
        sys.add_arg(
            *self
                .get_random_fd()
                .expect("not enough file descriptors for add_random_sendfile"),
            true,
        );
        sys.add_arg(
            *self
                .get_random_fd()
                .expect("not enough file descriptors for add_random_sendfile"),
            true,
        );
        // TODO: sendfile takes a pointer to an offset only sometimes should this be null
        sys.add_arg(rand_size(), false);
        sys.add_arg(rand_size(), false);
        self.p.add_syscall(sys);
    }

    pub fn add_random_access(&mut self) {
        let mut sys = Syscall::new(SysNo::Access);
        while self.p.avail_files.len() < 1 {
            self.add_random_open();
        }
        sys.add_arg(self.get_random_filename_index(), true);
        sys.add_arg(thread_rng().gen_range(0..10) as i64 - 1, false);
        self.p.add_syscall(sys);
    }

    pub fn add_random_ftruncate(&mut self) {
        while self.p.active_fds.len() < 1 {
            self.add_random_open();
        }
        let mut sys = Syscall::new(SysNo::Ftruncate);
        sys.add_arg(
            *self
                .get_random_file_fd_index()
                .expect("not enough random file fds for truncate"),
            true,
        );
        sys.add_arg(rand_size(), false);
        self.p.add_syscall(sys);
    }

    pub fn add_random_truncate(&mut self) {
        while self.p.avail_files.len() < 1 {
            self.add_random_open();
        }
        let mut sys = Syscall::new(SysNo::Truncate);
        sys.add_arg(self.get_random_filename_index(), true);
        sys.add_arg(rand_size(), false);
        self.p.add_syscall(sys);
    }

    pub fn add_random_mkdir(&mut self) {
        let mut sys = Syscall::new(SysNo::Mkdir);
        let path = self.rand_path();
        let var_idx = self.p.create_str(&path);
        sys.add_arg(var_idx, true);
        // S_ISVTX 512 or 0o777(is this correct?)
        sys.add_arg(thread_rng().gen_range(0..0o777 + 1), false);
        self.p.add_syscall(sys);
        let new_dir = FileObject::new(&path, FileType::Dir, var_idx);
        self.p.add_file(new_dir, var_idx);
    }

    pub fn add_random_rmdir(&mut self) {
        if self.p.avail_dirs.len() < 1 {
            self.add_random_mkdir();
        }
        let mut sys = Syscall::new(SysNo::Rmdir);
        // don't rm .
        // TODO: parse filesystem image for paths including . and ..
        let removed_dir = self
            .get_random_dir()
            .expect("random_rmdir has no directories to choose from");
        sys.add_arg(self.get_fd_index(&removed_dir), true);
        self.p.add_syscall(sys);
        self.p.remove_file(removed_dir);
    }

    pub fn get_random_dir(&mut self) -> Option<FileObject> {
        match self.p.avail_dirs.choose(&mut rand::thread_rng()) {
            None => None,
            Some(x) => Some(x.clone()),
        }
    }

    pub fn add_random_link(&mut self) {
        if self.p.avail_files.len() < 1 {
            eprintln!("link needs available files");
            self.add_random_open();
        }

        let mut sys = Syscall::new(SysNo::Link);
        let fobj = self.get_random_fobj();
        sys.add_arg(self.get_fd_index(&fobj), true);
        // later change to use existing paths as well but use random for now
        let path = self.rand_path();
        let var_idx = self.p.create_str(&path);
        sys.add_arg(var_idx, true);

        let mut new_copy = fobj.clone();
        // janus just copies and doesn't change rel_path?
        new_copy.rel_path = path;
        self.p.add_file(new_copy, var_idx);

        self.p.add_syscall(sys);
    }

    pub fn add_random_unlink(&mut self) {
        if self.p.avail_files.len() < 1 {
            eprintln!("unlink needs available files");
            self.add_random_open();
        }
        let mut sys = Syscall::new(SysNo::Unlink);
        let removed_file = self.get_random_fobj();
        sys.add_arg(self.get_fd_index(&removed_file), true);

        self.p.add_syscall(sys);

        self.p.remove_file(removed_file);
    }

    pub fn add_random_symlink(&mut self) {
        if self.p.avail_files.len() < 1 {
            eprintln!("symlink needs available files");
            self.add_random_open();
        }
        let mut sys = Syscall::new(SysNo::Symlink);
        let orig = self.get_random_fobj();
        sys.add_arg(self.get_fd_index(&orig), true);
        // later change to use existing paths as well but use random for now
        let path = self.rand_path();
        let var_idx = self.p.create_str(&path);
        sys.add_arg(var_idx, true);

        let new_copy = FileObject::new(&path, FileType::Symlink, var_idx);
        self.p.add_file(new_copy, var_idx);

        self.p.add_syscall(sys);
    }

    pub fn cprogram_to_file<P: AsRef<Path>>(&self, path: &mut P) -> std::io::Result<()> {
        write(path, format!("{}", self))
    }

    pub fn to_path<P: AsRef<Path>>(&self, path: P) -> std::io::Result<()> {
        self.p.to_path(path)
    }

    pub fn add_n_random_syscalls(&mut self, n: i32) {
        let mut i = 0;
        self.add_random_open();
        while i < (n - 1) {
            self.add_random_syscall();
            i += 1;
        }
    }

    pub fn get_random_xattr(&self, fobj: &FileObject) -> Option<Xattr> {
        match fobj.xattrs.choose(&mut rand::thread_rng()) {
            Some(v) => Some(v.clone()),
            None => None,
        }
    }

    pub fn get_random_xattr_index(&self, fobj: &FileObject) -> Option<i64> {
        match fobj.xattrs.choose(&mut rand::thread_rng()) {
            Some(v) => Some(v.2),
            None => None,
        }
    }

    pub fn add_random_setxattr(&mut self) {
        let mut sys = Syscall::new(SysNo::Setxattr);
        if self.p.avail_files.len() < 1{
            self.add_random_open();
        }
        let mut file = self.get_random_fobj();

        let idx = self.get_fd_index(&file);
        sys.add_arg(idx, true);
        // remove old file
        self.p.remove_file(file.clone());

        // create random value
        let value_sz = rand_size() % 40;
        let value = rand_xattr(value_sz as usize);

        // XATTR_CREATE = 1
        let mut flag = 1;
        let choice = thread_rng().gen_range(0..3) as i64;
        if file.xattrs.len() != 0 && choice > 0 {
            if choice == 1 {
                // replace old one XATTR_REPLACE = 2
                flag = 2;
            }
            // call create when really replacing
            let xattr = self
                .get_random_xattr(&file)
                .expect("no xattrs yet for setxattr");
            sys.add_arg(xattr.2, true);

            // remove old xattr
            file.xattrs.retain(|x: &Xattr| x.2 != xattr.2);
            // save to new name and value pair to file for later use
            file.xattrs.push(Xattr(xattr.0, value.clone(), xattr.2));
        } else {
            // make random name
            let name = rand_xattr((rand_size() % 40) as usize);
            let name_idx = self.p.create_str(&name);
            sys.add_arg(name_idx, true);
            // save to file for later use
            file.xattrs.push(Xattr(name, value.clone(), name_idx));
        }

        // add value
        sys.add_arg(self.p.create_str(&value), true);

        // add correct size or random size
        if (thread_rng().gen_range(0..10) as i64) < 8 {
            sys.add_arg(value_sz, false);
        } else {
            sys.add_arg(rand_size() - rand_size(), false);
        }

        sys.add_arg(flag, false);
        self.p.add_syscall(sys);

        // insert new file with changes
        self.p.add_file(file.clone(), idx);
    }

    pub fn add_random_removexattr(&mut self) {
        let mut fobj = match self.get_random_fobj_with_xattrs() {
            None => {
                eprintln!("add_random_removexattr could not find file with xattrs");
                return;
            }
            Some(x) => x,
        };

        let mut sys = Syscall::new(SysNo::Removexattr);

        let idx = self.get_fd_index(&fobj);
        sys.add_arg(idx, true);

        // remove old file
        self.p.remove_file(fobj.clone());

        let xattr = self.get_random_xattr(&fobj).unwrap();
        sys.add_arg(xattr.2, true);

        self.p.add_syscall(sys);
        // remove all xattr with same name
        fobj.xattrs.retain(|x| x.2 != xattr.2);

        // insert new file with changes
        self.p.add_file(fobj.clone(), idx);
    }

    pub fn add_random_listxattr(&mut self) {
        // get random file path
        let fobj = match self.get_random_fobj_with_xattrs() {
            Some(v) => v,
            None => {
                eprintln!("add_random_listxattr could not find file with xattrs");
                return;
            }
        };

        let mut sys = Syscall::new(SysNo::Listxattr);

        let idx = self.get_fd_index(&fobj);
        sys.add_arg(idx, true);

        sys.add_arg(Program::SRC8192, true);
        sys.add_arg((rand_size() % 40 as i64) - 1, false);
        self.p.add_syscall(sys);
    }

    pub fn add_random_getxattr(&mut self) {
        // get random file path
        let fobj = match self.get_random_fobj_with_xattrs() {
            Some(v) => v,
            None => {
                eprintln!("add_random_getxattr could not find file with xattrs");
                return;
            }
        };

        let mut sys = Syscall::new(SysNo::Getxattr);

        let idx = fobj.fd_index;
        sys.add_arg(idx, true);

        let xattr = self.get_random_xattr(&fobj).unwrap();
        sys.add_arg(xattr.2, true);
        sys.add_arg(Program::SRC8192, true);
        sys.add_arg((rand_size() % 40 as i64) - 1, false);
        self.p.add_syscall(sys);
    }
}

impl fmt::Display for ProgramMutator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.p)
    }
}

/*fn main() -> Result<(), std::io::Error> {
    // loop with return value not -11
    let mut p = ProgramMutator::new();
    p.add_n_random_syscalls(10);
    // serialize to filename
    let mut serialized_filename = String::from("serialized");
    serialized_filename.push_str(&rand_string(10));
    // serialize to filename
    p.to_path(&serialized_filename)?;
    // pass in filesystem image as argument
    // Command::new("").args().output().expect()
    //
    //pub fn main() -> Result<(), std::io::Error> {
    //    let args: Vec<String> = env::args().collect();
    //        if args.len() != 3 {
    //                eprintln!(
    //              "Usage: {} [deserialized program] [filesystem image]",
    //               &args[0]
    //              );
    //  return Err(Error::new(ErrorKind::Other,
    //      "invalid arguments"));
    //  }
    //  let f =
    //  Program::from_path(&args[1]);
    //      exec(&f,
    //      args[2].clone())?;}
    Ok(())
serde_with = "1.5.1"
}*/
