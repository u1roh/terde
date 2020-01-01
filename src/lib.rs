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
    fn i32(&mut self, x: i32) -> Result<()>;
}

pub trait ReadPrimitive {
    fn u8(&mut self) -> Result<u8>;
    fn u16(&mut self) -> Result<u16>;
    fn i32(&mut self) -> Result<i32>;
}

pub trait Write: WritePrimitive {
    fn obj<S: Serialize>(&mut self, x: &S) -> Result<()>;
}

pub trait Read: ReadPrimitive {
    fn obj<S: Deserialize>(&mut self) -> Result<S>;
}

impl<W: std::io::Write> WritePrimitive for W {
    fn u8(&mut self, x: u8) -> Result<()> {
        unsafe {
            self.write_all(&std::mem::transmute::<u8, [u8; 1]>(x))?;
        }
        Ok(())
    }
    fn u16(&mut self, x: u16) -> Result<()> {
        unsafe {
            self.write_all(&std::mem::transmute::<u16, [u8; 2]>(x))?;
        }
        Ok(())
    }
    fn i32(&mut self, x: i32) -> Result<()> {
        unsafe {
            self.write_all(&std::mem::transmute::<i32, [u8; 4]>(x))?;
        }
        Ok(())
    }
}

impl<R: std::io::Read> ReadPrimitive for R {
    fn u8(&mut self) -> Result<u8> {
        let mut buf = [0u8; 1];
        self.read_exact(&mut buf)?;
        Ok(unsafe { std::mem::transmute(buf) })
    }
    fn u16(&mut self) -> Result<u16> {
        let mut buf = [0u8; 2];
        self.read_exact(&mut buf)?;
        Ok(unsafe { std::mem::transmute(buf) })
    }
    fn i32(&mut self) -> Result<i32> {
        let mut buf = [0u8; 4];
        self.read_exact(&mut buf)?;
        Ok(unsafe { std::mem::transmute(buf) })
    }
}

struct Writer<W: std::io::Write>(W);

impl<W: std::io::Write> Writer<W> {
    fn tag(&mut self, tag: Tag) -> Result<()> {
        self.0.u8(tag as u8)
    }
}
impl<W: std::io::Write> WritePrimitive for Writer<W> {
    fn u8(&mut self, x: u8) -> Result<()> {
        self.tag(Tag::U8)?;
        self.0.u8(x)?;
        Ok(())
    }
    fn u16(&mut self, x: u16) -> Result<()> {
        self.tag(Tag::U16)?;
        self.0.u16(x)?;
        Ok(())
    }
    fn i32(&mut self, x: i32) -> Result<()> {
        self.tag(Tag::I32)?;
        self.0.i32(x)?;
        Ok(())
    }
}
impl<W: std::io::Write> Write for Writer<W> {
    fn obj<S: Serialize>(&mut self, x: &S) -> Result<()> {
        self.tag(Tag::OBJ)?;
        self.0.u16(S::VERSION)?;
        x.serialize(self)?;
        self.tag(Tag::END)?;
        Ok(())
    }
}

struct Reader<R: std::io::Read>(R);

impl<R: std::io::Read> Reader<R> {
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
}
impl<R: std::io::Read> ReadPrimitive for Reader<R> {
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
}

impl<R: std::io::Read> Read for Reader<R> {
    fn obj<S: Deserialize>(&mut self) -> Result<S> {
        if self.tag()? != Tag::OBJ {
            return Err(Error::TagMismatch);
        }
        let version = self.0.u16()?;
        let obj = S::deserialize(self, version)?;
        self.skip_to_end()?;
        Ok(obj)
    }
}

pub trait Serialize {
    const VERSION: u16 = 0;
    fn serialize(&self, write: &mut impl Write) -> Result<()>;
}

pub trait Deserialize: Sized {
    fn deserialize(read: &mut impl Read, version: u16) -> Result<Self>;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
