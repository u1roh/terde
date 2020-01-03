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
pub enum Primitive {
    Begin,
    End,
    U8(u8),
    U16(u16),
    U32(u32),
}

pub trait WriteBlock: WritePrimitive {
    fn primitive(&mut self, x: Primitive) -> Result<()>;
    fn begin(&mut self) -> Result<()>;
    fn end(&mut self) -> Result<()>;
}

pub trait ReadBlock: ReadPrimitive {
    fn primitive(&mut self) -> Result<Primitive>;
    fn begin(&mut self) -> Result<()>;
    fn end(&mut self) -> Result<()>;
}

pub fn create_writer<T: WritePrimitive>(write: T) -> impl WriteBlock {
    Writer(write)
}

pub fn create_reader<T: ReadPrimitive>(read: T) -> impl ReadBlock {
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

impl<T: WritePrimitive> WriteBlock for Writer<T> {
    fn primitive(&mut self, x: Primitive) -> Result<()> {
        match x {
            Primitive::Begin => self.begin(),
            Primitive::End => self.end(),
            Primitive::U8(x) => self.u8(x),
            Primitive::U16(x) => self.u16(x),
            Primitive::U32(x) => self.u32(x),
        }
    }
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

impl<T: ReadPrimitive> ReadBlock for Reader<T> {
    fn primitive(&mut self) -> Result<Primitive> {
        Ok(match self.read_tag()? {
            Tag::BEGIN => Primitive::Begin,
            Tag::END => Primitive::End,
            Tag::U8 => Primitive::U8(self.0.u8()?),
            Tag::U16 => Primitive::U16(self.0.u16()?),
            Tag::U32 => Primitive::U32(self.0.u32()?),
        })
    }
    fn begin(&mut self) -> Result<()> {
        unimplemented!()
    }
    fn end(&mut self) -> Result<()> {
        loop {
            match self.primitive()? {
                Primitive::End => return Ok(()),
                Primitive::Begin => self.end()?,
                _ => {}
            }
        }
    }
}
