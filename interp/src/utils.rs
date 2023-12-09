use crate::interpreter_ir::*;
use crate::values::Value;
use calyx_ir::{Binding, Id, Nothing, RRC};
use parking_lot::{RwLock, RwLockReadGuard};
use serde::Deserialize;
use std::fs;
use std::hash::{Hash, Hasher};
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use std::rc::Rc;
use std::{cell::Ref, sync::Arc};
use std::{collections::HashMap, sync::Weak};

pub use crate::debugger::PrintCode;
/// A wrapper to enable hashing of assignments by their destination port.
pub(super) struct PortAssignment<'a>(*const Port, &'a Assignment<Nothing>);

impl<'a> Hash for PortAssignment<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<'a> PartialEq for PortAssignment<'a> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.0, other.0)
    }
}

impl<'a> Eq for PortAssignment<'a> {}

#[allow(dead_code)]
impl<'a> PortAssignment<'a> {
    /// Construct a new PortAssignment.
    pub fn new(a_ref: &'a Assignment<Nothing>) -> Self {
        Self(a_ref.dst.as_raw(), a_ref)
    }

    /// Get the associated port.
    pub fn get_port(&self) -> *const Port {
        self.0
    }

    /// Get the associated assignment.
    pub fn get_assignment(&self) -> &Assignment<Nothing> {
        self.1
    }
}

/// A map representing all the identifiers and its associated values in a
/// Futil program.
#[derive(Debug, Deserialize)]
#[serde(transparent)]
pub struct MemoryMap(HashMap<Id, Vec<Value>>);

impl MemoryMap {
    pub fn inflate_map(
        path: &Option<PathBuf>,
    ) -> crate::errors::InterpreterResult<Option<Self>> {
        if let Some(path) = path {
            let v = fs::read(path)?;
            let file_contents = std::str::from_utf8(&v)?;
            let map: MemoryMap = serde_json::from_str(file_contents).unwrap();
            return Ok(Some(map));
        }

        Ok(None)
    }
}

impl Deref for MemoryMap {
    type Target = HashMap<Id, Vec<Value>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for MemoryMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Construct memory bindings.
pub fn construct_bindings<const N: usize>(binds: [(&str, u64); N]) -> Binding {
    let mut vec = Binding::new();
    for (name, val) in binds {
        vec.push((Id::from(name), val))
    }
    vec
}

pub trait AsRaw<Target> {
    fn as_raw(&self) -> *const Target;
}

impl<T> AsRaw<T> for &T {
    fn as_raw(&self) -> *const T {
        *self as *const T
    }
}

impl<T> AsRaw<T> for *const T {
    fn as_raw(&self) -> *const T {
        *self
    }
}

impl<'a, T> AsRaw<T> for &Ref<'a, T> {
    fn as_raw(&self) -> *const T {
        self as &T as *const T
    }
}
impl<'a, T> AsRaw<T> for Ref<'a, T> {
    fn as_raw(&self) -> *const T {
        self as &T as *const T
    }
}

impl<T> AsRaw<T> for *mut T {
    fn as_raw(&self) -> *const T {
        *self as *const T
    }
}

impl<T> AsRaw<T> for RRC<T> {
    fn as_raw(&self) -> *const T {
        self.as_ptr()
    }
}

impl<T> AsRaw<T> for &RRC<T> {
    fn as_raw(&self) -> *const T {
        self.as_ptr()
    }
}

#[allow(dead_code)]
pub fn assignment_to_string(
    assignment: &calyx_ir::Assignment<Nothing>,
) -> String {
    let mut str = vec![];
    calyx_ir::Printer::write_assignment(assignment, 0, &mut str)
        .expect("Write Failed");
    String::from_utf8(str).expect("Found invalid UTF-8")
}

pub enum RcOrConst<T> {
    Rc(RRC<T>),
    Const(*const T),
}
#[allow(dead_code)]
impl<T> RcOrConst<T> {
    pub fn get_rrc(&self) -> Option<RRC<T>> {
        match self {
            RcOrConst::Rc(c) => Some(Rc::clone(c)),
            RcOrConst::Const(_) => None,
        }
    }
}

impl<T> From<RRC<T>> for RcOrConst<T> {
    fn from(input: RRC<T>) -> Self {
        Self::Rc(input)
    }
}

impl<T> From<&RRC<T>> for RcOrConst<T> {
    fn from(input: &RRC<T>) -> Self {
        Self::Rc(Rc::clone(input))
    }
}

impl<T> From<*const T> for RcOrConst<T> {
    fn from(input: *const T) -> Self {
        Self::Const(input)
    }
}

impl<T> AsRaw<T> for RcOrConst<T> {
    fn as_raw(&self) -> *const T {
        match self {
            RcOrConst::Rc(a) => a.as_raw(),
            RcOrConst::Const(a) => *a,
        }
    }
}

pub type ArcTex<T> = Arc<RwLock<T>>;
pub fn arctex<T>(input: T) -> ArcTex<T> {
    Arc::new(RwLock::new(input))
}

impl<T> AsRaw<T> for ArcTex<T> {
    fn as_raw(&self) -> *const T {
        self.data_ptr()
    }
}

impl<T> AsRaw<T> for &ArcTex<T> {
    fn as_raw(&self) -> *const T {
        self.data_ptr()
    }
}

#[derive(Debug)]
pub struct WeakArcTex<T>(pub(super) Weak<RwLock<T>>);

impl<T> Clone for WeakArcTex<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> WeakArcTex<T> {
    pub fn upgrade(&self) -> ArcTex<T> {
        // fail gracelessly
        self.0.upgrade().unwrap()
    }
}

impl<T> From<&ArcTex<T>> for WeakArcTex<T> {
    fn from(value: &ArcTex<T>) -> Self {
        Self(Arc::downgrade(value))
    }
}

impl<T> From<ArcTex<T>> for WeakArcTex<T> {
    fn from(value: ArcTex<T>) -> Self {
        Self(Arc::downgrade(&value))
    }
}

pub enum ArcTexOrConst<T> {
    Arc(ArcTex<T>),
    Const(*const T),
}

impl<T> AsRaw<T> for ArcTexOrConst<T> {
    fn as_raw(&self) -> *const T {
        match self {
            ArcTexOrConst::Arc(a) => a.as_raw(),
            ArcTexOrConst::Const(c) => *c,
        }
    }
}

impl<T> ArcTexOrConst<T> {
    #[must_use]
    pub fn as_arc(&self) -> Option<&ArcTex<T>> {
        if let Self::Arc(v) = self {
            Some(v)
        } else {
            None
        }
    }
}

impl<T> From<*const T> for ArcTexOrConst<T> {
    fn from(v: *const T) -> Self {
        Self::Const(v)
    }
}

impl<T> From<ArcTex<T>> for ArcTexOrConst<T> {
    fn from(v: ArcTex<T>) -> Self {
        Self::Arc(v)
    }
}

impl<'a, T> AsRaw<T> for RwLockReadGuard<'a, T> {
    fn as_raw(&self) -> *const T {
        &**self as *const T
    }
}

impl<'a, T> AsRaw<T> for &RwLockReadGuard<'a, T> {
    fn as_raw(&self) -> *const T {
        &***self as *const T
    }
}
