use super::Result;

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
    fn u32(&mut self, x: u32) -> Result<()> {
        unsafe {
            self.write_all(&std::mem::transmute::<u32, [u8; 4]>(x))?;
        }
        Ok(())
    }
    fn str(&mut self, _: &str) -> Result<()> {
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
    fn u32(&mut self) -> Result<u32> {
        let mut buf = [0u8; 4];
        self.read_exact(&mut buf)?;
        Ok(unsafe { std::mem::transmute(buf) })
    }
    fn str(&mut self) -> Result<String> {
        unimplemented!()
    }
}
