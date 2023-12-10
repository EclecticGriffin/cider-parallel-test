use std::sync::Arc;

use calyx_frontend::{Attribute, Attributes, Direction};
use calyx_ir::{self as orig_ir, CellType, Nothing, PortComp, RRC};

use calyx_utils::{GetName, Id};
use itertools::Itertools;
use orig_ir::Canonical;
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

    /// Get the canonical representation for this Port.
    pub fn canonical(&self) -> Canonical {
        Canonical(self.get_parent_name(), self.name)
    }
    /// Gets name of parent object.
    pub fn get_parent_name(&self) -> Id {
        match &self.parent {
            PortParent::Cell(cell) => cell.upgrade().read().name,
            PortParent::Group(group) => group.upgrade().read().name,
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

impl Group {
    pub(crate) fn from_ir_partial(
        original: &RRC<orig_ir::Group>,
        _translator: &mut TranslationMap,
    ) -> Self {
        let orig = original.borrow();

        Self {
            name: orig.name(),
            assignments: vec![],
            holes: Default::default(),
            attributes: orig.attributes.clone(),
        }
    }

    /// Get a reference to the named hole if it exists.
    pub fn find<S>(&self, name: S) -> Option<ArcTex<Port>>
    where
        S: std::fmt::Display,
        Id: PartialEq<S>,
    {
        self.holes
            .iter()
            .find(|&g| g.read().name == name)
            .map(Arc::clone)
    }

    /// Get a reference to the named hole or panic.
    pub fn get<S>(&self, name: S) -> ArcTex<Port>
    where
        S: std::fmt::Display + Clone,
        Id: PartialEq<S>,
    {
        self.find(name.clone()).unwrap_or_else(|| {
            panic!("Hole `{name}' not found on group `{}'", self.name)
        })
    }

    pub fn name(&self) -> Id {
        self.name
    }
}

impl GetName for Group {
    fn name(&self) -> Id {
        self.name
    }
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

impl<T: Clone> Assignment<T> {
    pub(crate) fn from_ir(
        original: &orig_ir::Assignment<T>,
        translator: &mut TranslationMap,
    ) -> Self {
        Self {
            dst: translator.get_port(&original.dst),
            src: translator.get_port(&original.src),
            guard: Box::new(Guard::from_ir(&original.guard, translator)),
            attributes: original.attributes.clone(),
        }
    }
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
    _reference: bool,
}

impl GetName for Cell {
    fn name(&self) -> Id {
        self.name
    }
}

impl Cell {
    pub(crate) fn from_ir_partial(
        original: &RRC<orig_ir::Cell>,
        _translator: &mut TranslationMap,
    ) -> Self {
        let orig = original.borrow();

        Self {
            name: orig.name(),
            ports: Default::default(),
            prototype: orig.prototype.clone(),
            attributes: orig.attributes.clone(),
            _reference: orig.is_reference(),
        }
    }

    /// Returns a reference to all [super::Port] attached to this cells.
    pub fn ports(&self) -> &SmallVec<[ArcTex<Port>; 10]> {
        &self.ports
    }
    /// Get a reference to the named port if it exists.
    pub fn find<S>(&self, name: S) -> Option<ArcTex<Port>>
    where
        S: std::fmt::Display + Clone,
        Id: PartialEq<S>,
    {
        self.ports
            .iter()
            .find(|&g| g.read().name == name)
            .map(Arc::clone)
    }

    /// Get a reference to the named port and throw an error if it doesn't
    /// exist.
    pub fn get<S>(&self, name: S) -> ArcTex<Port>
    where
        S: std::fmt::Display + Clone,
        Id: PartialEq<S>,
    {
        self.find(name.clone()).unwrap_or_else(|| {
            panic!(
                "Port `{name}' not found on cell `{}'. Known ports are: {}",
                self.name,
                self.ports
                    .iter()
                    .map(|p| p.read().name.to_string())
                    .join(",")
            )
        })
    }

    pub fn name(&self) -> Id {
        self.name
    }
    /// Get parameter binding from the prototype used to build this cell.
    pub fn get_parameter<S>(&self, param: S) -> Option<u64>
    where
        Id: PartialEq<S>,
    {
        match &self.prototype {
            CellType::Primitive { param_binding, .. } => param_binding
                .iter()
                .find(|(key, _)| *key == param)
                .map(|(_, val)| *val),
            CellType::Component { .. } => None,
            CellType::ThisComponent => None,
            CellType::Constant { .. } => None,
        }
    }

    /// Return the value associated with this attribute key.
    pub fn get_attribute<A: Into<Attribute>>(&self, attr: A) -> Option<u64> {
        self.attributes.get(attr.into())
    }

    /// Return all ports that have the attribute `attr`.
    pub fn find_all_with_attr<A>(
        &self,
        attr: A,
    ) -> impl Iterator<Item = ArcTex<Port>> + '_
    where
        A: Into<Attribute>,
    {
        let attr = attr.into();
        self.ports
            .iter()
            .filter(move |&p| p.read().attributes.has(attr))
            .map(Arc::clone)
    }

    /// Return the unique port with the given attribute.
    /// If multiple ports have the same attribute, then we panic.
    /// If there are not ports with the give attribute, then we return None.
    pub fn find_unique_with_attr<A>(
        &self,
        attr: A,
    ) -> calyx_utils::CalyxResult<Option<ArcTex<Port>>>
    where
        A: Into<Attribute>,
    {
        let attr = attr.into();
        let mut ports = self.find_all_with_attr(attr);
        if let Some(port) = ports.next() {
            if ports.next().is_some() {
                Err(calyx_utils::Error::malformed_structure(format!(
                    "Multiple ports with attribute `{}` found on cell `{}`",
                    attr, self.name
                )))
            } else {
                Ok(Some(port))
            }
        } else {
            Ok(None)
        }
    }

    /// Get the unique port with the given attribute.
    /// Panic if no port with the attribute is found and returns an error if multiple ports with the attribute are found.
    pub fn get_unique_with_attr<A>(
        &self,
        attr: A,
    ) -> calyx_utils::CalyxResult<ArcTex<Port>>
    where
        A: Into<Attribute> + std::fmt::Display + Copy,
    {
        Ok(self.find_unique_with_attr(attr)?.unwrap_or_else(|| {
            panic!(
                "Port with attribute `{attr}' not found on cell `{}'",
                self.name,
            )
        }))
    }
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

impl<T: Clone> Guard<T> {
    pub(crate) fn from_ir(
        original: &orig_ir::Guard<T>,
        translator: &mut TranslationMap,
    ) -> Self {
        match original {
            orig_ir::Guard::Or(l, r) => Guard::Or(
                Guard::from_ir(l, translator).into(),
                Guard::from_ir(r, translator).into(),
            ),
            orig_ir::Guard::And(l, r) => Guard::And(
                Guard::from_ir(l, translator).into(),
                Guard::from_ir(r, translator).into(),
            ),
            orig_ir::Guard::Not(n) => {
                Guard::Not(Guard::from_ir(n, translator).into())
            }
            orig_ir::Guard::True => Guard::True,
            orig_ir::Guard::CompOp(op, l, r) => Guard::CompOp(
                op.clone(),
                translator.get_port(l),
                translator.get_port(r),
            ),
            orig_ir::Guard::Port(p) => Guard::Port(translator.get_port(p)),
            orig_ir::Guard::Info(i) => Guard::Info(i.clone()),
        }
    }
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

impl CombGroup {
    pub(crate) fn from_ir(
        original: &RRC<orig_ir::CombGroup>,
        translator: &mut TranslationMap,
    ) -> Self {
        let orig = original.borrow();
        Self {
            name: orig.name(),
            assignments: orig
                .assignments
                .iter()
                .map(|x| Assignment::from_ir(x, translator))
                .collect(),
            attributes: orig.attributes.clone(),
        }
    }

    pub fn name(&self) -> Id {
        self.name
    }
}

impl GetName for CombGroup {
    fn name(&self) -> Id {
        self.name
    }
}
