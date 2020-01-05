# terde

A serialization library. (experimental)

A entity struct can be modified in future.
So, the entity of newer version may have fields which doesn't exist in older version.

Since `terde` serialize entities with version number, you can deserialize it robustly.

```rust
use terde::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct DataVer0 {
    a: u32,
    b: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct DataVer1 {
    a: u32,
    b: u16,
    c: u8,
}

impl Serialize for DataVer0 {
    fn serialize(&self, write: &mut (impl Write + ?Sized)) -> Result<()> {
        write.u32(self.a)?;
        write.u16(self.b)?;
        Ok(())
    }
}
impl Serialize for DataVer1 {
    const VERSION: u16 = 1;
    fn serialize(&self, write: &mut (impl Write + ?Sized)) -> Result<()> {
        write.u32(self.a)?;
        write.u16(self.b)?;
        write.u8(self.c)?;
        Ok(())
    }
}
impl Deserialize for DataVer1 {
    fn deserialize(read: &mut (impl Read + ?Sized), version: u16) -> Result<Self> {
        match version {
            0 => Ok(DataVer1 {
                a: read.u32()?,
                b: read.u16()?,
                c: 0,
            }),
            1 => Ok(DataVer1 {
                a: read.u32()?,
                b: read.u16()?,
                c: read.u8()?,
            }),
            _ => Err(Error::NotImplemented),
        }
    }
}

#[test]
fn it_works() {
    let x1 = DataVer0 { a: 123, b: 456 };
    let x2 = DataVer1 {
        a: 321,
        b: 654,
        c: 111,
    };
    {
        let mut bin = Vec::<u8>::new();
        bin.obj(&x1).unwrap();
        let y = bin.as_slice().obj::<DataVer1>().unwrap();
        let y = DataVer0 { a: y.a, b: y.b };
        assert_eq!(x1, y);
    }
    {
        let mut bin = Vec::<u8>::new();
        bin.obj(&x2).unwrap();
        let y = bin.as_slice().obj::<DataVer1>().unwrap();
        assert_eq!(x2, y);
    }
}
```
