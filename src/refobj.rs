use super::*;
use crate::block::*;
use std::any::Any;
use std::collections::HashMap;
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

struct WritingContext(HashMap<*const (), u32>);

impl WritingContext {
    fn ptr2key(&self, ptr: *const ()) -> Result<u32> {
        self.0.get(&ptr).map(|key| *key).ok_or(Error::ObjNotFound)
    }
    fn rc<T>(&self, x: &Rc<T>) -> Result<u32> {
        use std::ops::Deref;
        self.ptr2key(x.deref() as *const _ as *const ())
    }
    fn emit(&mut self, x: *const ()) -> Option<u32> {
        use std::collections::hash_map::Entry;
        let id = self.0.len() as u32;
        match self.0.entry(x) {
            Entry::Occupied(_) => None,
            Entry::Vacant(v) => {
                v.insert(id);
                Some(id)
            }
        }
    }
}

trait WriteRef: WriteBlock {
    fn refobj(&mut self, ptr: *const ()) -> Result<()>;
}

trait ReadRef: ReadBlock {
    fn refobj(&mut self) -> Result<RefObj>;
}

struct Writer<T> {
    write: T,
    con: WritingContext,
}

impl<T: WritePrimitive> WritePrimitive for Writer<T> {
    fn u8(&mut self, x: u8) -> Result<()> {
        self.write.u8(x)
    }
    fn u16(&mut self, x: u16) -> Result<()> {
        self.write.u16(x)
    }
    fn u32(&mut self, x: u32) -> Result<()> {
        self.write.u32(x)
    }
    fn str(&mut self, x: &str) -> Result<()> {
        self.write.str(x)
    }
}

impl<T: WriteBlock> WriteBlock for Writer<T> {
    fn begin(&mut self) -> Result<()> {
        self.write.begin()
    }
    fn end(&mut self) -> Result<()> {
        self.write.end()
    }
    fn primitive(&mut self, x: Primitive) -> Result<()> {
        self.write.primitive(x)
    }
}

impl<T: WriteBlock> WriteRef for Writer<T> {
    fn refobj(&mut self, ptr: *const ()) -> Result<()> {
        let key = self.con.ptr2key(ptr)?;
        self.write.u32(key)
    }
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
        self.refobj(x.deref() as *const _ as *const ())
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
        let obj = self.refobj()?;
        obj.as_rc::<T>().ok_or(Error::ObjNotFound)
    }
}

pub trait DynSerialize {
    fn serialize(&self, id: u32, write: &mut dyn WriteRef) -> Result<()>;
}

pub trait SerializationNode: DynSerialize {
    fn get_dependencies(&self) -> &[&dyn SerializationNode];
}

impl<T> DynSerialize for T
where
    T: Serialize + TypeKey,
{
    fn serialize(&self, id: u32, write: &mut dyn WriteRef) -> Result<()>
    where
        Self: Serialize + TypeKey + Sized,
    {
        write.u32(id)?;
        write.str(Self::TYPE_KEY)?;
        write.obj(self)
    }
}

fn write_object<T: WriteBlock>(write: &mut Writer<T>, obj: &dyn SerializationNode) -> Result<()> {
    if let Some(id) = write.con.emit(obj as *const _ as *const ()) {
        for x in obj.get_dependencies() {
            write_object(write, *x)?;
        }
        obj.serialize(id, write)?;
    }
    Ok(())
}

fn write_object_dag<T: WriteBlock>(write: T, obj: &dyn SerializationNode) -> Result<()> {
    let mut writer = Writer {
        write,
        con: WritingContext(HashMap::new()),
    };
    write_object(&mut writer, obj)
}
