//! Fixed-capacity text buffer with `core::fmt::Write`, for log lines and telemetry packets
//! without an allocator. Writes past the end are truncated and reported as `fmt::Error`.

pub struct Buf<const N: usize> {
    buf: [u8; N],
    len: usize,
}

impl<const N: usize> Buf<N> {
    pub const fn new() -> Self {
        Self { buf: [0; N], len: 0 }
    }
    pub fn clear(&mut self) {
        self.len = 0;
    }
    pub fn as_bytes(&self) -> &[u8] {
        &self.buf[..self.len]
    }
    pub fn len(&self) -> usize {
        self.len
    }
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl<const N: usize> Default for Buf<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> core::fmt::Write for Buf<N> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let b = s.as_bytes();
        let n = b.len().min(N - self.len);
        self.buf[self.len..self.len + n].copy_from_slice(&b[..n]);
        self.len += n;
        if n < b.len() { Err(core::fmt::Error) } else { Ok(()) }
    }
}
