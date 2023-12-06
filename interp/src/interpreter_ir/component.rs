use crate::utils::ArcTex;

use super::{
    control::Control, translator::TranslationMap, Assignment, Cell, CombGroup,
    Group,
};
use calyx_frontend::Attributes;
use calyx_ir::{Component as CalyxComponent, Nothing};

use calyx_utils::{GetName, Id};
use itertools::Itertools;
use linked_hash_map::LinkedHashMap;
use std::sync::Arc;

#[derive(Debug)]
pub struct Component {
    /// Name of the component.
    pub name: Id,
    /// The input/output signature of this component.
    pub signature: ArcTex<Cell>,
    /// The cells instantiated for this component.
    pub cells: IdListArcTex<Cell>,
    /// Groups of assignment wires.
    pub groups: IdListArcTex<Group>,
    /// Groups of assignment wires.
    pub comb_groups: IdListArcTex<CombGroup>,
    /// The set of "continuous assignments", i.e., assignments that are always
    /// active.
    pub continuous_assignments: Arc<Vec<Assignment<Nothing>>>,
    /// The control program for this component.
    pub control: Control,
    /// Attributes for this component
    pub attributes: Attributes,
}

impl Component {
    /// Return a reference to the cell with `name` if present.
    pub fn find_cell<S>(&self, name: S) -> Option<ArcTex<Cell>>
    where
        S: Clone + Into<Id>,
    {
        self.cells.find(name)
    }

    pub fn from_ir(
        cc: &CalyxComponent,
        translator: &mut TranslationMap,
    ) -> Self {
        Self {
            name: cc.name,
            signature: translator.get_cell(&cc.signature),
            cells: cc.cells.iter().map(|x| translator.get_cell(x)).into(),
            groups: cc.groups.iter().map(|x| translator.get_group(x)).into(),
            comb_groups: cc
                .comb_groups
                .iter()
                .map(|x| translator.get_comb_group(x))
                .into(),
            continuous_assignments: Arc::new(
                cc.continuous_assignments
                    .iter()
                    .map(|x| translator.get_assignment(x))
                    .collect_vec(),
            ),
            control: Control::from_ir(&cc.control.borrow(), translator),
            attributes: cc.attributes.clone(),
        }
    }
}

/// A straightforward copy of [calyx_ir::IdList] lifted to [ArcTex] insides
#[derive(Debug)]
pub struct IdListArcTex<T: GetName>(LinkedHashMap<Id, ArcTex<T>>);

impl<T: GetName> IdListArcTex<T> {
    /// Returns the element indicated by the name, if present, otherwise None.
    pub fn find<S>(&self, name: S) -> Option<ArcTex<T>>
    where
        S: Into<Id>,
    {
        self.0.get(&name.into()).map(Arc::clone)
    }

    /// Returns an iterator over immutable references
    pub fn iter(&self) -> impl Clone + Iterator<Item = &ArcTex<T>> {
        self.0.values()
    }
}

impl<T, F> From<F> for IdListArcTex<T>
where
    T: GetName,
    F: IntoIterator<Item = ArcTex<T>>,
{
    fn from(list: F) -> Self {
        IdListArcTex(
            list.into_iter()
                .map(|item| {
                    let name = item.read().name();
                    (name, item)
                })
                .collect::<LinkedHashMap<Id, ArcTex<T>>>(),
        )
    }
}
