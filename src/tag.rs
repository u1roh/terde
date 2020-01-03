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
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Value {
    Begin,
    End,
    U8(u8),
    U16(u16),
    U32(u32),
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
        }
    }
}

pub trait TagRead: ReadPrimitive {
    fn begin(&mut self) -> Result<()>;
    fn end(&mut self) -> Result<()>;
    fn primitive(&mut self) -> Result<Value>;
}

pub fn create_writer<T: WritePrimitive>(write: T) -> impl TagWrite {
    Writer(write)
}

pub fn create_reader<T: ReadPrimitive>(read: T) -> impl TagRead {
    Reader(read)
}

struct Writer<T>(T);
struct Reader<T>(T);

impl<T: WritePrimitive> Writer<T> {
    fn tag(&mut self, tag: Tag) -> Result<()> {
        self.0.u8(tag as u8)
    }
}

impl<T: WritePrimitive> WritePrimitive for Writer<T> {
    fn u8(&mut self, x: u8) -> Result<()> {
        self.tag(Tag::U8)?;
        self.0.u8(x)
    }
    fn u16(&mut self, x: u16) -> Result<()> {
        self.tag(Tag::U16)?;
        self.0.u16(x)
    }
    fn u32(&mut self, x: u32) -> Result<()> {
        self.tag(Tag::U32)?;
        self.0.u32(x)
    }
    fn str(&mut self, _: &str) -> Result<()> {
        unimplemented!()
    }
}

impl<T: WritePrimitive> TagWrite for Writer<T> {
    fn begin(&mut self) -> Result<()> {
        self.tag(Tag::BEGIN)
    }
    fn end(&mut self) -> Result<()> {
        self.tag(Tag::END)
    }
}

impl<T: ReadPrimitive> Reader<T> {
    fn read_tag(&mut self) -> Result<Tag> {
        Ok(unsafe { std::mem::transmute(self.0.u8()?) })
    }
    fn tag(&mut self, tag: Tag) -> Result<()> {
        if self.read_tag()? == tag {
            Ok(())
        } else {
            Err(Error::TagMismatch)
        }
    }
}

impl<T: ReadPrimitive> ReadPrimitive for Reader<T> {
    fn u8(&mut self) -> Result<u8> {
        self.tag(Tag::U8)?;
        self.0.u8()
    }
    fn u16(&mut self) -> Result<u16> {
        self.tag(Tag::U16)?;
        self.0.u16()
    }
    fn u32(&mut self) -> Result<u32> {
        self.tag(Tag::U32)?;
        self.0.u32()
    }
    fn str(&mut self) -> Result<String> {
        unimplemented!()
    }
}

impl<T: ReadPrimitive> TagRead for Reader<T> {
    fn primitive(&mut self) -> Result<Value> {
        Ok(match self.read_tag()? {
            Tag::BEGIN => Value::Begin,
            Tag::END => Value::End,
            Tag::U8 => Value::U8(self.0.u8()?),
            Tag::U16 => Value::U16(self.0.u16()?),
            Tag::U32 => Value::U32(self.0.u32()?),
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
