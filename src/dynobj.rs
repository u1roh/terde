use super::*;
use crate::tag::*;
use std::any::Any;
use std::rc::Rc;

#[derive(Debug)]
pub struct DynObj(Box<dyn Any>);
impl DynObj {
    pub fn new<T: 'static>(x: T) -> Self {
        Self(Box::new(Rc::new(x)))
    }
    pub fn as_rc<T: 'static>(&self) -> Option<Rc<T>> {
        self.0.downcast_ref::<Rc<T>>().cloned()
    }
}

pub trait DynSerialize {
    fn type_key(&self) -> u128;
    fn version(&self) -> u16;
    fn serialize(&self, write: &mut dyn DynWrite) -> Result<()>;
}

pub trait DynWrite: TagWrite {
    fn ptr(&mut self, ptr: *const ()) -> Result<()>;
}

pub trait DynRead: TagRead {
    fn ptr(&mut self) -> Result<&DynObj>;
}

impl<'a> Write for dyn DynWrite + 'a {
    fn rc<T>(&mut self, x: &Rc<T>) -> Result<()> {
        use std::ops::Deref;
        self.ptr(x.deref() as *const _ as *const ())
    }
}

impl<'a> Read for dyn DynRead + 'a {
    fn rc<T: 'static>(&mut self) -> Result<Rc<T>> {
        let obj = self.ptr()?;
        obj.as_rc::<T>().ok_or(Error::ObjNotFound)
    }
}

impl<T> DynSerialize for T
where
    T: Serialize + TypeKey,
{
    fn type_key(&self) -> u128 {
        Self::TYPE_KEY
    }
    fn version(&self) -> u16 {
        Self::VERSION
    }
    fn serialize(&self, write: &mut dyn DynWrite) -> Result<()> {
        self.serialize(write)
    }
}
