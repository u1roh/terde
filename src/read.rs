use super::*;
use std::any::Any;
use std::collections::HashMap;
use std::rc::Rc;

struct Reader<'a, R: std::io::Read>(R, &'a mut ReadingContext);

impl<'a, R: std::io::Read> Reader<'a, R> {
    fn tag(&mut self) -> Result<Tag> {
        Ok(unsafe { std::mem::transmute(self.0.u8()?) })
    }
    fn ensure(&mut self, tag: Tag) -> Result<()> {
        if self.tag()? == tag {
            Ok(())
        } else {
            Err(Error::TagMismatch)
        }
    }
    fn skip_to_end(&mut self) -> Result<()> {
        let mut buf = [0u8; 8];
        loop {
            let tag = self.tag()?;
            match tag {
                Tag::END => return Ok(()),
                Tag::OBJ => {
                    let _version = self.0.u16()?;
                    self.skip_to_end()?;
                }
                Tag::U8 => self.0.read_exact(&mut buf[..1])?,
                Tag::U16 => self.0.read_exact(&mut buf[..2])?,
                Tag::I32 => self.0.read_exact(&mut buf[..4])?,
            }
        }
    }
    fn read(&mut self, deserializers: &HashMap<String, DeserializeFn>) -> Result<()> {
        let obj_key = self.i32()?;
        let type_key = self.str()?;
        let deserialize = deserializers
            .get(&type_key)
            .ok_or(Error::DeserializerNotFound)?;
        let obj = deserialize(&mut self.0, &mut self.1)?;
        self.1.shared_objects.insert(obj_key as u32, obj);
        Ok(())
    }
}
impl<'a, R: std::io::Read> ReadPrimitive for Reader<'a, R> {
    fn u8(&mut self) -> Result<u8> {
        self.ensure(Tag::U8)?;
        self.0.u8()
    }
    fn u16(&mut self) -> Result<u16> {
        self.ensure(Tag::U16)?;
        self.0.u16()
    }
    fn i32(&mut self) -> Result<i32> {
        self.ensure(Tag::I32)?;
        self.0.i32()
    }
    fn str(&mut self) -> Result<String> {
        unimplemented!()
    }
}

impl<'a, R: std::io::Read> Read for Reader<'a, R> {
    fn obj<T: Deserialize>(&mut self) -> Result<T> {
        if self.tag()? != Tag::OBJ {
            return Err(Error::TagMismatch);
        }
        let version = self.0.u16()?;
        let obj = T::deserialize(self, version)?;
        self.skip_to_end()?;
        Ok(obj)
    }
    fn rc<T: 'static>(&mut self) -> Result<Rc<T>> {
        let obj_key = self.i32()?;
        self.1.rc::<T>(obj_key as u32).ok_or(Error::ObjNotFound)
    }
}

type DeserializeFn = Box<dyn Fn(&mut dyn std::io::Read, &mut ReadingContext) -> Result<Shared>>;

fn deserializer<T: Deserialize + 'static>() -> DeserializeFn {
    Box::new(|read, con| {
        let mut read = Reader(read, con);
        let obj = read.obj::<T>()?;
        Ok(Shared::new(obj))
    })
}

struct Shared(Box<dyn Any>);
impl Shared {
    fn new<T: 'static>(x: T) -> Self {
        Self(Box::new(Rc::new(x)))
    }
    fn as_rc<T: 'static>(&self) -> Option<Rc<T>> {
        self.0.downcast_ref::<Rc<T>>().map(|rc| rc.clone())
    }
}

pub struct ReadingContext {
    shared_objects: HashMap<u32, Shared>,
}

impl ReadingContext {
    pub fn rc<T: 'static>(&self, key: u32) -> Option<Rc<T>> {
        self.shared_objects
            .get(&key)
            .and_then(|shared| shared.as_rc::<T>())
    }
}
