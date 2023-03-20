use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Default, Copy, Clone, Zeroable, Pod)]
pub struct OctreeNode(pub u32);

impl OctreeNode {
    pub fn new(child_ptr: u16, far: bool, valid: u8, leaf: u8) -> Self {
        Self(
            (child_ptr as u32) << 17 |
                if far { 1u32 } else { 0u32 } |
                (valid as u32) << 8 |
                leaf as u32
        )
    }

    pub fn child_ptr(&self) -> u16 {
        ((self.0 & 0xfffe0000) >> 17) as u16
    }

    pub fn far(&self) -> bool {
        (self.0 & 0x00010000) != 0
    }

    pub fn valid(&self, index: usize) -> bool {
        (self.0 & (1 << (index + 8))) != 0
    }

    pub fn leaf(&self, index: usize) -> bool {
        (self.0 & (1 << index)) != 0
    }
}
