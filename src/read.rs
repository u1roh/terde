use super::*;
use crate::dynobj::*;
use crate::tag::*;
use std::collections::HashMap;

type DeserializeFn = Box<dyn Fn(&mut dyn ReadRef) -> Result<RefObj>>;

fn deserializer<T: Deserialize + 'static>() -> DeserializeFn {
    Box::new(|read| {
        let obj = read.obj::<T>()?;
        Ok(RefObj::new(obj))
    })
}

#[derive(Default)]
pub struct DeserializerRegistry(HashMap<String, DeserializeFn>);
impl DeserializerRegistry {
    pub fn new() -> Self {
        Self(HashMap::new())
    }
    pub fn register<T: Deserialize + TypeKey + 'static>(&mut self) {
        self.0.insert(T::TYPE_KEY.to_string(), deserializer::<T>());
    }
    fn read_next_object(&self, read: &mut dyn ReadRef) -> Result<(u32, RefObj)> {
        read.begin()?;
        let id = read.u32()?;
        let type_key = read.str()?;
        let des = self.0.get(&type_key).ok_or(Error::DeserializerNotFound)?;
        let obj = des(read)?;
        Ok((id, obj))
    }
    pub fn read_object(&self, read: impl TagRead) -> Result<RefObj> {
        let mut reader = Reader {
            read,
            con: HashMap::new(),
        };
        let mut last = None;
        while let Ok((id, obj)) = self.read_next_object(&mut reader) {
            reader.con.insert(id, obj);
            last = Some(id);
        }
        last.and_then(|id| reader.con.remove(&id))
            .ok_or(Error::ObjNotFound)
    }
}

struct Reader<T> {
    read: T,
    con: HashMap<u32, RefObj>,
}

mod impl_traits {
    use super::*;

    impl<T: ReadPrimitive> ReadPrimitive for Reader<T> {
        fn u8(&mut self) -> Result<u8> {
            self.read.u8()
        }
        fn u16(&mut self) -> Result<u16> {
            self.read.u16()
        }
        fn u32(&mut self) -> Result<u32> {
            self.read.u32()
        }
        fn str(&mut self) -> Result<String> {
            self.read.str()
        }
    }

    impl<T: TagRead> TagRead for Reader<T> {
        fn begin(&mut self) -> Result<()> {
            self.read.begin()
        }
        fn end(&mut self) -> Result<()> {
            self.read.end()
        }
        fn primitive(&mut self) -> Result<Value> {
            self.read.primitive()
        }
    }

    impl<T: TagRead> ReadRef for Reader<T> {
        fn ptr(&mut self) -> Result<&RefObj> {
            let id = self.read.u32()?;
            self.con.get(&id).ok_or(Error::ObjNotFound)
        }
    }
}
