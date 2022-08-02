use std::time::Instant;
use crate::gui::aeth::{self, F2, DrawList};

mod settings;
mod manager;
mod browser;
mod explorer;
mod creator;

pub struct Window {
	pub visible: bool,
	pub settings: settings::Tab,
	pub manager: manager::Tab,
	pub browser: browser::Tab,
	pub explorer: explorer::Tab,
	pub creator: creator::Tab,
	pub login: Option<(String, Instant, Instant)>,
}

impl Window {
	pub fn new(state: &mut crate::Data) -> Self {
		Window {
			visible: true,
			settings: settings::Tab::new(state),
			manager: manager::Tab::new(state),
			browser: browser::Tab::new(state),
			explorer: explorer::Tab::new(state),
			creator: creator::Tab::new(state),
			login: None,
		}
	}
	
	pub fn draw(&mut self, state: &mut crate::Data) -> anyhow::Result<()> {
		if self.login.is_some() {
			self.draw_login(state);
			return Ok(());
		}
		
		let namesize = imgui::calc_text_size(if let Some(user) = &state.user {user.name.as_ref()} else {"Login"}, false, -1.0).x();
		let spacing = imgui::get_style().item_spacing.x() + imgui::get_style().frame_padding.x();
		let height = aeth::frame_height();
		let w = imgui::get_column_width(-1)
			- namesize
			- height * 2.0
			- spacing * 2.0;
		
		imgui::set_next_item_width(w);
		aeth::tab_bar("tabs")
			.tab("Settings", || {
				aeth::offset([0.0, -imgui::get_style().item_spacing[1]]);
				imgui::set_next_item_width(w);
				self.settings.draw(state)
			})
			.tab("Mod Manager", ||{
				self.manager.draw(state);
			})
			.tab("Mod Browser", || {
				aeth::offset([0.0, -imgui::get_style().item_spacing[1]]);
				imgui::set_next_item_width(w);
				self.browser.draw(state);
			})
			.condition(state.config.tab_explorer).tab("File Explorer", || {
				self.explorer.draw(state);
			})
			.condition(state.config.tab_moddev).tab("Mod Creator", || {
				self.creator.draw(state);
			})
			.finish();
		
		// User segment
		let (name, avatar) = if let Some(user) = &state.user {
			(user.name.as_ref(), Some(&user.avatar))
		} else {
			("Login", None)
		};
		
		imgui::same_line();
		let posb = imgui::get_cursor_screen_pos().sub([spacing, 0.0]);
		if imgui::button(name, [0.0, 0.0]) && state.user.is_none() { // TODO: profile page if logged in?
			use rand::Rng;
			let state = rand::thread_rng()
				.sample_iter(rand::distributions::Alphanumeric)
				.take(32)
				.map(char::from)
				.collect::<String>();
			
			open::that(format!("{}/login?app_state={}", crate::SERVER, state)).unwrap();
			self.login = Some((state, Instant::now(), Instant::now()));
		}
		if state.user.is_some() && imgui::is_item_clicked(imgui::MouseButton::Right) {imgui::open_popup("profile_context", imgui::PopupFlags::None)}
		imgui::same_line();
		let mut draw = imgui::get_window_draw_list();
		let pos = imgui::get_cursor_screen_pos();
		let clra = imgui::get_color(imgui::Col::TabActive);
		let rounding = if imgui::get_style().tab_rounding > 0.0 {height} else {0.0};
		draw.add_line(posb.add([0.0, height]), pos.add([0.0, height]), clra, 1.0);
		
		if let Some(avatar) = avatar {
			// draw.add_image_rounded(avatar.resource(), pos, pos.add([height * 2.0; 2]), [0.0; 2], [1.0; 2], 0xFFFFFFFF, rounding, imgui::DrawFlags::RoundCornersAll)
			draw.push_texture_id(avatar.resource());
			draw.add_rect_rounded(pos, pos.add([height * 2.0; 2]), [0.0; 2], [1.0; 2], 0xFFFFFFFF, rounding);
			draw.pop_texture_id();
		} else {
			draw.add_rect_filled(pos, pos.add([height * 2.0; 2]), 0xFF000000, rounding, imgui::DrawFlags::RoundCornersAll)
		}
		
		draw.add_rect(pos, pos.add([height * 2.0; 2]), clra, rounding, imgui::DrawFlags::RoundCornersAll, 1.0);
		
		if imgui::begin_popup("profile_context", imgui::WindowFlags::None) {
			if imgui::selectable("Logout", false, imgui::SelectableFlags::None, [0.0, 0.0]) {
				state.user.as_ref().unwrap().delete().unwrap();
				state.user = None;
			}
			imgui::end_popup();
		}
		
		Ok(())
	}
	
	fn draw_login(&mut self, state: &mut crate::Data) {
		let (app_state, start, last) = self.login.as_mut().unwrap();
		let now = Instant::now();
		let dur = now.duration_since(*start).as_secs_f32();
		if dur >= 60.0 {
			imgui::text("Login took too long");
			if imgui::button("Close", [0.0, 0.0]) {
				self.login = None;
			}
			
			return;
		}
		
		imgui::text("The login page has been opened in your default web browser");
		imgui::text(&format!("{:.0}", 60.0 - dur));
		if imgui::button("Cancel", [0.0, 0.0]) {
			self.login = None;
			return;
		}
		if now.duration_since(*last).as_secs_f32() >= 3.0 {
			*last = now;
			let resp: serde_json::Value = crate::CLIENT.get(format!("{}/login/app?app_state={}", crate::SERVER, app_state))
				.send()
				.unwrap()
				.json()
				.unwrap();
			
			// log!("{}", crate::serialize_json(resp.clone()));
			
			if resp.get("error").is_none() {
				#[derive(serde::Deserialize)]
				pub struct User {
					pub id: i32,
					pub name: String,
					pub token: String,
				}
				
				let user: User = serde_json::from_value(resp).unwrap();
				let user = crate::server::user::User::new(user.id, user.name, user.token);
				user.store().unwrap();
				state.user = Some(user);
				self.login = None;
			}
		}
	}
}