use super::*;
use crate::tag::*;
use std::any::Any;
use std::rc::Rc;

pub struct RefObj(Box<dyn Any>);
impl RefObj {
    pub fn new<T: 'static>(x: T) -> Self {
        Self(Box::new(Rc::new(x)))
    }
    pub fn as_rc<T: 'static>(&self) -> Option<Rc<T>> {
        self.0.downcast_ref::<Rc<T>>().map(|rc| rc.clone())
    }
}

pub trait WriteRef: TagWrite {
    fn ptr(&mut self, ptr: *const ()) -> Result<()>;
}

pub trait ReadRef: TagRead {
    fn ptr(&mut self) -> Result<RefObj>;
}

impl<W: WriteRef + ?Sized> Write for W {
    fn obj<S: Serialize>(&mut self, x: &S) -> Result<()> {
        self.begin()?;
        self.u16(S::VERSION)?;
        x.serialize(self)?;
        self.end()
    }
    fn rc<T>(&mut self, x: &Rc<T>) -> Result<()> {
        use std::ops::Deref;
        self.ptr(x.deref() as *const _ as *const ())
    }
}

impl<R: ReadRef + ?Sized> Read for R {
    fn obj<T: Deserialize>(&mut self) -> Result<T> {
        self.begin()?;
        let version = self.u16()?;
        let obj = T::deserialize(self, version)?;
        self.end()?;
        Ok(obj)
    }
    fn rc<T: 'static>(&mut self) -> Result<Rc<T>> {
        let obj = self.ptr()?;
        obj.as_rc::<T>().ok_or(Error::ObjNotFound)
    }
}

impl<T> DynSerialize for T
where
    T: Serialize + TypeKey,
{
    fn type_key(&self) -> &'static str {
        Self::TYPE_KEY
    }
    fn serialize(&self, write: &mut dyn WriteRef) -> Result<()> {
        write.obj(self)
    }
}
