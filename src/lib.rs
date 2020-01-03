mod block;
mod primitive;
mod read;
mod refobj;
mod write;
use primitive::*;
use std::rc::Rc;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Tag {
    END,
    OBJ,
    U8,
    U16,
    I32,
}

#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
    TagMismatch,
    DeserializerNotFound,
    ObjNotFound,
}

impl std::convert::From<std::io::Error> for Error {
    fn from(src: std::io::Error) -> Self {
        Error::IoError(src)
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub trait Read: ReadPrimitive {
    fn obj<T: Deserialize>(&mut self) -> Result<T>;
    fn rc<T: 'static>(&mut self) -> Result<Rc<T>>;
}

pub trait Write: WritePrimitive {
    fn obj<S: Serialize>(&mut self, x: &S) -> Result<()>;
    fn rc<T>(&mut self, x: &Rc<T>) -> Result<()>;
}

pub trait Serialize {
    const VERSION: u16 = 0;
    fn serialize(&self, write: &mut (impl Write + ?Sized)) -> Result<()>;
}

pub trait Deserialize: Sized {
    fn deserialize(read: &mut (impl Read + ?Sized), version: u16) -> Result<Self>;
}

pub trait TypeKey {
    const TYPE_KEY: &'static str;
}

pub trait DynSerialize {
    fn serialize(
        &self,
        id: u32,
        write: &mut dyn std::io::Write,
        con: &write::WritingContext,
    ) -> Result<()>;
}

pub trait SerializationNode: DynSerialize {
    fn get_dependencies(&self) -> &[&dyn SerializationNode];
}

pub use write::write_object_dag;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
