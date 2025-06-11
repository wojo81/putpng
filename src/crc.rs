pub struct Crc32 {
    table: [u32; 256],
}

impl Crc32 {
    pub fn new() -> Self {
        Self {
            table: std::array::from_fn(|i| {
                let mut e = i as u32;
                for _ in 0..8 {
                    if e & 1 == 1 {
                        e = 0xedb88320 ^ ((e >> 1) & 0x7fffffff);
                    } else {
                        e = (e >> 1) & 0x7fffffff;
                    }
                }
                e
            }),
        }
    }

    pub fn calculate(&self, bytes: &[u8]) -> u32 {
        let mut answer = u32::MAX;
        bytes.iter().for_each(|byte| {
            answer =
                self.table[(answer as usize ^ *byte as usize) & 0xff] ^ ((answer >> 8) & 0xffffff)
        });
        !answer
    }
}
