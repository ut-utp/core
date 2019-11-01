use std::io::Error as IoError;

#[derive(Debug)]
pub enum MemoryShimError {
    IoError(IoError),
    IncorrectlySizedFile(u64),
}

impl From<IoError> for MemoryShimError {
    fn from(err: IoError) -> Self {
        Self::IoError(err)
    }
}
