use calyx_ir::Control as CalyxControl;
use calyx_ir::{self as ir, Attributes, CombGroup, Port, RRC};

use std::rc::Rc;

// These IR constructs are unchanged but are here re-exported for consistency
pub use calyx_ir::{Empty, Enable, Invoke};

use crate::utils::{arctex, ArcTex};

/// Data for the `seq` control statement.
#[derive(Debug)]
pub struct Seq {
    /// List of `Control` statements to run in sequence.
    pub stmts: Vec<Control>,
    /// Attributes attached to this control statement.
    pub attributes: Attributes,
}

/// Data for the `par` control statement.
#[derive(Debug)]
pub struct Par {
    /// List of `Control` statements to run in parallel.
    pub stmts: Vec<Control>,
    /// Attributes attached to this control statement.
    pub attributes: Attributes,
}

/// Data for the `if` control statement.
#[derive(Debug)]
pub struct If {
    /// Port that connects the conditional check.
    pub port: ArcTex<Port>,
    /// Optional combinational group attached using `with`.
    pub cond: Option<ArcTex<CombGroup>>,
    /// Control for the true branch.
    pub tbranch: Control,
    /// Control for the true branch.
    pub fbranch: Control,
    /// Attributes attached to this control statement.
    pub attributes: Attributes,
}

/// Data for the `if` control statement.
#[derive(Debug)]
pub struct While {
    /// Port that connects the conditional check.
    pub port: ArcTex<Port>,
    /// Group that makes the signal on the conditional port valid.
    pub cond: Option<ArcTex<CombGroup>>,
    /// Control for the loop body.
    pub body: Control,
    /// Attributes attached to this control statement.
    pub attributes: Attributes,
}

/// Control AST nodes.
#[derive(Debug, Clone)]
pub enum Control {
    /// Represents sequential composition of control statements.
    Seq(ArcTex<Seq>),
    /// Represents parallel composition of control statements.
    Par(ArcTex<Par>),
    /// Standard imperative if statement
    If(ArcTex<If>),
    /// Standard imperative while statement
    While(ArcTex<While>),
    /// Invoke a sub-component with the given port assignments
    Invoke(ArcTex<Invoke>),
    /// Runs the control for a list of subcomponents.
    Enable(ArcTex<Enable>),
    /// Control statement that does nothing.
    Empty(ArcTex<Empty>),
}

impl From<CalyxControl> for Control {
    fn from(cc: CalyxControl) -> Self {
        match cc {
            CalyxControl::Seq(s) => Control::Seq(arctex(s.into())),
            CalyxControl::Par(p) => Control::Par(arctex(p.into())),
            CalyxControl::If(i) => Control::If(arctex(i.into())),
            CalyxControl::While(wh) => Control::While(arctex(wh.into())),
            CalyxControl::Invoke(invoke) => Control::Invoke(arctex(invoke)),
            CalyxControl::Enable(enable) => Control::Enable(arctex(enable)),
            CalyxControl::Static(_) => {
                todo!("interpreter does not yet support static")
            }
            CalyxControl::Repeat(_) => {
                todo!("interpreter does not yet support repeat")
            }
            CalyxControl::Empty(empty) => Control::Empty(arctex(empty)),
        }
    }
}

impl From<ir::Seq> for Seq {
    fn from(seq: ir::Seq) -> Self {
        Self {
            stmts: seq.stmts.into_iter().map(|x| x.into()).collect(),
            attributes: seq.attributes,
        }
    }
}

impl From<ir::Par> for Par {
    fn from(par: ir::Par) -> Self {
        Self {
            stmts: par.stmts.into_iter().map(|x| x.into()).collect(),
            attributes: par.attributes,
        }
    }
}

impl From<ir::If> for If {
    fn from(i: ir::If) -> Self {
        Self {
            port: arctex(i.port.borrow().clone()),
            cond: i.cond.map(|c| arctex(c.borrow().clone())),
            tbranch: (*i.tbranch).into(),
            fbranch: (*i.fbranch).into(),
            attributes: i.attributes,
        }
    }
}

impl From<ir::While> for While {
    fn from(wh: ir::While) -> Self {
        Self {
            port: arctex(wh.port.borrow().clone()),
            cond: wh.cond.map(|c| arctex(c.borrow().clone())),
            body: (*wh.body).into(),
            attributes: wh.attributes,
        }
    }
}
