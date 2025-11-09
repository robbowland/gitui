use crate::{
	components::CommandInfo, keys::SharedKeyConfig, strings,
	ui::style::SharedTheme,
};
use ratatui::{
	layout::{Alignment, Rect},
	text::{Line, Span},
	widgets::Paragraph,
	Frame,
};
use std::borrow::Cow;
use unicode_width::UnicodeWidthStr;

enum DrawListEntry {
	LineBreak,
	Splitter,
	Command(Command),
}

struct Command {
	txt: String,
	enabled: bool,
}

/// helper to be used while drawing
pub struct CommandBar {
	draw_list: Vec<DrawListEntry>,
	cmd_infos: Vec<CommandInfo>,
	theme: SharedTheme,
	key_config: SharedKeyConfig,
	lines: u16,
	width: u16,
	has_entries: bool,
	expanded: bool,
}

impl CommandBar {
	pub const fn new(
		theme: SharedTheme,
		key_config: SharedKeyConfig,
	) -> Self {
		Self {
			draw_list: Vec::new(),
			cmd_infos: Vec::new(),
			theme,
			key_config,
			lines: 0,
			width: 0,
			has_entries: false,
			expanded: false,
		}
	}

	pub fn refresh_width(&mut self, width: u16) {
		if width != self.width {
			self.refresh_list(width);
			self.width = width;
		}
	}

	fn refresh_list(&mut self, width: u16) {
		self.draw_list.clear();

		let mut line_width = 0_usize;
		let mut lines = 1_u16;
		let mut has_entries = false;

		for c in &self.cmd_infos {
			has_entries = true;
			let entry_w =
				UnicodeWidthStr::width(c.text.name.as_str());

			if line_width + entry_w > width as usize {
				self.draw_list.push(DrawListEntry::LineBreak);
				line_width = 0;
				lines += 1;
			} else if line_width > 0 {
				self.draw_list.push(DrawListEntry::Splitter);
			}

			line_width += entry_w + 1;

			self.draw_list.push(DrawListEntry::Command(Command {
				txt: c.text.name.clone(),
				enabled: c.enabled,
			}));
		}

		if !has_entries {
			lines = 0;
		}

		self.has_entries = has_entries;
		self.lines = lines;

		if !self.has_entries {
			self.expanded = false;
		}
	}

	pub fn set_cmds(&mut self, cmds: Vec<CommandInfo>) {
		self.cmd_infos = cmds
			.into_iter()
			.filter(CommandInfo::show_in_quickbar)
			.collect::<Vec<_>>();
		self.cmd_infos.sort_by_key(|e| e.order);
		self.refresh_list(self.width);
	}

	pub const fn height(&self) -> u16 {
		if self.expanded && self.has_entries {
			self.lines
		} else {
			0_u16
		}
	}

	pub fn toggle_more(&mut self) {
		if self.has_entries {
			self.expanded = !self.expanded;
		}
	}

	pub fn draw(&self, f: &mut Frame, r: Rect) {
		if r.width == 0 || r.height == 0 || !self.expanded {
			return;
		}

		if !self.has_entries {
			return;
		}
		let splitter = Span::raw(Cow::from(strings::cmd_splitter(
			&self.key_config,
		)));

		let texts = self
			.draw_list
			.split(|c| matches!(c, DrawListEntry::LineBreak))
			.map(|c_arr| {
				Line::from(
					c_arr
						.iter()
						.map(|c| match c {
							DrawListEntry::Command(c) => {
								Span::styled(
									Cow::from(c.txt.as_str()),
									self.theme.commandbar(c.enabled),
								)
							}
							DrawListEntry::LineBreak => {
								// Doesn't exist in split array
								Span::raw("")
							}
							DrawListEntry::Splitter => {
								splitter.clone()
							}
						})
						.collect::<Vec<Span>>(),
				)
			})
			.collect::<Vec<Line>>();

		f.render_widget(
			Paragraph::new(texts).alignment(Alignment::Left),
			r,
		);
	}
}
