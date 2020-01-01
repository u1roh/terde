mod code {
    pub const END: u8 = 0;
    pub const OBJ: u8 = 1;
    pub const U8: u8 = 2;
    pub const U16: u8 = 3;
    pub const I32: u8 = 4;
}

pub type Result<T> = std::io::Result<T>;

pub trait WritePrimitive {
    fn u8(&mut self, x: u8) -> Result<()>;
    fn u16(&mut self, x: u16) -> Result<()>;
    fn i32(&mut self, x: i32) -> Result<()>;
}

pub trait Write: WritePrimitive {
    fn obj<S: Serialize>(&mut self, x: &S) -> Result<()>;
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

pub trait Serialize {
    const VERSION: u16 = 0;
    fn serialize(&self, write: &mut impl Write) -> Result<()>;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
