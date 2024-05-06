#[repr(C)]
pub struct MotifHints {
    pub flags: u32,
    pub functions: u32,
    pub decorations: u32,
    pub input_mode: u32,
    pub status: u32,
}

impl MotifHints {
    pub fn no_decorations() -> Self {
        Self {
            flags: 0x2,
            functions: 0,
            decorations: 0,
            input_mode: 0,
            status: 0,
        }
    }

    pub fn default_decorations() -> Self {
        Self {
            flags: 0x3,
            functions: 0x3e,
            decorations: 0x7e,
            input_mode: 0,
            status: 0,
        }
    }

    pub fn to_prop_data(&self) -> [u32; 5] {
        [
            self.flags,
            self.functions,
            self.decorations,
            self.input_mode as u32,
            self.status,
        ]
    }
}
