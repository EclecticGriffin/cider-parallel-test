use calyx_ir::{self as orig_ir, Attributes, Control as CalyxControl, RRC};
use calyx_utils::Id;

use std::sync::Arc;

// These IR constructs are unchanged but are here re-exported for consistency
pub use calyx_ir::Empty;

use crate::utils::{arctex, ArcTex};

use super::{translator::TranslationMap, Cell, CombGroup, Group, Port};

/// Data for the `enable` control statement.
#[derive(Debug)]
pub struct Enable {
    /// List of components to run.
    pub group: ArcTex<Group>,
    /// Attributes attached to this control statement.
    pub attributes: Attributes,
}

impl Enable {
    pub(crate) fn from_ir(
        original: &orig_ir::Enable,
        translator: &mut TranslationMap,
    ) -> Self {
        Self {
            group: translator.get_group(&original.group),
            attributes: original.attributes.clone(),
        }
    }
}

/// Data for the `seq` control statement.
#[derive(Debug)]
pub struct Seq {
    /// List of `Control` statements to run in sequence.
    pub stmts: Vec<Control>,
    /// Attributes attached to this control statement.
    pub attributes: Attributes,
}

impl Seq {
    pub(crate) fn from_ir(
        original: &orig_ir::Seq,
        translator: &mut TranslationMap,
    ) -> Self {
        Self {
            stmts: original
                .stmts
                .iter()
                .map(|x| Control::from_ir(x, translator))
                .collect(),
            attributes: original.attributes.clone(),
        }
    }
}

/// Data for the `par` control statement.
#[derive(Debug)]
pub struct Par {
    /// List of `Control` statements to run in parallel.
    pub stmts: Vec<Control>,
    /// Attributes attached to this control statement.
    pub attributes: Attributes,
}

impl Par {
    pub(crate) fn from_ir(
        original: &orig_ir::Par,
        translator: &mut TranslationMap,
    ) -> Self {
        Self {
            stmts: original
                .stmts
                .iter()
                .map(|x| Control::from_ir(x, translator))
                .collect(),
            attributes: original.attributes.clone(),
        }
    }
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

impl If {
    pub(crate) fn from_ir(
        original: &orig_ir::If,
        translator: &mut TranslationMap,
    ) -> Self {
        Self {
            port: translator.get_port(&original.port),
            cond: original
                .cond
                .as_ref()
                .map(|x| translator.get_comb_group(&x)),
            tbranch: Control::from_ir(&original.tbranch, translator),
            fbranch: Control::from_ir(&original.fbranch, translator),
            attributes: original.attributes.clone(),
        }
    }
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

impl While {
    pub(crate) fn from_ir(
        original: &orig_ir::While,
        translator: &mut TranslationMap,
    ) -> Self {
        Self {
            port: translator.get_port(&original.port),
            cond: original
                .cond
                .as_ref()
                .map(|x| translator.get_comb_group(&x)),
            body: Control::from_ir(&original.body, translator),
            attributes: original.attributes.clone(),
        }
    }
}

type PortMap = Vec<(Id, ArcTex<Port>)>;
type CellMap = Vec<(Id, ArcTex<Cell>)>;

/// Data for an `invoke` control statement.
#[derive(Debug)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize))]
pub struct Invoke {
    /// Cell that is being invoked.
    pub comp: ArcTex<Cell>,
    /// Mapping from name of input ports in `comp` to the port connected to it.
    pub inputs: PortMap,
    /// Mapping from name of output ports in `comp` to the port connected to it.
    pub outputs: PortMap,
    /// Attributes attached to this control statement.
    pub attributes: Attributes,
    /// Optional combinational group that is active when the invoke is active.
    pub comb_group: Option<ArcTex<CombGroup>>,
    /// Mapping from name of external cell in 'comp' to the cell connected to it.
    pub ref_cells: CellMap,
}

impl Invoke {
    pub(crate) fn from_ir(
        original: &orig_ir::Invoke,
        translator: &mut TranslationMap,
    ) -> Self {
        Self {
            comp: translator.get_cell(&original.comp),
            inputs: original
                .inputs
                .iter()
                .map(|(id, x)| (*id, translator.get_port(x)))
                .collect(),
            outputs: original
                .outputs
                .iter()
                .map(|(id, x)| (*id, translator.get_port(x)))
                .collect(),
            attributes: original.attributes.clone(),
            comb_group: original
                .comb_group
                .as_ref()
                .map(|x| translator.get_comb_group(x)),
            ref_cells: original
                .ref_cells
                .iter()
                .map(|(id, x)| (*id, translator.get_cell(x)))
                .collect(),
        }
    }
}

/// Control AST nodes.
#[derive(Debug, Clone)]
pub enum Control {
    /// Represents sequential composition of control statements.
    Seq(Arc<Seq>),
    /// Represents parallel composition of control statements.
    Par(Arc<Par>),
    /// Standard imperative if statement
    If(Arc<If>),
    /// Standard imperative while statement
    While(Arc<While>),
    /// Invoke a sub-component with the given port assignments
    Invoke(Arc<Invoke>),
    /// Runs the control for a list of subcomponents.
    Enable(Arc<Enable>),
    /// Control statement that does nothing.
    Empty(Arc<Empty>),
}

impl Control {
    pub(crate) fn from_ir(
        cc: &CalyxControl,
        translator: &mut TranslationMap,
    ) -> Self {
        match cc {
            CalyxControl::Seq(s) => {
                Control::Seq(Seq::from_ir(s, translator).into())
            }
            CalyxControl::Par(p) => {
                Control::Par(Par::from_ir(p, translator).into())
            }
            CalyxControl::If(i) => {
                Control::If(If::from_ir(i, translator).into())
            }
            CalyxControl::While(wh) => {
                Control::While(While::from_ir(wh, translator).into())
            }
            CalyxControl::Invoke(invoke) => {
                Control::Invoke(Invoke::from_ir(invoke, translator).into())
            }
            CalyxControl::Enable(enable) => {
                Control::Enable(Enable::from_ir(enable, translator).into())
            }
            CalyxControl::Static(_) => {
                todo!("interpreter does not yet support static")
            }
            CalyxControl::Repeat(_) => {
                todo!("interpreter does not yet support repeat")
            }
            CalyxControl::Empty(empty) => Control::Empty(empty.clone().into()),
        }
    }
}
