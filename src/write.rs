use super::*;
use crate::refobj::*;
use crate::tag::*;
use std::collections::HashMap;

struct WritingContext(HashMap<*const (), u32>);

impl WritingContext {
    fn ptr2key(&self, ptr: *const ()) -> Result<u32> {
        self.0.get(&ptr).map(|key| *key).ok_or(Error::ObjNotFound)
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

impl<T: TagWrite> TagWrite for Writer<T> {
    fn begin(&mut self) -> Result<()> {
        self.write.begin()
    }
    fn end(&mut self) -> Result<()> {
        self.write.end()
    }
    fn primitive(&mut self, x: Value) -> Result<()> {
        self.write.primitive(x)
    }
}

impl<T: TagWrite> WriteRef for Writer<T> {
    fn ptr(&mut self, ptr: *const ()) -> Result<()> {
        let key = self.con.ptr2key(ptr)?;
        self.write.u32(key)
    }
}

fn write_object_recursive<T: TagWrite>(
    write: &mut Writer<T>,
    obj: &dyn SerializationNode,
) -> Result<()> {
    if let Some(id) = write.con.emit(obj as *const _ as *const ()) {
        for x in obj.get_dependencies() {
            write_object_recursive(write, *x)?;
        }
        write.begin()?;
        write.u32(id)?;
        write.str(obj.type_key())?;
        obj.serialize(write)?;
        write.end()?;
    }
    Ok(())
}

pub fn write_object(write: impl TagWrite, obj: &dyn SerializationNode) -> Result<()> {
    let mut writer = Writer {
        write,
        con: WritingContext(HashMap::new()),
    };
    write_object_recursive(&mut writer, obj)
}
