use super::*;
use crate::primitive::*;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Tag {
    BEGIN,
    END,
    U8,
    U16,
    U32,
    U128,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Value {
    Begin,
    End,
    U8(u8),
    U16(u16),
    U32(u32),
    U128(u128),
}

pub trait TagWrite: WritePrimitive {
    fn begin(&mut self) -> Result<()>;
    fn end(&mut self) -> Result<()>;
    fn primitive(&mut self, x: Value) -> Result<()> {
        match x {
            Value::Begin => self.begin(),
            Value::End => self.end(),
            Value::U8(x) => self.u8(x),
            Value::U16(x) => self.u16(x),
            Value::U32(x) => self.u32(x),
            Value::U128(x) => self.u128(x),
        }
    }
}

pub trait TagRead: ReadPrimitive {
    fn begin(&mut self) -> Result<()>;
    fn end(&mut self) -> Result<()>;
    fn primitive(&mut self) -> Result<Value>;
}

trait WriteTag {
    fn tag(&mut self, tag: Tag) -> Result<()>;
}

impl<W: PrimitiveWrite> WriteTag for W {
    fn tag(&mut self, tag: Tag) -> Result<()> {
        self.write8(tag as u8)
    }
}

impl<W: PrimitiveWrite> WritePrimitive for W {
    fn u8(&mut self, x: u8) -> Result<()> {
        self.tag(Tag::U8)?;
        self.write8(x)
    }
    fn u16(&mut self, x: u16) -> Result<()> {
        self.tag(Tag::U16)?;
        self.write16(x)
    }
    fn u32(&mut self, x: u32) -> Result<()> {
        self.tag(Tag::U32)?;
        self.write32(x)
    }
    fn u128(&mut self, x: u128) -> Result<()> {
        self.tag(Tag::U128)?;
        self.write128(x)
    }
    fn str(&mut self, _: &str) -> Result<()> {
        unimplemented!()
    }
}

impl<T: PrimitiveWrite> TagWrite for T {
    fn begin(&mut self) -> Result<()> {
        self.tag(Tag::BEGIN)
    }
    fn end(&mut self) -> Result<()> {
        self.tag(Tag::END)
    }
}

trait ReadTag {
    fn read_tag(&mut self) -> Result<Tag>;
    fn tag(&mut self, tag: Tag) -> Result<()> {
        if self.read_tag()? == tag {
            Ok(())
        } else {
            Err(Error::TagMismatch)
        }
    }
}

impl<R: PrimitiveRead> ReadTag for R {
    fn read_tag(&mut self) -> Result<Tag> {
        Ok(unsafe { std::mem::transmute(self.read8()?) })
    }
}

impl<R: PrimitiveRead> ReadPrimitive for R {
    fn u8(&mut self) -> Result<u8> {
        self.tag(Tag::U8)?;
        self.read8()
    }
    fn u16(&mut self) -> Result<u16> {
        self.tag(Tag::U16)?;
        self.read16()
    }
    fn u32(&mut self) -> Result<u32> {
        self.tag(Tag::U32)?;
        self.read32()
    }
    fn u128(&mut self) -> Result<u128> {
        self.tag(Tag::U128)?;
        self.read128()
    }
    fn str(&mut self) -> Result<String> {
        unimplemented!()
    }
}

impl<R: PrimitiveRead> TagRead for R {
    fn primitive(&mut self) -> Result<Value> {
        Ok(match self.read_tag()? {
            Tag::BEGIN => Value::Begin,
            Tag::END => Value::End,
            Tag::U8 => Value::U8(self.read8()?),
            Tag::U16 => Value::U16(self.read16()?),
            Tag::U32 => Value::U32(self.read32()?),
            Tag::U128 => Value::U128(self.read128()?),
        })
    }
    fn begin(&mut self) -> Result<()> {
        self.tag(Tag::BEGIN)
    }
    fn end(&mut self) -> Result<()> {
        loop {
            match self.primitive()? {
                Value::End => return Ok(()),
                Value::Begin => self.end()?,
                _ => {}
            }
        }
    }
}
