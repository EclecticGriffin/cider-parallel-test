use crate::{
    interpreter_ir::{Assignment, Cell, Control, Group, Port, PortParent},
    utils::ArcTex,
    values::Value,
};
use calyx_ir as orig_ir;
use std::cell::Ref;
use std::collections::HashSet;
use std::ops::Deref;
pub type ConstPort = *const Port;
pub type ConstCell = *const Cell;

#[inline]
pub fn get_done_port(group: &Group) -> ArcTex<Port> {
    group.get("done")
}

#[inline]
pub fn get_go_port(group: &Group) -> ArcTex<Port> {
    group.get("go")
}

#[inline]
pub fn is_signal_high(done: &Value) -> bool {
    done.as_bool()
}

pub fn get_dest_cells<'a, I>(
    iter: I,
    done_sig: Option<ArcTex<Port>>,
) -> Vec<ArcTex<Cell>>
where
    I: Iterator<Item = &'a Assignment<orig_ir::Nothing>>,
{
    let mut assign_set: HashSet<*const Cell> = HashSet::new();
    let mut output_vec = vec![];

    if let Some(done_prt) = done_sig {
        if let PortParent::Cell(c) = &done_prt.read().parent {
            let parent = c.upgrade();
            assign_set.insert(parent.data_ptr());
            output_vec.push(parent)
        }
    };

    let iterator = iter.filter_map(|assign| {
        match &assign.dst.read().parent {
            PortParent::Cell(c) => {
                match &c.upgrade().read().prototype {
                    orig_ir::CellType::Primitive { .. }
                    | orig_ir::CellType::Constant { .. }
                    | orig_ir::CellType::Component { .. } => {
                        let const_cell: *const Cell = c.upgrade().data_ptr();
                        if assign_set.contains(&const_cell) {
                            None //b/c we don't want duplicates
                        } else {
                            assign_set.insert(const_cell);
                            Some(c.upgrade())
                        }
                    }

                    orig_ir::CellType::ThisComponent => None,
                }
            }
            PortParent::Group(_) => None,
        }
    });
    output_vec.extend(iterator);

    output_vec
}
pub fn control_is_empty(control: &Control) -> bool {
    match control {
        Control::Seq(s) => s.stmts.iter().all(control_is_empty),
        Control::Par(p) => p.stmts.iter().all(control_is_empty),
        Control::If(_) => false,
        Control::While(_) => false,
        Control::Invoke(_) => false,
        Control::Enable(_) => false,
        Control::Empty(_) => true,
    }
}

pub enum ReferenceHolder<'a, T> {
    Ref(Ref<'a, T>),
    Borrow(&'a T),
}

impl<'a, T> From<&'a T> for ReferenceHolder<'a, T> {
    fn from(input: &'a T) -> Self {
        Self::Borrow(input)
    }
}

impl<'a, T> From<Ref<'a, T>> for ReferenceHolder<'a, T> {
    fn from(input: Ref<'a, T>) -> Self {
        Self::Ref(input)
    }
}

impl<'a, T> Deref for ReferenceHolder<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            ReferenceHolder::Ref(r) => r,
            ReferenceHolder::Borrow(b) => b,
        }
    }
}
