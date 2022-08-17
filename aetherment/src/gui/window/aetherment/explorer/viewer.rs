use std::{path::PathBuf, fs::File, io::Write};
use serde_json::json;
use crate::apply::{penumbra::PenumbraFile, Datas};

mod generic;
mod options;
mod tex;
mod mtrl;

pub use generic::*;
pub use options::*;
pub use tex::*;
pub use mtrl::*;

pub struct Conf<'a> {
	pub path: PathBuf,
	pub datas: &'a mut Datas,
	pub option: &'a mut String,
	pub sub_option: &'a mut String,
}

impl <'a> Conf<'a> {
	pub fn save(&self) {
		File::create(&self.path.join("datas.json")).unwrap().write_all(crate::serialize_json(json!(self.datas)).as_bytes()).unwrap();
	}
	
	pub fn file_mut<'b>(&'b mut self, path: &str) -> Option<&'b mut PenumbraFile> {
		self.datas.penumbra.as_mut().unwrap().file_mut(self.option, self.sub_option, path)
	}
	
	pub fn file_ref<'b>(&'b self, path: &str) -> Option<&'b PenumbraFile> {
		self.datas.penumbra.as_ref().unwrap().file_ref(self.option, self.sub_option, path)
	}
}

pub trait Viewer {
	fn valid_imports(&self) -> Vec<String>;
	fn valid_exports(&self) -> Vec<String>;
	fn draw(&mut self, state: &mut crate::Data, conf: Option<Conf>);
	fn save(&self, ext: &str, writer: &mut Vec<u8>);
}