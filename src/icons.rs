use once_cell::sync::Lazy;
use ratatui::style::Color;
use serde::Deserialize;
use std::{collections::HashMap, path::Path};

const DEFAULT_FILE_ICON: &str = "";
const DEFAULT_CLOSED_FOLDER_ICON: &str = "";
const DEFAULT_OPEN_FOLDER_ICON: &str = "";

pub(crate) struct Icon<'a> {
	pub glyph: &'a str,
	pub color: Option<Color>,
}

static ICONS: Lazy<IconStore> = Lazy::new(IconStore::load);

pub(crate) fn file_icon(path: &Path) -> Icon<'static> {
	ICONS.file_icon(path)
}

pub(crate) fn folder_icon(name: &str, open: bool) -> Icon<'static> {
	ICONS.folder_icon(name, open)
}

struct IconStore {
	file: IconEntry,
	folder: IconEntry,
	dirs: HashMap<String, IconEntry>,
	files: HashMap<String, IconEntry>,
	exts: HashMap<String, IconEntry>,
}

impl IconStore {
	fn load() -> Self {
		let raw: RawTheme = toml::from_str(include_str!(
			"../assets/yazi-icon-theme.toml"
		))
		.expect("failed to parse Yazi icon theme");
		let RawTheme { cmp, icon } = raw;
		let file_default =
			icon.cond_icon("!dir").unwrap_or_else(|| {
				IconEntry::new(
					cmp.icon_file.unwrap_or_else(|| {
						DEFAULT_FILE_ICON.to_string()
					}),
					None,
				)
			});
		let folder_default = icon
			.cond_icon("dir")
			.unwrap_or_else(|| {
				IconEntry::new(
					cmp.icon_folder.unwrap_or_else(|| {
						DEFAULT_CLOSED_FOLDER_ICON.to_string()
					}),
					None,
				)
			})
			.with_open_glyph(DEFAULT_OPEN_FOLDER_ICON.to_string());
		let RawIconSection {
			dirs, files, exts, ..
		} = icon;

		Self {
			file: file_default,
			folder: folder_default,
			dirs: Self::build_icon_map(dirs),
			files: Self::build_icon_map(files),
			exts: Self::build_icon_map(exts),
		}
	}

	fn build_icon_map(
		entries: Vec<RawEntry>,
	) -> HashMap<String, IconEntry> {
		let mut map = HashMap::with_capacity(entries.len() * 2);
		for entry in entries {
			let icon = IconEntry::new(
				entry.text,
				parse_color(entry.fg.as_deref()),
			);
			Self::insert_icon(&mut map, entry.name, icon);
		}
		map
	}

	fn folder_icon<'a>(&'a self, name: &str, open: bool) -> Icon<'a> {
		let name = Self::basename(name);
		let entry = self
			.lookup_name(&self.dirs, name)
			.unwrap_or(&self.folder);
		if open {
			entry.as_open_icon()
		} else {
			entry.as_icon()
		}
	}

	fn file_icon<'a>(&'a self, path: &Path) -> Icon<'a> {
		let name = path
			.file_name()
			.and_then(std::ffi::OsStr::to_str)
			.unwrap_or_default();

		if let Some(entry) = self.lookup_name(&self.files, name) {
			return entry.as_icon();
		}

		if let Some(entry) = self.lookup_by_suffix(name) {
			return entry.as_icon();
		}

		self.lookup_extension(path).unwrap_or(&self.file).as_icon()
	}

	fn lookup_extension<'a>(
		&'a self,
		path: &Path,
	) -> Option<&'a IconEntry> {
		let ext = path
			.extension()
			.and_then(std::ffi::OsStr::to_str)
			.unwrap_or_default();
		self.lookup_name(&self.exts, ext)
	}

	fn lookup_by_suffix<'a>(
		&'a self,
		name: &str,
	) -> Option<&'a IconEntry> {
		for (idx, ch) in name.char_indices() {
			if ch != '.' {
				continue;
			}

			if idx + 1 >= name.len() {
				continue;
			}

			let ext = &name[idx + 1..];
			if let Some(entry) = self.lookup_name(&self.exts, ext) {
				return Some(entry);
			}
		}

		None
	}

	fn lookup_name<'a>(
		&'a self,
		map: &'a HashMap<String, IconEntry>,
		name: &str,
	) -> Option<&'a IconEntry> {
		if name.is_empty() {
			return None;
		}

		map.get(name)
			.or_else(|| map.get(&name.to_ascii_lowercase()))
	}

	fn basename(path: &str) -> &str {
		path.rsplit(['/', '\\']).next().unwrap_or(path)
	}

	fn insert_icon(
		map: &mut HashMap<String, IconEntry>,
		name: String,
		icon: IconEntry,
	) {
		let lower = name.to_ascii_lowercase();
		let same = lower == name;
		map.insert(name, icon.clone());
		if !same {
			map.entry(lower).or_insert(icon);
		}
	}
}

#[derive(Clone)]
struct IconEntry {
	glyph: String,
	color: Option<Color>,
	open_glyph: Option<String>,
}

impl IconEntry {
	fn new(glyph: String, color: Option<Color>) -> Self {
		Self {
			glyph,
			color,
			open_glyph: None,
		}
	}

	fn with_open_glyph(mut self, glyph: String) -> Self {
		self.open_glyph = Some(glyph);
		self
	}

	fn as_icon(&self) -> Icon<'_> {
		Icon {
			glyph: self.glyph.as_str(),
			color: self.color,
		}
	}

	fn as_open_icon(&self) -> Icon<'_> {
		let glyph = self
			.open_glyph
			.as_deref()
			.unwrap_or(DEFAULT_OPEN_FOLDER_ICON);
		Icon {
			glyph,
			color: self.color,
		}
	}
}

#[derive(Deserialize, Default)]
struct RawTheme {
	#[serde(default)]
	cmp: RawCmp,
	#[serde(default)]
	icon: RawIconSection,
}

#[derive(Deserialize, Default)]
struct RawCmp {
	#[serde(default)]
	icon_file: Option<String>,
	#[serde(default)]
	icon_folder: Option<String>,
}

#[derive(Deserialize, Default)]
struct RawIconSection {
	#[serde(default)]
	dirs: Vec<RawEntry>,
	#[serde(default)]
	files: Vec<RawEntry>,
	#[serde(default)]
	exts: Vec<RawEntry>,
	#[serde(default)]
	conds: Vec<RawCondEntry>,
}

impl RawIconSection {
	fn cond_icon(&self, cond: &str) -> Option<IconEntry> {
		self.conds.iter().find(|entry| entry.condition == cond).map(
			|entry| {
				let icon = IconEntry::new(
					entry.text.clone(),
					parse_color(entry.fg.as_deref()),
				);
				if cond == "dir" {
					icon.with_open_glyph(
						DEFAULT_OPEN_FOLDER_ICON.to_string(),
					)
				} else {
					icon
				}
			},
		)
	}
}

#[derive(Deserialize)]
struct RawEntry {
	name: String,
	text: String,
	#[serde(default)]
	fg: Option<String>,
}

#[derive(Deserialize)]
struct RawCondEntry {
	#[serde(rename = "if")]
	condition: String,
	text: String,
	#[serde(default)]
	fg: Option<String>,
}

fn parse_color(value: Option<&str>) -> Option<Color> {
	let value = value?;
	let trimmed = value.trim();
	let hex = trimmed.strip_prefix('#').unwrap_or(trimmed);
	if hex.len() != 6 {
		return None;
	}

	let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
	let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
	let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
	Some(Color::Rgb(r, g, b))
}
