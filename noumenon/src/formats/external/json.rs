use std::io::{Read, Seek, Write};

pub trait Json {
	fn read<T>(reader: &mut T) -> Self where T: Read + Seek;
	fn write<T>(&self, writer: &mut T) where T: Write + Seek;
}