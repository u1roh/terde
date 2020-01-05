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
    NotImplemented,
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

pub trait Write: WritePrimitive {
    fn obj<S: Serialize>(&mut self, x: &S) -> Result<()>
    where
        Self: tag::TagWrite,
    {
        self.begin()?;
        self.u16(S::VERSION)?;
        x.serialize(self)?;
        self.end()
    }
    fn rc<T>(&mut self, _: &Rc<T>) -> Result<()>;
}

pub trait Read: ReadPrimitive {
    fn obj<T: Deserialize>(&mut self) -> Result<T>
    where
        Self: tag::TagRead,
    {
        self.begin()?;
        let version = self.u16()?;
        let obj = T::deserialize(self, version)?;
        self.end()?;
        Ok(obj)
    }
    fn rc<T: 'static>(&mut self) -> Result<Rc<T>>;
}

impl<W: tag::TagWrite> Write for W {
    fn rc<T>(&mut self, _: &Rc<T>) -> Result<()> {
        Err(Error::NotImplemented)
    }
}

impl<R: tag::TagRead> Read for R {
    fn rc<T>(&mut self) -> Result<Rc<T>> {
        Err(Error::NotImplemented)
    }
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

pub trait SerializationNode: dynobj::DynSerialize {
    fn get_dependencies(&self) -> &[&dyn SerializationNode];
}

pub use read::DeserializerRegistry;
pub use write::write_object;

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    struct DataVer0 {
        a: u32,
        b: u16,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    struct DataVer1 {
        a: u32,
        b: u16,
        c: u8,
    }

    impl Serialize for DataVer0 {
        fn serialize(&self, write: &mut (impl Write + ?Sized)) -> Result<()> {
            write.u32(self.a)?;
            write.u16(self.b)?;
            Ok(())
        }
    }
    impl Serialize for DataVer1 {
        const VERSION: u16 = 1;
        fn serialize(&self, write: &mut (impl Write + ?Sized)) -> Result<()> {
            write.u32(self.a)?;
            write.u16(self.b)?;
            write.u8(self.c)?;
            Ok(())
        }
    }
    impl Deserialize for DataVer1 {
        fn deserialize(read: &mut (impl Read + ?Sized), version: u16) -> Result<Self> {
            match version {
                0 => Ok(DataVer1 {
                    a: read.u32()?,
                    b: read.u16()?,
                    c: 0,
                }),
                1 => Ok(DataVer1 {
                    a: read.u32()?,
                    b: read.u16()?,
                    c: read.u8()?,
                }),
                _ => Err(Error::NotImplemented),
            }
        }
    }

    #[test]
    fn it_works() {
        let x1 = DataVer0 { a: 123, b: 456 };
        let x2 = DataVer1 {
            a: 321,
            b: 654,
            c: 111,
        };
        {
            let mut bin = Vec::<u8>::new();
            bin.obj(&x1).unwrap();
            let y = bin.as_slice().obj::<DataVer1>().unwrap();
            let y = DataVer0 { a: y.a, b: y.b };
            assert_eq!(x1, y);
        }
        {
            let mut bin = Vec::<u8>::new();
            bin.obj(&x2).unwrap();
            let y = bin.as_slice().obj::<DataVer1>().unwrap();
            assert_eq!(x2, y);
        }
    }
}
