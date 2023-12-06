use calyx_frontend::{Attributes, Direction};
use calyx_ir::{self as orig_ir, CellType, Nothing, PortComp, RRC};
use calyx_utils::Id;
use smallvec::SmallVec;

use crate::utils::{ArcTex, WeakArcTex};

use super::translator::TranslationMap;

/// Ports can come from Cells or Groups
#[derive(Debug, Clone)]
pub enum PortParent {
    Cell(WeakArcTex<Cell>),
    Group(WeakArcTex<Group>),
}

impl From<WeakArcTex<Group>> for PortParent {
    fn from(v: WeakArcTex<Group>) -> Self {
        Self::Group(v)
    }
}

impl From<WeakArcTex<Cell>> for PortParent {
    fn from(v: WeakArcTex<Cell>) -> Self {
        Self::Cell(v)
    }
}

/// Represents a port on a cell.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize))]
pub struct Port {
    /// Name of the port
    pub name: Id,
    /// Width of the port
    pub width: u64,
    /// Direction of the port
    pub direction: Direction,
    /// Weak pointer to this port's parent
    pub parent: PortParent,
    /// Attributes associated with this port.
    pub attributes: Attributes,
}

impl Port {
    pub(crate) fn from_ir(
        original: &RRC<orig_ir::Port>,
        translator: &mut TranslationMap,
    ) -> Self {
        let orig = original.borrow();
        let new_parent: PortParent = match &orig.parent {
            orig_ir::PortParent::Cell(c) => {
                let c = c.upgrade();
                let cell_ref = WeakArcTex::from(translator.get_cell(&c));
                cell_ref.into()
            }
            orig_ir::PortParent::Group(g) => {
                let g = g.upgrade();
                let group_ref = WeakArcTex::from(translator.get_group(&g));
                group_ref.into()
            }
            orig_ir::PortParent::StaticGroup(_) => unimplemented!(
                "interpreter does not currently support static groups"
            ),
        };

        Self {
            name: orig.name,
            width: orig.width,
            direction: orig.direction.clone(),
            parent: new_parent,
            attributes: orig.attributes.clone(),
        }
    }
}

/// A Group of assignments that perform a logical action.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize))]
pub struct Group {
    /// Name of this group
    name: Id,

    /// The assignments used in this group
    pub assignments: Vec<Assignment<Nothing>>,

    /// Holes for this group
    pub holes: SmallVec<[ArcTex<Port>; 3]>,

    /// Attributes for this group.
    pub attributes: Attributes,
}

/// Represents a guarded assignment in the program
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize))]
pub struct Assignment<T> {
    /// The destination for the assignment.
    pub dst: ArcTex<Port>,

    /// The source for the assignment.
    pub src: ArcTex<Port>,

    /// The guard for this assignment.
    pub guard: Box<Guard<T>>,

    /// Attributes for this assignment.
    pub attributes: Attributes,
}

/// Represents an instantiated cell.
#[derive(Debug)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize))]
pub struct Cell {
    /// Name of this cell.
    name: Id,
    /// Ports on this cell
    pub ports: SmallVec<[ArcTex<Port>; 10]>,
    /// Underlying type for this cell
    pub prototype: CellType,
    /// Attributes for this group.
    pub attributes: Attributes,
    /// Whether the cell is external
    reference: bool,
}

/// An assignment guard which has pointers to the various ports from which it reads.
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize))]
pub enum Guard<T> {
    /// Represents `c1 || c2`.
    Or(Box<Guard<T>>, Box<Guard<T>>),
    /// Represents `c1 && c2`.
    And(Box<Guard<T>>, Box<Guard<T>>),
    /// Represents `!c1`
    Not(Box<Guard<T>>),
    #[default]
    /// The constant true
    True,
    /// Comparison operator.
    CompOp(PortComp, ArcTex<Port>, ArcTex<Port>),
    /// Uses the value on a port as the condition. Same as `p1 == true`
    Port(ArcTex<Port>),
    /// Other types of information.
    Info(T),
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize))]
pub struct CombGroup {
    /// Name of this group
    pub(super) name: Id,

    /// The assignments used in this group
    pub assignments: Vec<Assignment<Nothing>>,

    /// Attributes for this group.
    pub attributes: Attributes,
}
