use std::io;

pub trait Serialize: Sized {
    fn serialize<W: io::Write>(&self, dst: W) -> io::Result<usize>;
    fn deserialize<R: io::Read>(src: R) -> io::Result<Self>;
}

macro_rules! serializable_default {
    ($t: ty) => {
        impl Serialize for $t {
            fn serialize<W: io::Write>(&self, mut dst: W) -> io::Result<usize> {
                dst.write(&self.to_ne_bytes())
            }
            fn deserialize<R: io::Read>(mut src: R) -> io::Result<Self> {
                let mut value: Self = Self::default();
                unsafe {
                    let buffer = std::slice::from_raw_parts_mut(
                        &mut value as *mut Self as *mut u8,
                        std::mem::size_of::<Self>(),
                    );
                    src.read_exact(buffer)?;
                }
                Ok(value)
            }
        }
    };
}

serializable_default!(u8);
serializable_default!(u16);
serializable_default!(u32);
serializable_default!(u64);

macro_rules! serializable_u8_array {
    ($sz: literal) => {
        impl Serialize for [u8; $sz] {
            fn serialize<W: io::Write>(&self, mut dst: W) -> io::Result<usize> {
                dst.write(self)
            }
            fn deserialize<R: io::Read>(mut src: R) -> io::Result<Self> {
                let mut value: Self = Self::default();
                src.read_exact(&mut value)?;
                Ok(value)
            }
        }
    };
}

serializable_u8_array!(1);
serializable_u8_array!(2);
serializable_u8_array!(4);
serializable_u8_array!(8);
