use super::Result;

pub trait WritePrimitive {
    fn u8(&mut self, x: u8) -> Result<()>;
    fn u16(&mut self, x: u16) -> Result<()>;
    fn i32(&mut self, x: i32) -> Result<()>;
    fn str(&mut self, x: &str) -> Result<()>;
}
pub trait ReadPrimitive {
    fn u8(&mut self) -> Result<u8>;
    fn u16(&mut self) -> Result<u16>;
    fn i32(&mut self) -> Result<i32>;
    fn str(&mut self) -> Result<String>;
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
    fn str(&mut self, x: &str) -> Result<()> {
        unimplemented!()
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
    fn str(&mut self) -> Result<String> {
        unimplemented!()
    }
}
