use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Default, Copy, Clone, Zeroable, Pod)]
pub struct OctreeNode(pub u32);

impl OctreeNode {
    pub fn child_ptr(&self) -> u16 {
        ((self.0 & 0xfffe) >> 17) as u16
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

pub struct OctreeNodeBuilder {
    node: OctreeNode,
}

impl OctreeNodeBuilder {
    pub fn new() -> Self {
        Self {
            node: OctreeNode(0),
        }
    }

    pub fn child_ptr(mut self, value: u16) -> Self {
        self.node.0 |= (value as u32) << 17;
        self
    }

    pub fn far(mut self, value: bool) -> Self {
        self.node.0 |= (if value { 1 } else { 0 }) << 14;
        self
    }

    pub fn valid(mut self, value: u8) -> Self {
        self.node.0 |= (value as u32) << 8;
        self
    }

    pub fn leaf(mut self, value: u8) -> Self {
        self.node.0 |= value as u32;
        self
    }

    pub fn build(self) -> OctreeNode {
        self.node
    }
}
