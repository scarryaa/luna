use crate::renderer::primatives::{CircleInstance, LineInstance, RectInstance};

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct PrimId(u32);

pub enum NodePrim {
    Rect { id: PrimId, data: RectInstance },
    Line { id: PrimId, data: LineInstance },
    Circ { id: PrimId, data: CircleInstance },
}
