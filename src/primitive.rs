use super::Result;

pub trait PrimitiveWrite {
    fn write8(&mut self, x: u8) -> Result<()>;
    fn write16(&mut self, x: u16) -> Result<()>;
    fn write32(&mut self, x: u32) -> Result<()>;
}

pub trait PrimitiveRead {
    fn read8(&mut self) -> Result<u8>;
    fn read16(&mut self) -> Result<u16>;
    fn read32(&mut self) -> Result<u32>;
}

impl<W: std::io::Write> PrimitiveWrite for W {
    fn write8(&mut self, x: u8) -> Result<()> {
        unsafe {
            self.write_all(&std::mem::transmute::<u8, [u8; 1]>(x))?;
        }
        Ok(())
    }
    fn write16(&mut self, x: u16) -> Result<()> {
        unsafe {
            self.write_all(&std::mem::transmute::<u16, [u8; 2]>(x))?;
        }
        Ok(())
    }
    fn write32(&mut self, x: u32) -> Result<()> {
        unsafe {
            self.write_all(&std::mem::transmute::<u32, [u8; 4]>(x))?;
        }
        Ok(())
    }
}

impl<R: std::io::Read> PrimitiveRead for R {
    fn read8(&mut self) -> Result<u8> {
        let mut buf = [0u8; 1];
        self.read_exact(&mut buf)?;
        Ok(unsafe { std::mem::transmute(buf) })
    }
    fn read16(&mut self) -> Result<u16> {
        let mut buf = [0u8; 2];
        self.read_exact(&mut buf)?;
        Ok(unsafe { std::mem::transmute(buf) })
    }
    fn read32(&mut self) -> Result<u32> {
        let mut buf = [0u8; 4];
        self.read_exact(&mut buf)?;
        Ok(unsafe { std::mem::transmute(buf) })
    }
}
