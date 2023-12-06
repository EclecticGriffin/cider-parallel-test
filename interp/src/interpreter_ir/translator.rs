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
        let key = target.as_raw();
        if let Some(x) = self.cell_map.get(&key) {
            x.clone()
        } else {
            let v = arctex(Cell::from_ir(target, self));
            self.cell_map.insert(key, v.clone());
            v
        }
    }

    pub fn get_group(&mut self, target: &RRC<orig_ir::Group>) -> ArcTex<Group> {
        let key = target.as_raw();
        if let Some(x) = self.group_map.get(&key) {
            x.clone()
        } else {
            let v = arctex(Group::from_ir(target, self));
            self.group_map.insert(key, v.clone());
            v
        }
    }

    pub fn get_comb_group(
        &mut self,
        target: &RRC<orig_ir::CombGroup>,
    ) -> ArcTex<CombGroup> {
        let key = target.as_raw();
        if let Some(x) = self.comb_group_map.get(&key) {
            x.clone()
        } else {
            let v = arctex(CombGroup::from_ir(target, self));
            self.comb_group_map.insert(key, v.clone());
            v
        }
    }
}
