use super::*;
use crate::dynobj::*;
use crate::tag::*;
use std::collections::HashMap;

type DeserializeFn = Box<dyn Fn(&mut dyn DynRead) -> Result<DynObj>>;

fn deserializer<T: Deserialize + 'static>() -> DeserializeFn {
    Box::new(|read| {
        let obj = read.obj::<T>()?;
        Ok(DynObj::new(obj))
    })
}

#[derive(Default)]
pub struct DeserializerRegistry(HashMap<u128, DeserializeFn>);
impl DeserializerRegistry {
    pub fn new() -> Self {
        Self(HashMap::new())
    }
    pub fn register<T: Deserialize + TypeKey + 'static>(&mut self) {
        self.0.insert(T::TYPE_KEY, deserializer::<T>());
    }
    fn read_next_object(&self, read: &mut dyn DynRead) -> Result<(u32, DynObj)> {
        read.begin()?;
        let id = read.u32()?;
        let type_key = read.u128()?;
        let des = self.0.get(&type_key).ok_or(Error::DeserializerNotFound)?;
        let obj = des(read)?;
        read.end()?;
        println!("id = {}, type_key = {}, obj = {:?}", id, type_key, obj);
        Ok((id, obj))
    }
    pub fn read_object(&self, read: impl TagRead) -> Result<DynObj> {
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
    con: HashMap<u32, DynObj>,
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
        fn u128(&mut self) -> Result<u128> {
            self.read.u128()
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

    impl<T: TagRead> DynRead for Reader<T> {
        fn ptr(&mut self) -> Result<&DynObj> {
            let id = self.read.u32()?;
            self.con.get(&id).ok_or(Error::ObjNotFound)
        }
    }
}
