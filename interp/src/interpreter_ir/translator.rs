use super::structure::*;
use crate::utils::{arctex, ArcTex, AsRaw};
use ahash::HashMap;
use calyx_ir::{self as orig_ir, RRC};

pub(crate) struct TranslationMap {
    cell_map: HashMap<*const orig_ir::Cell, ArcTex<Cell>>,
    port_map: HashMap<*const orig_ir::Port, ArcTex<Port>>,
    group_map: HashMap<*const orig_ir::Group, ArcTex<Group>>,
    comb_group_map: HashMap<*const orig_ir::CombGroup, ArcTex<CombGroup>>,
}

impl TranslationMap {
    // pub fn get_cell(&mut self, target: &RRC<Cell>) -> ArcTex<Cell> {
    //     let key = target.as_raw();
    //     self.cell_map
    //         .entry(key)
    //         .or_insert_with(|| arctex((*target.borrow()).clone()))
    // }

    pub fn get_port(&mut self, target: &RRC<orig_ir::Port>) -> ArcTex<Port> {
        let key = target.as_raw();
        if let Some(x) = self.port_map.get(&key) {
            x.clone()
        } else {
            let v = arctex(Port::from_ir(target, self));
            self.port_map.insert(key, v.clone());
            v
        }
    }

    pub fn get_cell(&mut self, target: &RRC<orig_ir::Cell>) -> ArcTex<Cell> {
        todo!()
    }

    pub fn get_group(&mut self, target: &RRC<orig_ir::Group>) -> ArcTex<Group> {
        todo!()
    }

    pub fn get_comb_group(
        &mut self,
        target: &RRC<orig_ir::CombGroup>,
    ) -> ArcTex<CombGroup> {
        todo!()
    }
}
