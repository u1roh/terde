use super::*;
use crate::block::*;
use std::any::Any;
use std::rc::Rc;

pub struct RefObj(Box<dyn Any>);
impl RefObj {
    fn new<T: 'static>(x: T) -> Self {
        Self(Box::new(Rc::new(x)))
    }
    fn as_rc<T: 'static>(&self) -> Option<Rc<T>> {
        self.0.downcast_ref::<Rc<T>>().map(|rc| rc.clone())
    }
}
pub trait WriteRef: WriteBlock {
    fn refobj(&mut self, ptr: *const ()) -> Result<()>;
}

pub trait ReadRef: ReadBlock {
    fn refobj(&mut self) -> Result<RefObj>;
}

impl<W: WriteRef> Write for W {
    fn obj<S: Serialize>(&mut self, x: &S) -> Result<()> {
        self.begin()?;
        self.u16(S::VERSION)?;
        x.serialize(self)?;
        self.end()
    }
    fn rc<T>(&mut self, x: &Rc<T>) -> Result<()> {
        use std::ops::Deref;
        self.refobj(x.deref() as *const _ as *const ())
    }
}

impl<R: ReadRef> Read for R {
    fn obj<T: Deserialize>(&mut self) -> Result<T> {
        self.begin()?;
        let version = self.u16()?;
        let obj = T::deserialize(self, version)?;
        self.end()?;
        Ok(obj)
    }
    fn rc<T: 'static>(&mut self) -> Result<Rc<T>> {
        let obj = self.refobj()?;
        obj.as_rc::<T>().ok_or(Error::ObjNotFound)
    }
}
