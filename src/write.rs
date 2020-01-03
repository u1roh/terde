use super::*;
use std::collections::HashMap;

struct Writer<'a, W: std::io::Write> {
    write: W,
    con: &'a WritingContext,
}

impl<'a, W: std::io::Write> Writer<'a, W> {
    fn tag(&mut self, tag: Tag) -> Result<()> {
        self.write.u8(tag as u8)
    }
}

impl<'a, W: std::io::Write> WritePrimitive for Writer<'a, W> {
    fn u8(&mut self, x: u8) -> Result<()> {
        self.tag(Tag::U8)?;
        self.write.u8(x)
    }
    fn u16(&mut self, x: u16) -> Result<()> {
        self.tag(Tag::U16)?;
        self.write.u16(x)
    }
    fn u32(&mut self, x: u32) -> Result<()> {
        self.tag(Tag::I32)?;
        self.write.u32(x)
    }
    fn str(&mut self, _: &str) -> Result<()> {
        unimplemented!()
    }
}

impl<'a, W: std::io::Write> Write for Writer<'a, W> {
    fn obj<S: Serialize>(&mut self, x: &S) -> Result<()> {
        self.tag(Tag::OBJ)?;
        self.write.u16(S::VERSION)?;
        x.serialize(self)?;
        self.tag(Tag::END)?;
        Ok(())
    }
    fn rc<T>(&mut self, x: &Rc<T>) -> Result<()> {
        let key = self.con.rc(x).ok_or(Error::ObjNotFound)?;
        self.u32(key as u32)
    }
}

pub struct WritingContext(HashMap<*const (), u32>);

impl WritingContext {
    fn rc<T>(&self, x: &Rc<T>) -> Option<u32> {
        use std::ops::Deref;
        self.0
            .get(&(x.deref() as *const _ as *const ()))
            .map(|key| *key)
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

impl<T> DynSerialize for T
where
    T: Serialize + TypeKey,
{
    fn serialize(
        &self,
        id: u32,
        write: &mut dyn std::io::Write,
        con: &write::WritingContext,
    ) -> Result<()>
    where
        Self: Serialize + TypeKey + Sized,
    {
        let mut write = Writer { write, con };
        write.u32(id as u32)?;
        write.str(Self::TYPE_KEY)?;
        write.obj(self)
    }
}

fn write_object(
    write: &mut dyn std::io::Write,
    con: &mut WritingContext,
    obj: &dyn SerializationNode,
) -> Result<()> {
    if let Some(id) = con.emit(obj as *const _ as *const ()) {
        for x in obj.get_dependencies() {
            write_object(write, con, *x)?;
        }
        obj.serialize(id, write, con)?;
    }
    Ok(())
}

pub fn write_object_dag(write: &mut dyn std::io::Write, obj: &dyn SerializationNode) -> Result<()> {
    let mut con = WritingContext(HashMap::new());
    write_object(write, &mut con, obj)
}
