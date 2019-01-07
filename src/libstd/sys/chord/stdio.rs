use io;
use sys::{ReadSysCall};

pub struct Stdin;
pub struct Stdout;
pub struct Stderr;

extern "Rust" {
    #[allow(improper_ctypes)]
    fn ipc_print(data: &[u8]);
}

impl Stdin {
    pub fn new() -> io::Result<Stdin> {
        Ok(Stdin)
    }

    pub fn read(&self, data: &mut [u8]) -> io::Result<usize> {
        Ok(ReadSysCall::perform(0, data))
    }
}

impl Stdout {
    pub fn new() -> io::Result<Stdout> {
        Ok(Stdout)
    }

    pub fn write(&self, data: &[u8]) -> io::Result<usize> {
        unsafe { ipc_print(data); }
        Ok(data.len())
    }

    pub fn flush(&self) -> io::Result<()> {
        Ok(())
    }
}

impl Stderr {
    pub fn new() -> io::Result<Stderr> {
        Ok(Stderr)
    }

    pub fn write(&self, data: &[u8]) -> io::Result<usize> {
        unsafe { ipc_print(data); }
        Ok(data.len())
    }

    pub fn flush(&self) -> io::Result<()> {
        Ok(())
    }
}

impl io::Write for Stderr {
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        (&*self).write(data)
    }
    fn flush(&mut self) -> io::Result<()> {
        (&*self).flush()
    }
}

pub const STDIN_BUF_SIZE: usize = 0;

pub fn is_ebadf(_err: &io::Error) -> bool {
    true
}

pub fn panic_output() -> Option<impl io::Write> {
    Stderr::new().ok()
}
