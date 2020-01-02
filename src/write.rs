use super::*;

pub trait Write: WritePrimitive {
    fn obj<S: Serialize>(&mut self, x: &S) -> Result<()>;
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
        self.0.u8(x)
    }
    fn u16(&mut self, x: u16) -> Result<()> {
        self.tag(Tag::U16)?;
        self.0.u16(x)
    }
    fn i32(&mut self, x: i32) -> Result<()> {
        self.tag(Tag::I32)?;
        self.0.i32(x)
    }
    fn str(&mut self, x: &str) -> Result<()> {
        unimplemented!()
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

trait SerializationNode {
    fn get_dependencies(&self) -> &[&dyn SerializationNode];
    fn serialize(&self, write: &mut dyn std::io::Write) -> Result<()>
    where
        Self: Serialize + TypeKey + Sized,
    {
        let mut write = Writer(write);
        write.str(Self::TYPE_KEY)?;
        write.obj(self)
    }
}
