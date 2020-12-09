use std::io;

pub trait Serialize: Sized {
    fn serialize<W: io::Write>(&self, dst: W) -> io::Result<usize>;
    fn deserialize<R: io::Read>(src: R) -> io::Result<Self>;
}

macro_rules! declare_serializable_integer {
    ($t: ty) => {
        impl Serialize for $t {
            fn serialize<W: io::Write>(&self, mut dst: W) -> io::Result<usize> {
                dst.write(&self.to_le_bytes())
            }
            fn deserialize<R: io::Read>(mut src: R) -> io::Result<Self> {
                let mut value: Self = 0;
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

declare_serializable_integer!(u8);
declare_serializable_integer!(u16);
declare_serializable_integer!(u32);
declare_serializable_integer!(u64);
