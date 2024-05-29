
pub trait ByteSize {
    fn byte_size(&self) -> usize;
}

impl ByteSize for Vec<u8> {
    fn byte_size(&self) -> usize {
        self.len()
    }
}