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
    fn i32(&mut self, x: i32) -> Result<()> {
        self.tag(Tag::I32)?;
        self.write.i32(x)
    }
    fn str(&mut self, x: &str) -> Result<()> {
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
        self.i32(key as i32)
    }
}

pub struct WritingContext {
    shared_objects: HashMap<*const (), u32>,
}

impl WritingContext {
    fn rc<T>(&self, x: &Rc<T>) -> Option<u32> {
        use std::ops::Deref;
        self.shared_objects
            .get(&(x.deref() as *const _ as *const ()))
            .map(|key| *key)
    }
    pub(super) fn emit(&mut self, x: *const ()) -> Option<u32> {
        use std::collections::hash_map::Entry;
        let id = self.shared_objects.len() as u32;
        match self.shared_objects.entry(x) {
            Entry::Occupied(_) => None,
            Entry::Vacant(v) => {
                v.insert(id);
                Some(id)
            }
        }
    }
}

pub(super) fn create<'a, W: std::io::Write + 'a>(
    write: W,
    con: &'a WritingContext,
) -> impl Write + 'a {
    Writer { write, con }
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
        //obj.serialize(id, write, con)?;
    }
    Ok(())
}
