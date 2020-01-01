mod code {
    pub const END: u8 = 0;
    pub const OBJ: u8 = 1;
    pub const U8: u8 = 2;
    pub const U16: u8 = 3;
    pub const I32: u8 = 4;
}

#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
    TypeMismatch,
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
            self.write(&std::mem::transmute::<u8, [u8; 1]>(x))?;
        }
        Ok(())
    }
    fn u16(&mut self, x: u16) -> Result<()> {
        unsafe {
            self.write(&std::mem::transmute::<u16, [u8; 2]>(x))?;
        }
        Ok(())
    }
    fn i32(&mut self, x: i32) -> Result<()> {
        unsafe {
            self.write(&std::mem::transmute::<i32, [u8; 4]>(x))?;
        }
        Ok(())
    }
}

impl<R: std::io::Read> ReadPrimitive for R {
    fn u8(&mut self) -> Result<u8> {
        let mut buf = [0u8; 1];
        self.read(&mut buf)?;
        Ok(unsafe { std::mem::transmute(buf) })
    }
    fn u16(&mut self) -> Result<u16> {
        let mut buf = [0u8; 2];
        self.read(&mut buf)?;
        Ok(unsafe { std::mem::transmute(buf) })
    }
    fn i32(&mut self) -> Result<i32> {
        let mut buf = [0u8; 4];
        self.read(&mut buf)?;
        Ok(unsafe { std::mem::transmute(buf) })
    }
}

struct Writer<W: std::io::Write>(W);

impl<W: std::io::Write> WritePrimitive for Writer<W> {
    fn u8(&mut self, x: u8) -> Result<()> {
        self.0.u8(code::U8)?;
        self.0.u8(x)?;
        Ok(())
    }
    fn u16(&mut self, x: u16) -> Result<()> {
        self.0.u8(code::U16)?;
        self.0.u16(x)?;
        Ok(())
    }
    fn i32(&mut self, x: i32) -> Result<()> {
        self.0.u8(code::I32)?;
        self.0.i32(x)?;
        Ok(())
    }
}
impl<W: std::io::Write> Write for Writer<W> {
    fn obj<S: Serialize>(&mut self, x: &S) -> Result<()> {
        self.0.u8(code::OBJ)?;
        self.0.u16(S::VERSION)?;
        x.serialize(self)?;
        self.0.u8(code::END)?;
        Ok(())
    }
}

struct Reader<R: std::io::Read>(R);

impl<R: std::io::Read> ReadPrimitive for Reader<R> {
    fn u8(&mut self) -> Result<u8> {
        if self.0.u8()? == code::U8 {
            self.0.u8()
        } else {
            Err(Error::TypeMismatch)
        }
    }
    fn u16(&mut self) -> Result<u16> {
        if self.0.u8()? == code::U16 {
            self.0.u16()
        } else {
            Err(Error::TypeMismatch)
        }
    }
    fn i32(&mut self) -> Result<i32> {
        if self.0.u8()? == code::I32 {
            self.0.i32()
        } else {
            Err(Error::TypeMismatch)
        }
    }
}

pub trait Serialize {
    const VERSION: u16 = 0;
    fn serialize(&self, write: &mut impl Write) -> Result<()>;
}

pub trait Deserialize: Sized {
    fn deserialize(read: &mut impl Read) -> Result<Self>;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
