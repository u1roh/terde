mod dynobj;
mod primitive;
mod read;
mod tag;
mod write;
use std::rc::Rc;

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

pub trait WritePrimitive {
    fn u8(&mut self, x: u8) -> Result<()>;
    fn u16(&mut self, x: u16) -> Result<()>;
    fn u32(&mut self, x: u32) -> Result<()>;
    fn str(&mut self, x: &str) -> Result<()>;
}

pub trait ReadPrimitive {
    fn u8(&mut self) -> Result<u8>;
    fn u16(&mut self) -> Result<u16>;
    fn u32(&mut self) -> Result<u32>;
    fn str(&mut self) -> Result<String>;
}

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
    fn type_key(&self) -> &'static str;
    fn serialize(&self, write: &mut dyn dynobj::WriteRef) -> Result<()>;
}

pub trait SerializationNode: DynSerialize {
    fn get_dependencies(&self) -> &[&dyn SerializationNode];
}

pub use read::DeserializerRegistry;
//pub use tag::create_reader;
//pub use tag::create_writer;
pub use write::write_object;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
