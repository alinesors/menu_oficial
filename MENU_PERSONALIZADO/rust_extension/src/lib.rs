use godot::prelude::*;
use godot::classes::{
	CanvasItem, CanvasLayer, Control, Engine, ICanvasLayer, IControl, IRefCounted,
	Input, InputEvent, Os,
	InputEventMouseButton, Label, Mesh, Node, ResourceLoader, ScrollContainer, Timer,
	RichTextLabel, Texture2D, TextureRect, Time, RefCounted, Object,
};
use godot::classes::input::MouseMode;
use godot::global::{Error, MouseButton};

struct MenuUiRustExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MenuUiRustExtension {}

#[derive(GodotClass)]
#[class(base=CanvasLayer)]
struct PauseMenuLayerRust {
	base: Base<CanvasLayer>,
}

#[godot_api]
impl ICanvasLayer for PauseMenuLayerRust {
	fn init(base: Base<CanvasLayer>) -> Self {
		Self { base }
	}

	fn ready(&mut self) {
		let callable = self.base().callable("_on_visibility_changed");
		let mut base = self.base_mut();
		base.connect("visibility_changed", &callable);
	}
}

#[godot_api]
impl PauseMenuLayerRust {
	#[func]
	fn _on_pause_menu_hidden(&mut self) {
		self.base_mut().hide();
	}

	#[func]
	fn _on_visibility_changed(&mut self) {
		if !self.base().is_visible() {
			return;
		}
		let base = self.base_mut();
		if base.has_node("PauseMenu") {
			let mut pause_menu = base.get_node_as::<CanvasItem>("PauseMenu");
			pause_menu.show();
		}
	}
}

#[derive(GodotClass)]
#[class(base=Control)]
struct TelaInicialRust {
	#[export]
	main_menu_scene_path: GString,
	#[export]
	background_image_path: GString,
	#[export]
	prompt_text: GString,
	transitioning: bool,
	blink_time: f64,
	base: Base<Control>,
}

#[godot_api]
impl IControl for TelaInicialRust {
	fn init(base: Base<Control>) -> Self {
		Self {
			main_menu_scene_path: GString::new(),
			background_image_path: GString::from("res://assets/menu_custom/opening_bg_01.png"),
			prompt_text: GString::from("Clique na tela para iniciar"),
			transitioning: false,
			blink_time: 0.0,
			base,
		}
	}

	fn ready(&mut self) {
		let background_path = self.background_image_path.clone();
		let texture = self.load_background_texture(&background_path);
		if let Some(mut background) = self.get_background_rect() {
			if let Some(tex) = texture {
				background.set_texture(&tex);
			} else {
				godot_warn!("Unable to load start screen background image.");
			}
		}
		if let Some(mut label) = self.get_prompt_label() {
			label.set_text(&self.prompt_text);
		}
	}

	fn process(&mut self, delta: f64) {
		self.blink_time += delta;
		let alpha = 0.42 + 0.58 * (0.5 + 0.5 * (self.blink_time * 2.8).sin());
		if let Some(mut label) = self.get_prompt_label() {
			let mut modulate = label.get_modulate();
			modulate.a = alpha as f32;
			label.set_modulate(modulate);
		}
	}

	fn unhandled_input(&mut self, event: Gd<InputEvent>) {
		if let Ok(mouse_event) = event.try_cast::<InputEventMouseButton>() {
			if mouse_event.is_pressed() && mouse_event.get_button_index() == MouseButton::LEFT {
				self.go_to_menu();
			}
		}
	}
}

#[godot_api]
impl TelaInicialRust {
	const DEFAULT_MAIN_MENU_SCENE: &'static str = "res://scenes/menus/main_menu/main_menu_with_animations.tscn";
	const DEFAULT_BACKGROUND_IMAGE: &'static str = "res://assets/menu_custom/opening_bg_01.png";

	fn get_background_rect(&mut self) -> Option<Gd<TextureRect>> {
		let base = self.base_mut();
		if !base.has_node("BackgroundTextureRect") {
			return None;
		}
		Some(base.get_node_as::<TextureRect>("BackgroundTextureRect"))
	}

	fn get_prompt_label(&mut self) -> Option<Gd<Label>> {
		let base = self.base_mut();
		if !base.has_node("PromptLabel") {
			return None;
		}
		Some(base.get_node_as::<Label>("PromptLabel"))
	}

	fn resolve_main_menu_scene_path(&self) -> GString {
		if !self.main_menu_scene_path.is_empty() {
			return self.main_menu_scene_path.clone();
		}
		let tree = self.base().get_tree();
		if let Some(tree) = tree {
			let root = tree.get_root();
			if let Some(root) = root {
				if root.has_node("AppConfig") {
					let app_config = root.get_node_as::<Node>("AppConfig");
					return app_config.get("main_menu_scene_path").to::<GString>();
				}
			}
		}
		GString::from(Self::DEFAULT_MAIN_MENU_SCENE)
	}

	fn load_background_texture(&mut self, path: &GString) -> Option<Gd<Texture2D>> {
		let normalized = path.strip_edges(true, true);
		let fallback = GString::from(Self::DEFAULT_BACKGROUND_IMAGE);
		let mut loader = ResourceLoader::singleton();
		if !normalized.is_empty() {
			if let Some(res) = loader.load(&normalized) {
				if let Ok(texture) = res.try_cast::<Texture2D>() {
					return Some(texture);
				}
			}
		}
		if normalized != fallback {
			if let Some(res) = loader.load(&fallback) {
				if let Ok(texture) = res.try_cast::<Texture2D>() {
					return Some(texture);
				}
			}
		}
		None
	}

	fn go_to_menu(&mut self) {
		if self.transitioning {
			return;
		}
		let target = self.resolve_main_menu_scene_path();
		if target.is_empty() {
			godot_warn!("main_menu_scene_path is empty.");
			return;
		}
		self.transitioning = true;
		if let Some(tree) = self.base().get_tree() {
			if let Some(root) = tree.get_root() {
				if root.has_node("SceneLoader") {
					let mut loader = root.get_node_as::<Node>("SceneLoader");
					loader.call("load_scene", &[target.to_variant()]);
					return;
				}
			}
		}
		if let Some(mut tree) = self.base().get_tree() {
			let result = tree.change_scene_to_file(&target);
			if result != Error::OK {
				godot_warn!("Failed to change scene to target.");
				self.transitioning = false;
			}
		}
	}
}

#[derive(GodotClass)]
#[class(base=Control, tool)]
struct ScrollableCreditsRust {
	#[export]
	input_scroll_speed: f64,
	line_number: f64,
	base: Base<Control>,
}

#[godot_api]
impl IControl for ScrollableCreditsRust {
	fn init(base: Base<Control>) -> Self {
		Self {
			input_scroll_speed: 10.0,
			line_number: 0.0,
			base,
		}
	}

	fn ready(&mut self) {
		let callable = self.base().callable("_on_visibility_changed");
		let mut base = self.base_mut();
		base.connect("visibility_changed", &callable);
	}

	fn process(&mut self, delta: f64) {
		if Engine::singleton().is_editor_hint() || !self.base().is_visible() {
			return;
		}
		let input_axis = Input::singleton().get_axis("ui_up", "ui_down") as f64;
		if input_axis.abs() <= 0.5 {
			return;
		}
		self.line_number += input_axis * delta * self.input_scroll_speed;
		if let Some(mut credits_label) = self.get_credits_label() {
			let max_lines =
				(credits_label.get_line_count() - credits_label.get_visible_line_count()) as f64;
			if self.line_number < 0.0 {
				self.line_number = 0.0;
			}
			if self.line_number > max_lines {
				self.line_number = max_lines;
			}
			credits_label.scroll_to_line(self.line_number.round() as i32);
		}
	}
}

#[godot_api]
impl ScrollableCreditsRust {
	fn get_credits_label(&mut self) -> Option<Gd<RichTextLabel>> {
		let base = self.base_mut();
		if !base.has_node("CreditsLabel") {
			return None;
		}
		Some(base.get_node_as::<RichTextLabel>("CreditsLabel"))
	}

	#[func]
	fn _on_visibility_changed(&mut self) {
		if !self.base().is_visible() {
			return;
		}
		self.line_number = 0.0;
		if let Some(mut credits_label) = self.get_credits_label() {
			credits_label.scroll_to_line(0);
			credits_label.grab_focus();
		}
	}
}

#[derive(GodotClass)]
#[class(base=Control, tool)]
struct EndCreditsRust {
	#[export]
	main_menu_scene_path: GString,
	#[export]
	force_mouse_mode_visible: bool,
	#[export]
	auto_scroll_speed: f64,
	#[export]
	input_scroll_speed: f64,
	#[export]
	scroll_restart_delay: f64,
	#[export]
	scroll_paused: bool,
	current_scroll_position: f64,
	timer: Option<Gd<Timer>>,
	base: Base<Control>,
}

#[godot_api]
impl IControl for EndCreditsRust {
	fn init(base: Base<Control>) -> Self {
		Self {
			main_menu_scene_path: GString::new(),
			force_mouse_mode_visible: false,
			auto_scroll_speed: 60.0,
			input_scroll_speed: 400.0,
			scroll_restart_delay: 1.5,
			scroll_paused: false,
			current_scroll_position: 0.0,
			timer: None,
			base,
		}
	}

	fn ready(&mut self) {
		if self.resolve_main_menu_scene_path().is_empty() {
			if let Some(mut menu_button) = self.get_node_control("MenuButton") {
				menu_button.hide();
			}
		}
		if Engine::singleton().has_singleton("JavaScriptBridge") {
			if let Some(mut exit_button) = self.get_node_control("ExitButton") {
				exit_button.hide();
			}
		}
		if let Some(mut end_panel) = self.get_node_control("EndMessagePanel") {
			end_panel.hide();
		}

		self.set_header_and_footer();
		self.scroll_paused = false;

		let callable_resize = self.base().callable("_on_resized");
		let callable_visibility = self.base().callable("_on_visibility_changed");
		let callable_input = self.base().callable("_on_gui_input");
		{
			let mut base = self.base_mut();
			base.connect("resized", &callable_resize);
			base.connect("visibility_changed", &callable_visibility);
			base.connect("gui_input", &callable_input);
		}

		if let Some(mut scroll) = self.get_scroll_container() {
			let callable_scroll_started = self.base().callable("_on_scroll_started");
			scroll.connect("scroll_started", &callable_scroll_started);
		}

		let mut timer = Timer::new_alloc();
		let callable_timeout = self.base().callable("_on_scroll_restart_timer_timeout");
		timer.connect("timeout", &callable_timeout);
		self.base_mut().add_child(&timer);
		self.timer = Some(timer);
	}

	fn process(&mut self, delta: f64) {
		let input_axis = Input::singleton().get_axis("ui_up", "ui_down") as f64;
		if input_axis != 0.0 {
			self.scroll_container(input_axis * self.input_scroll_speed * delta);
		} else {
			self.scroll_container(self.auto_scroll_speed * delta);
		}
	}

	fn exit_tree(&mut self) {
		if let Some(scroll) = self.get_scroll_container() {
			self.current_scroll_position = scroll.get("scroll_vertical").to::<i64>() as f64;
		}
	}

	fn unhandled_input(&mut self, event: Gd<InputEvent>) {
		if !event.is_action_released("ui_cancel") {
			return;
		}
		let visible = self
			.get_node_control("EndMessagePanel")
			.map(|panel| panel.is_visible())
			.unwrap_or(false);
		if !visible {
			self.end_reached();
		} else {
			self.exit_game();
		}
	}
}

#[godot_api]
impl EndCreditsRust {
	fn resolve_main_menu_scene_path(&self) -> GString {
		if !self.main_menu_scene_path.is_empty() {
			return self.main_menu_scene_path.clone();
		}
		let tree = self.base().get_tree();
		if let Some(tree) = tree {
			let root = tree.get_root();
			if let Some(root) = root {
				if root.has_node("AppConfig") {
					let app_config = root.get_node_as::<Node>("AppConfig");
					return app_config.get("main_menu_scene_path").to::<GString>();
				}
			}
		}
		GString::new()
	}

	fn get_node_control(&mut self, path: &str) -> Option<Gd<Control>> {
		let base = self.base_mut();
		if !base.has_node(path) {
			return None;
		}
		Some(base.get_node_as::<Control>(path))
	}

	fn get_scroll_container(&mut self) -> Option<Gd<ScrollContainer>> {
		let base = self.base_mut();
		if !base.has_node("ScrollContainer") {
			return None;
		}
		Some(base.get_node_as::<ScrollContainer>("ScrollContainer"))
	}

	fn set_header_and_footer(&mut self) {
		let size = self.base().get_size();
		if let Some(mut header) = self.get_node_control("HeaderSpace") {
			let mut min = header.get_custom_minimum_size();
			min.y = size.y;
			header.set_custom_minimum_size(min);
		}
		if let Some(mut footer) = self.get_node_control("FooterSpace") {
			let mut min = footer.get_custom_minimum_size();
			min.y = size.y;
			footer.set_custom_minimum_size(min);
		}
		if let Some(mut credits_label) = self.get_node_control("CreditsLabel") {
			let mut min = credits_label.get_custom_minimum_size();
			min.x = size.x;
			credits_label.set_custom_minimum_size(min);
		}
	}

	fn is_end_reached(&mut self) -> bool {
		let credits_h = self
			.get_node_control("CreditsLabel")
			.map(|n| n.get_size().y)
			.unwrap_or(0.0);
		let header_h = self
			.get_node_control("HeaderSpace")
			.map(|n| n.get_size().y)
			.unwrap_or(0.0);
		let end_vertical = credits_h + header_h;
		if let Some(scroll) = self.get_scroll_container() {
			return (scroll.get("scroll_vertical").to::<i64>() as f32) > end_vertical;
		}
		false
	}

	fn check_end_reached(&mut self) {
		if self.is_end_reached() {
			self.end_reached();
		}
	}

	fn scroll_container(&mut self, amount: f64) {
		if !self.base().is_visible() || self.scroll_paused {
			return;
		}
		self.current_scroll_position += amount;
		if let Some(mut scroll) = self.get_scroll_container() {
			scroll.set("scroll_vertical", &(self.current_scroll_position.round() as i64).to_variant());
		}
		self.check_end_reached();
	}

	fn start_scroll_restart_timer(&mut self) {
		if let Some(mut timer) = self.timer.clone() {
			timer.start_ex().time_sec(self.scroll_restart_delay).done();
		}
	}

	fn end_reached(&mut self) {
		self.scroll_paused = true;
		if let Some(mut end_panel) = self.get_node_control("EndMessagePanel") {
			end_panel.show();
		}
		if self.force_mouse_mode_visible {
			Input::singleton().set_mouse_mode(MouseMode::VISIBLE);
		}
	}

	fn load_main_menu(&mut self) {
		let target = self.resolve_main_menu_scene_path();
		if target.is_empty() {
			return;
		}
		let tree = self.base().get_tree();
		if let Some(tree) = tree {
			let root = tree.get_root();
			if let Some(root) = root {
				if root.has_node("SceneLoader") {
					let mut loader = root.get_node_as::<Node>("SceneLoader");
					loader.call("load_scene", &[target.to_variant()]);
				}
			}
		}
	}

	fn exit_game(&mut self) {
		if Engine::singleton().has_singleton("JavaScriptBridge") {
			self.load_main_menu();
			return;
		}
		if let Some(mut tree) = self.base().get_tree() {
			tree.quit_ex().done();
		}
	}

	#[func]
	fn _on_resized(&mut self) {
		self.set_header_and_footer();
		if let Some(scroll) = self.get_scroll_container() {
			self.current_scroll_position = scroll.get("scroll_vertical").to::<i64>() as f64;
		}
	}

	#[func]
	fn _on_gui_input(&mut self, event: Gd<InputEvent>) {
		if event.try_cast::<InputEventMouseButton>().is_ok() {
			self.scroll_paused = true;
			self.start_scroll_restart_timer();
		}
		self.check_end_reached();
	}

	#[func]
	fn _on_scroll_started(&mut self) {
		self.scroll_paused = true;
		self.start_scroll_restart_timer();
	}

	#[func]
	fn _on_scroll_restart_timer_timeout(&mut self) {
		if let Some(scroll) = self.get_scroll_container() {
			self.current_scroll_position = scroll.get("scroll_vertical").to::<i64>() as f64;
		}
		self.scroll_paused = false;
	}

	#[func]
	fn _on_visibility_changed(&mut self) {
		if !self.base().is_visible() {
			return;
		}
		if let Some(mut end_panel) = self.get_node_control("EndMessagePanel") {
			end_panel.hide();
		}
		if let Some(mut scroll) = self.get_scroll_container() {
			scroll.set("scroll_vertical", &0_i64.to_variant());
			self.current_scroll_position = 0.0;
		}
		self.scroll_paused = false;
	}

	#[func]
	fn _on_exit_button_pressed(&mut self) {
		self.exit_game();
	}

	#[func]
	fn _on_menu_button_pressed(&mut self) {
		self.load_main_menu();
	}
}

#[derive(GodotClass)]
#[class(base=Control, tool)]
struct ScrollingCreditsRust {
	#[export]
	auto_scroll_speed: f64,
	#[export]
	input_scroll_speed: f64,
	#[export]
	scroll_restart_delay: f64,
	#[export]
	scroll_paused: bool,
	current_scroll_position: f64,
	timer: Option<Gd<Timer>>,
	base: Base<Control>,
}

#[derive(GodotClass)]
#[class(base=CanvasLayer)]
struct LoadingScreenRust {
	#[export]
	state_change_delay: f64,
	#[export]
	progress_lerp_speed: f64,
	#[export]
	hold_after_complete_sec: f64,
	#[export]
	in_progress_text: GString,
	#[export]
	complete_text: GString,
	scene_loading_complete: bool,
	scene_loading_progress: f64,
	displayed_progress: f64,
	loading_start_time_ms: i64,
	complete_visible_since_ms: i64,
	base: Base<CanvasLayer>,
}

#[godot_api]
impl ICanvasLayer for LoadingScreenRust {
	fn init(base: Base<CanvasLayer>) -> Self {
		Self {
			state_change_delay: 15.0,
			progress_lerp_speed: 5.5,
			hold_after_complete_sec: 0.25,
			in_progress_text: GString::from("Loading..."),
			complete_text: GString::from("Loading Complete!"),
			scene_loading_complete: false,
			scene_loading_progress: 0.0,
			displayed_progress: 0.0,
			loading_start_time_ms: 0,
			complete_visible_since_ms: 0,
			base,
		}
	}

	fn ready(&mut self) {
		self.reset();
	}

	fn process(&mut self, delta: f64) {
		let status = self
			.get_scene_loader()
			.map(|mut sl| sl.call("get_status", &[]).to::<i64>())
			.unwrap_or(-1);

		if let Some(mut sl) = self.get_scene_loader() {
			let progress = sl.call("get_progress", &[]).to::<f64>();
			if progress > self.scene_loading_progress {
				self.scene_loading_progress = progress;
			}
		}

		// THREAD_LOAD_LOADED is 1 in Godot 4's threaded loader status enum.
		if status == 1 {
			self.scene_loading_complete = true;
			self.scene_loading_progress = 1.0;
		}

		let blend = (delta * self.progress_lerp_speed).clamp(0.0, 1.0);
		self.displayed_progress = self.displayed_progress + (self.scene_loading_progress - self.displayed_progress) * blend;
		if (self.scene_loading_progress - self.displayed_progress).abs() < 0.001 {
			self.displayed_progress = self.scene_loading_progress;
		}

		self.set_progress_value(self.displayed_progress);
		let percent = (self.displayed_progress * 100.0).round() as i64;
		if self.scene_loading_complete {
			if self.complete_visible_since_ms == 0 {
				self.complete_visible_since_ms = Time::singleton().get_ticks_msec() as i64;
			}
			self.set_progress_label(GString::from(format!("{} ({}%)", self.complete_text, percent)));
		} else {
			self.complete_visible_since_ms = 0;
			self.set_progress_label(GString::from(format!("{} ({}%)", self.in_progress_text, percent)));
		}
	}
}

#[godot_api]
impl LoadingScreenRust {
	fn get_scene_loader(&mut self) -> Option<Gd<Node>> {
		let tree = self.base().get_tree()?;
		let root = tree.get_root()?;
		if !root.has_node("SceneLoader") {
			return None;
		}
		Some(root.get_node_as::<Node>("SceneLoader"))
	}

	fn set_progress_value(&mut self, value: f64) {
		let base = self.base_mut();
		if !base.has_node("Control/VBoxContainer/ProgressBar") {
			return;
		}
		let mut bar = base.get_node_as::<Node>("Control/VBoxContainer/ProgressBar");
		bar.set("value", &value.to_variant());
	}

	fn set_progress_label(&mut self, text: GString) {
		let base = self.base_mut();
		if !base.has_node("Control/VBoxContainer/ProgressLabel") {
			return;
		}
		let mut label = base.get_node_as::<Node>("Control/VBoxContainer/ProgressLabel");
		label.set("text", &text.to_variant());
	}

	#[func]
	fn reset(&mut self) {
		self.scene_loading_complete = false;
		self.scene_loading_progress = 0.0;
		self.displayed_progress = 0.0;
		self.loading_start_time_ms = Time::singleton().get_ticks_msec() as i64;
		self.complete_visible_since_ms = 0;
		self.set_progress_value(0.0);
		self.base_mut().show();
		self.base_mut().set_process(true);
	}

	#[func]
	fn can_close_loading_screen(&self) -> bool {
		if !self.scene_loading_complete {
			return false;
		}
		if self.displayed_progress < 0.999 {
			return false;
		}
		if self.complete_visible_since_ms <= 0 {
			return false;
		}
		let now = Time::singleton().get_ticks_msec() as i64;
		let elapsed_ms = now - self.complete_visible_since_ms;
		elapsed_ms >= (self.hold_after_complete_sec * 1000.0) as i64
	}

	#[func]
	fn close(&mut self) {
		self.base_mut().set_process(false);
		self.base_mut().hide();
	}
}

#[derive(GodotClass)]
#[class(base=CanvasLayer)]
struct LoadingScreenWithShaderCachingRust {
	#[export]
	spatial_shader_material_dir: GString,
	#[export]
	cache_shaders_scene: GString,
	#[export]
	mesh: Option<Gd<Mesh>>,
	#[export]
	matching_extensions: PackedStringArray,
	#[export]
	ignore_subfolders: PackedStringArray,
	#[export]
	shader_delay_timer: f64,
	#[export]
	progress_lerp_speed: f64,
	#[export]
	hold_after_complete_sec: f64,
	scene_loading_complete: bool,
	scene_loading_progress: f64,
	caching_progress: f64,
	displayed_progress: f64,
	loading_shader_cache: bool,
	complete_visible_since_ms: i64,
	base: Base<CanvasLayer>,
}

#[derive(GodotClass)]
#[class(base=Control)]
struct MainMenuRust {
	#[export]
	game_scene_path_value: GString,
	#[export]
	options_packed_scene: Option<Gd<PackedScene>>,
	#[export]
	credits_packed_scene: Option<Gd<PackedScene>>,
	#[export]
	confirm_exit: bool,
	#[export]
	signal_game_start: bool,
	#[export]
	signal_game_exit: bool,
	sub_menu: Option<Gd<Control>>,
	base: Base<Control>,
}

#[derive(GodotClass)]
#[class(base=Control)]
struct MainMenuWithAnimationsRust {
	#[export]
	background_image_path: GString,
	base: Base<Control>,
}

#[godot_api]
impl IControl for MainMenuWithAnimationsRust {
	fn init(base: Base<Control>) -> Self {
		Self {
			background_image_path: GString::from("res://assets/menu_custom/opening_bg_01.png"),
			base,
		}
	}

	fn ready(&mut self) {
		let background_path = self.background_image_path.clone();
		if let Some(mut new_game_button) = self.get_node("MenuContainer/MenuButtonsMargin/MenuButtonsContainer/MenuButtonsBoxContainer/NewGameButton") {
			new_game_button.set("visible", &true.to_variant());
			new_game_button.set("text", &GString::from("Jogar").to_variant());
		}

		if let Some(mut background_rect) = self.get_node("BackgroundTextureRect") {
			if let Some(texture) = self.load_texture(&background_path) {
				background_rect.set("texture", &texture.to_variant());
			}
			background_rect.set("expand_mode", &1_i64.to_variant());
			background_rect.set("stretch_mode", &6_i64.to_variant());
		}

		if let Some(mut title_margin) = self.get_node("MenuContainer/TitleMargin") {
			title_margin.set("visible", &false.to_variant());
		}
		if let Some(mut subtitle_margin) = self.get_node("MenuContainer/SubTitleMargin") {
			subtitle_margin.set("visible", &false.to_variant());
		}
		if let Some(mut buttons_margin) = self.get_node("MenuContainer/MenuButtonsMargin") {
			buttons_margin.call("add_theme_constant_override", &[
				GString::from("margin_top").to_variant(),
				72_i64.to_variant(),
			]);
		}
	}

	fn input(&mut self, event: Gd<InputEvent>) {
		if self.is_in_intro() && self.event_skips_intro(event.clone()) {
			self.intro_done();
			return;
		}

		if event.is_action_released("ui_cancel") {
			self.base_mut().call("try_exit_game", &[]);
		}
		if event.is_action_released("ui_accept") {
			if let Some(viewport) = self.base().get_viewport() {
				if viewport.gui_get_focus_owner().is_none() {
					if let Some(mut box_container) = self.get_node("MenuContainer/MenuButtonsMargin/MenuButtonsContainer/MenuButtonsBoxContainer") {
						box_container.call("focus_first", &[]);
					}
				}
			}
		}
	}
}

#[godot_api]
impl MainMenuWithAnimationsRust {
	fn get_node(&mut self, path: &str) -> Option<Gd<Node>> {
		let base = self.base_mut();
		if !base.has_node(path) {
			return None;
		}
		Some(base.get_node_as::<Node>(path))
	}

	fn get_playback(&mut self) -> Option<Gd<Object>> {
		if let Some(tree) = self.get_node("MenuAnimationTree") {
			let playback = tree.get("parameters/playback");
			if playback.is_nil() {
				return None;
			}
			return Some(playback.to::<Gd<Object>>());
		}
		None
	}

	fn load_texture(&mut self, path: &GString) -> Option<Gd<Texture2D>> {
		let normalized = path.strip_edges(true, true);
		if normalized.is_empty() {
			return None;
		}
		let mut loader = ResourceLoader::singleton();
		let loaded = loader.load(&normalized)?;
		loaded.try_cast::<Texture2D>().ok()
	}

	fn is_in_intro(&mut self) -> bool {
		if let Some(mut playback) = self.get_playback() {
			return playback.call("get_current_node", &[]).to::<GString>() == GString::from("Intro");
		}
		false
	}

	fn event_skips_intro(&self, event: Gd<InputEvent>) -> bool {
		if event.is_action_released("ui_accept") ||
			event.is_action_released("ui_select") ||
			event.is_action_released("ui_cancel") {
			return true;
		}
		if let Ok(mouse_event) = event.try_cast::<InputEventMouseButton>() {
			return !mouse_event.is_pressed();
		}
		false
	}

	#[func]
	fn intro_done(&mut self) {
		if let Some(mut playback) = self.get_playback() {
			playback.call("travel", &[GString::from("OpenMainMenu").to_variant()]);
		}
	}

	#[func]
	fn _on_continue_game_button_pressed(&mut self) {
		self.base_mut().call("load_game_scene", &[]);
	}

	#[func]
	fn _on_new_game_button_pressed(&mut self) {
		let game_path = self.base_mut().call("get_game_scene_path", &[]).to::<GString>();
		if game_path.is_empty() {
			godot_warn!("Configure AppConfig.game_scene_path para habilitar o botao Jogar.");
			return;
		}
		self.base_mut().call("new_game", &[]);
	}
}

#[godot_api]
impl IControl for MainMenuRust {
	fn init(base: Base<Control>) -> Self {
		Self {
			game_scene_path_value: GString::new(),
			options_packed_scene: None,
			credits_packed_scene: None,
			confirm_exit: true,
			signal_game_start: false,
			signal_game_exit: false,
			sub_menu: None,
			base,
		}
	}

	fn ready(&mut self) {
		self.hide_exit_for_web();
		self.hide_options_if_unset();
		self.hide_credits_if_unset();
		self.hide_new_game_if_unset();
	}

	fn input(&mut self, event: Gd<InputEvent>) {
		if event.is_action_released("ui_cancel") {
			if self.sub_menu.is_some() {
				self.close_sub_menu();
			} else {
				self.try_exit_game();
			}
		}
		if event.is_action_released("ui_accept") {
			if let Some(viewport) = self.base().get_viewport() {
				if viewport.gui_get_focus_owner().is_none() {
					if let Some(mut box_container) = self.get_node("MenuContainer/MenuButtonsMargin/MenuButtonsContainer/MenuButtonsBoxContainer") {
						box_container.call("focus_first", &[]);
					}
				}
			}
		}
	}
}

#[godot_api]
impl MainMenuRust {
	fn get_autoload_string(&self, node_name: &str, key: &str) -> GString {
		let tree = self.base().get_tree();
		if let Some(tree) = tree {
			let root = tree.get_root();
			if let Some(root) = root {
				if root.has_node(node_name) {
					let node = root.get_node_as::<Node>(node_name);
					return node.get(key).to::<GString>();
				}
			}
		}
		GString::new()
	}

	fn get_node(&mut self, path: &str) -> Option<Gd<Node>> {
		let base = self.base_mut();
		if !base.has_node(path) {
			return None;
		}
		Some(base.get_node_as::<Node>(path))
	}

	fn get_game_scene_path_internal(&self) -> GString {
		if !self.game_scene_path_value.is_empty() {
			return self.game_scene_path_value.clone();
		}
		self.get_autoload_string("AppConfig", "game_scene_path")
	}

	fn scene_loader_call(&mut self, method: &str, args: &[Variant]) {
		let tree = self.base().get_tree();
		if let Some(tree) = tree {
			let root = tree.get_root();
			if let Some(root) = root {
				if root.has_node("SceneLoader") {
					let mut loader = root.get_node_as::<Node>("SceneLoader");
					loader.call(method, args);
				}
			}
		}
	}

	fn hide_exit_for_web(&mut self) {
		if Engine::singleton().has_singleton("JavaScriptBridge") {
			if let Some(mut exit_button) = self.get_node("MenuContainer/MenuButtonsMargin/MenuButtonsContainer/MenuButtonsBoxContainer/ExitButton") {
				exit_button.call("hide", &[]);
			}
		}
	}

	fn hide_new_game_if_unset(&mut self) {
		if self.get_game_scene_path_internal().is_empty() {
			if let Some(mut new_game_button) = self.get_node("MenuContainer/MenuButtonsMargin/MenuButtonsContainer/MenuButtonsBoxContainer/NewGameButton") {
				new_game_button.call("hide", &[]);
			}
		}
	}

	fn hide_options_if_unset(&mut self) {
		if self.options_packed_scene.is_none() {
			if let Some(mut options_button) = self.get_node("MenuContainer/MenuButtonsMargin/MenuButtonsContainer/MenuButtonsBoxContainer/OptionsButton") {
				options_button.call("hide", &[]);
			}
		}
	}

	fn hide_credits_if_unset(&mut self) {
		if self.credits_packed_scene.is_none() {
			if let Some(mut credits_button) = self.get_node("MenuContainer/MenuButtonsMargin/MenuButtonsContainer/MenuButtonsBoxContainer/CreditsButton") {
				credits_button.call("hide", &[]);
			}
		}
	}

	#[func]
	fn get_game_scene_path(&self) -> GString {
		self.get_game_scene_path_internal()
	}

	#[func]
	fn load_game_scene(&mut self) {
		let path = self.get_game_scene_path_internal();
		if path.is_empty() {
			return;
		}
		if self.signal_game_start {
			self.scene_loader_call("load_scene", &[path.to_variant(), true.to_variant()]);
			self.base_mut().emit_signal("game_started", &[]);
		} else {
			self.scene_loader_call("load_scene", &[path.to_variant()]);
		}
	}

	#[func]
	fn new_game(&mut self) {
		self.load_game_scene();
	}

	#[func]
	fn try_exit_game(&mut self) {
		if self.confirm_exit {
			if let Some(exit_conf) = self.get_node("ExitConfirmation") {
				if !exit_conf.get("visible").to::<bool>() {
					let mut exit_conf = exit_conf;
					exit_conf.call("show", &[]);
					return;
				}
			}
		}
		self.exit_game();
	}

	#[func]
	fn exit_game(&mut self) {
		if Engine::singleton().has_singleton("JavaScriptBridge") {
			return;
		}
		if self.signal_game_exit {
			self.base_mut().emit_signal("game_exited", &[]);
			return;
		}
		if let Some(mut tree) = self.base().get_tree() {
			tree.quit_ex().done();
		}
	}

	#[func]
	fn _open_sub_menu(&mut self, menu: Gd<PackedScene>) -> Variant {
		let instance = menu.instantiate();
		if let Some(instance) = instance {
			if let Ok(control) = instance.try_cast::<Control>() {
				let control = control;
				self.base_mut().add_child(&control);
				if let Some(mut menu_container) = self.get_node("MenuContainer") {
					menu_container.call("hide", &[]);
				}
				self.sub_menu = Some(control.clone());
				self.base_mut().emit_signal("sub_menu_opened", &[]);
				return control.to_variant();
			}
		}
		Variant::nil()
	}

	#[func]
	fn _close_sub_menu(&mut self) {
		if let Some(mut sub_menu) = self.sub_menu.take() {
			sub_menu.queue_free();
			if let Some(mut menu_container) = self.get_node("MenuContainer") {
				menu_container.call("show", &[]);
			}
			self.base_mut().emit_signal("sub_menu_closed", &[]);
		}
	}

	fn close_sub_menu(&mut self) {
		self._close_sub_menu();
	}

	#[func]
	fn _event_is_mouse_button_released(&self, event: Gd<InputEvent>) -> bool {
		if let Ok(mouse_event) = event.try_cast::<InputEventMouseButton>() {
			return !mouse_event.is_pressed();
		}
		false
	}

	#[func]
	fn _on_new_game_button_pressed(&mut self) {
		self.new_game();
	}

	#[func]
	fn _on_options_button_pressed(&mut self) {
		if let Some(menu) = self.options_packed_scene.clone() {
			self._open_sub_menu(menu);
		}
	}

	#[func]
	fn _on_credits_button_pressed(&mut self) {
		if let Some(menu) = self.credits_packed_scene.clone() {
			self._open_sub_menu(menu);
		}
	}

	#[func]
	fn _on_exit_button_pressed(&mut self) {
		self.try_exit_game();
	}

	#[func]
	fn _on_exit_confirmation_confirmed(&mut self) {
		self.exit_game();
	}
}

#[godot_api]
impl ICanvasLayer for LoadingScreenWithShaderCachingRust {
	fn init(base: Base<CanvasLayer>) -> Self {
		Self {
			spatial_shader_material_dir: GString::new(),
			cache_shaders_scene: GString::new(),
			mesh: None,
			matching_extensions: PackedStringArray::from([
				GString::from(".tres"),
				GString::from(".material"),
				GString::from(".res"),
			]),
			ignore_subfolders: PackedStringArray::from([
				GString::from("."),
				GString::from(".."),
			]),
			shader_delay_timer: 0.1,
			progress_lerp_speed: 5.5,
			hold_after_complete_sec: 0.25,
			scene_loading_complete: false,
			scene_loading_progress: 0.0,
			caching_progress: 0.0,
			displayed_progress: 0.0,
			loading_shader_cache: false,
			complete_visible_since_ms: 0,
			base,
		}
	}

	fn ready(&mut self) {
		if let Some(mut sl) = self.get_scene_loader() {
			sl.set("_background_loading", &true.to_variant());
		}
		self.displayed_progress = 0.0;
		self.complete_visible_since_ms = 0;
		self.set_progress_value(0.0);
		self.base_mut().set_process(true);
	}

	fn process(&mut self, delta: f64) {
		let status = self
			.get_scene_loader()
			.map(|mut sl| sl.call("get_status", &[]).to::<i64>())
			.unwrap_or(-1);

		if let Some(mut sl) = self.get_scene_loader() {
			let progress = sl.call("get_progress", &[]).to::<f64>();
			if progress > self.scene_loading_progress {
				self.scene_loading_progress = progress;
			}
		}

		// THREAD_LOAD_LOADED
		if status == 1 {
			self.scene_loading_complete = true;
			self.scene_loading_progress = 1.0;
			if self.can_load_shader_cache() && !self.loading_shader_cache {
				self.loading_shader_cache = true;
				// Safe migration behavior: mark cache phase as complete.
				self.caching_progress = 1.0;
			}
			if let Some(mut sl) = self.get_scene_loader() {
				if !self.can_load_shader_cache() || self.caching_progress >= 1.0 {
					sl.set("_background_loading", &false.to_variant());
					sl.call("set_process", &[true.to_variant()]);
				}
			}
		}

		self.update_total_loading_progress(delta);
	}
}

#[godot_api]
impl LoadingScreenWithShaderCachingRust {
	fn get_scene_loader(&mut self) -> Option<Gd<Node>> {
		let tree = self.base().get_tree()?;
		let root = tree.get_root()?;
		if !root.has_node("SceneLoader") {
			return None;
		}
		Some(root.get_node_as::<Node>("SceneLoader"))
	}

	fn can_load_shader_cache(&mut self) -> bool {
		if self.spatial_shader_material_dir.is_empty() || self.cache_shaders_scene.is_empty() {
			return false;
		}
		if let Some(mut sl) = self.get_scene_loader() {
			return sl
				.call("is_loading_scene", &[self.cache_shaders_scene.to_variant()])
				.to::<bool>();
		}
		false
	}

	fn update_total_loading_progress(&mut self, delta: f64) {
		let mut total = self.scene_loading_progress;
		if self.can_load_shader_cache() {
			total = (total + self.caching_progress) / 2.0;
		}

		let blend = (delta * self.progress_lerp_speed).clamp(0.0, 1.0);
		self.displayed_progress = self.displayed_progress + (total - self.displayed_progress) * blend;
		if (total - self.displayed_progress).abs() < 0.001 {
			self.displayed_progress = total;
		}

		self.set_progress_value(self.displayed_progress);
		let percent = (self.displayed_progress * 100.0).round() as i64;
		if self.scene_loading_complete {
			if self.complete_visible_since_ms == 0 {
				self.complete_visible_since_ms = Time::singleton().get_ticks_msec() as i64;
			}
			self.set_progress_label(GString::from(format!("Loading Complete! ({}%)", percent)));
		} else {
			self.complete_visible_since_ms = 0;
			self.set_progress_label(GString::from(format!("Loading... ({}%)", percent)));
		}
	}

	#[func]
	fn can_close_loading_screen(&self) -> bool {
		if !self.scene_loading_complete {
			return false;
		}
		if self.displayed_progress < 0.999 {
			return false;
		}
		if self.complete_visible_since_ms <= 0 {
			return false;
		}
		let now = Time::singleton().get_ticks_msec() as i64;
		let elapsed_ms = now - self.complete_visible_since_ms;
		elapsed_ms >= (self.hold_after_complete_sec * 1000.0) as i64
	}

	fn set_progress_value(&mut self, value: f64) {
		let base = self.base_mut();
		if !base.has_node("Control/VBoxContainer/ProgressBar") {
			return;
		}
		let mut bar = base.get_node_as::<Node>("Control/VBoxContainer/ProgressBar");
		bar.set("value", &value.to_variant());
	}

	fn set_progress_label(&mut self, text: GString) {
		let base = self.base_mut();
		if !base.has_node("Control/VBoxContainer/ProgressLabel") {
			return;
		}
		let mut label = base.get_node_as::<Node>("Control/VBoxContainer/ProgressLabel");
		label.set("text", &text.to_variant());
	}
}

#[godot_api]
impl IControl for ScrollingCreditsRust {
	fn init(base: Base<Control>) -> Self {
		Self {
			auto_scroll_speed: 60.0,
			input_scroll_speed: 400.0,
			scroll_restart_delay: 1.5,
			scroll_paused: false,
			current_scroll_position: 0.0,
			timer: None,
			base,
		}
	}

	fn ready(&mut self) {
		self.set_header_and_footer();
		self.scroll_paused = false;

		let callable_resize = self.base().callable("_on_resized");
		let callable_visibility = self.base().callable("_on_visibility_changed");
		let callable_input = self.base().callable("_on_gui_input");
		{
			let mut base = self.base_mut();
			base.connect("resized", &callable_resize);
			base.connect("visibility_changed", &callable_visibility);
			base.connect("gui_input", &callable_input);
		}

		if let Some(mut scroll) = self.get_scroll_container() {
			let callable_scroll_started = self.base().callable("_on_scroll_started");
			scroll.connect("scroll_started", &callable_scroll_started);
		}

		let mut timer = Timer::new_alloc();
		let callable_timeout = self.base().callable("_on_scroll_restart_timer_timeout");
		timer.connect("timeout", &callable_timeout);
		self.base_mut().add_child(&timer);
		self.timer = Some(timer);
	}

	fn process(&mut self, delta: f64) {
		let input_axis = Input::singleton().get_axis("ui_up", "ui_down") as f64;
		if input_axis != 0.0 {
			self.scroll_container(input_axis * self.input_scroll_speed * delta);
		} else {
			self.scroll_container(self.auto_scroll_speed * delta);
		}
	}

	fn exit_tree(&mut self) {
		if let Some(scroll) = self.get_scroll_container() {
			self.current_scroll_position = scroll.get("scroll_vertical").to::<i64>() as f64;
		}
	}
}

#[godot_api]
impl ScrollingCreditsRust {
	fn get_node_control(&mut self, path: &str) -> Option<Gd<Control>> {
		let base = self.base_mut();
		if !base.has_node(path) {
			return None;
		}
		Some(base.get_node_as::<Control>(path))
	}

	fn get_scroll_container(&mut self) -> Option<Gd<ScrollContainer>> {
		let base = self.base_mut();
		if !base.has_node("ScrollContainer") {
			return None;
		}
		Some(base.get_node_as::<ScrollContainer>("ScrollContainer"))
	}

	fn set_header_and_footer(&mut self) {
		let size = self.base().get_size();
		if let Some(mut header) = self.get_node_control("HeaderSpace") {
			let mut min = header.get_custom_minimum_size();
			min.y = size.y;
			header.set_custom_minimum_size(min);
		}
		if let Some(mut footer) = self.get_node_control("FooterSpace") {
			let mut min = footer.get_custom_minimum_size();
			min.y = size.y;
			footer.set_custom_minimum_size(min);
		}
		if let Some(mut credits_label) = self.get_node_control("CreditsLabel") {
			let mut min = credits_label.get_custom_minimum_size();
			min.x = size.x;
			credits_label.set_custom_minimum_size(min);
		}
	}

	fn is_end_reached(&mut self) -> bool {
		let credits_h = self
			.get_node_control("CreditsLabel")
			.map(|n| n.get_size().y)
			.unwrap_or(0.0);
		let header_h = self
			.get_node_control("HeaderSpace")
			.map(|n| n.get_size().y)
			.unwrap_or(0.0);
		let end_vertical = credits_h + header_h;
		if let Some(scroll) = self.get_scroll_container() {
			return (scroll.get("scroll_vertical").to::<i64>() as f32) > end_vertical;
		}
		false
	}

	fn check_end_reached(&mut self) {
		if self.is_end_reached() {
			self.scroll_paused = true;
			self.base_mut().emit_signal("end_reached", &[]);
		}
	}

	fn scroll_container(&mut self, amount: f64) {
		if !self.base().is_visible() || self.scroll_paused {
			return;
		}
		self.current_scroll_position += amount;
		if let Some(mut scroll) = self.get_scroll_container() {
			scroll.set("scroll_vertical", &(self.current_scroll_position.round() as i64).to_variant());
		}
		self.check_end_reached();
	}

	fn start_scroll_restart_timer(&mut self) {
		if let Some(mut timer) = self.timer.clone() {
			timer.start_ex().time_sec(self.scroll_restart_delay).done();
		}
	}

	#[signal]
	fn end_reached();

	#[func]
	fn _on_resized(&mut self) {
		self.set_header_and_footer();
		if let Some(scroll) = self.get_scroll_container() {
			self.current_scroll_position = scroll.get("scroll_vertical").to::<i64>() as f64;
		}
	}

	#[func]
	fn _on_gui_input(&mut self, event: Gd<InputEvent>) {
		if event.try_cast::<InputEventMouseButton>().is_ok() {
			self.scroll_paused = true;
			self.start_scroll_restart_timer();
		}
		self.check_end_reached();
	}

	#[func]
	fn _on_scroll_started(&mut self) {
		self.scroll_paused = true;
		self.start_scroll_restart_timer();
	}

	#[func]
	fn _on_scroll_restart_timer_timeout(&mut self) {
		if let Some(scroll) = self.get_scroll_container() {
			self.current_scroll_position = scroll.get("scroll_vertical").to::<i64>() as f64;
		}
		self.scroll_paused = false;
	}

	#[func]
	fn _on_visibility_changed(&mut self) {
		if !self.base().is_visible() {
			return;
		}
		if let Some(mut scroll) = self.get_scroll_container() {
			scroll.set("scroll_vertical", &0_i64.to_variant());
			self.current_scroll_position = 0.0;
		}
		self.scroll_paused = false;
	}
}

#[derive(GodotClass)]
#[class(base=RefCounted)]
struct TelaInicialLogicRust {
	main_menu_scene_path: GString,
	background_image_path: GString,
	prompt_text: GString,
	transitioning: bool,
	blink_time: f64,
	base: Base<RefCounted>,
}

#[godot_api]
impl IRefCounted for TelaInicialLogicRust {
	fn init(base: Base<RefCounted>) -> Self {
		Self {
			main_menu_scene_path: GString::new(),
			background_image_path: GString::from("res://assets/menu_custom/main_menu_bg_rustland_01.png"),
			prompt_text: GString::from("Clique na tela para iniciar"),
			transitioning: false,
			blink_time: 0.0,
			base,
		}
	}
}

#[godot_api]
impl TelaInicialLogicRust {
	fn get_node_label(owner: &mut Gd<Control>, path: &str) -> Option<Gd<Label>> {
		if !owner.has_node(path) {
			return None;
		}
		Some(owner.get_node_as::<Label>(path))
	}

	fn get_node_texture_rect(owner: &mut Gd<Control>, path: &str) -> Option<Gd<TextureRect>> {
		if !owner.has_node(path) {
			return None;
		}
		Some(owner.get_node_as::<TextureRect>(path))
	}

	fn load_texture(path: &GString) -> Option<Gd<Texture2D>> {
		let normalized = path.strip_edges(true, true);
		if normalized.is_empty() {
			return None;
		}
		let mut loader = ResourceLoader::singleton();
		let loaded = loader.load(&normalized)?;
		loaded.try_cast::<Texture2D>().ok()
	}

	fn resolve_main_menu_scene_path(&self, owner: &Gd<Control>) -> GString {
		if !self.main_menu_scene_path.is_empty() {
			return self.main_menu_scene_path.clone();
		}
		if let Some(tree) = owner.get_tree() {
			if let Some(root) = tree.get_root() {
				if root.has_node("AppConfig") {
					let app_config = root.get_node_as::<Node>("AppConfig");
					let value = app_config.get("main_menu_scene_path");
					if !value.is_nil() {
						return value.to::<GString>();
					}
				}
			}
		}
		GString::new()
	}

	#[func]
	fn configure(&mut self, main_menu_scene_path: GString, background_image_path: GString, prompt_text: GString) {
		self.main_menu_scene_path = main_menu_scene_path;
		self.background_image_path = background_image_path;
		self.prompt_text = prompt_text;
	}

	#[func]
	fn on_ready(&mut self, mut owner: Gd<Control>) {
		if let Some(mut background) = Self::get_node_texture_rect(&mut owner, "BackgroundTextureRect") {
			if let Some(texture) = Self::load_texture(&self.background_image_path) {
				background.set_texture(&texture);
			}
		}
		if let Some(mut label) = Self::get_node_label(&mut owner, "PromptLabel") {
			label.set_text(&self.prompt_text);
		}
	}

	#[func]
	fn on_process(&mut self, mut owner: Gd<Control>, delta: f64) {
		self.blink_time += delta;
		let alpha = 0.42 + 0.58 * (0.5 + 0.5 * (self.blink_time * 2.8).sin());
		if let Some(mut label) = Self::get_node_label(&mut owner, "PromptLabel") {
			let mut modulate = label.get_modulate();
			modulate.a = alpha as f32;
			label.set_modulate(modulate);
		}
	}

	#[func]
	fn on_unhandled_input(&mut self, owner: Gd<Control>, event: Gd<InputEvent>) {
		if self.transitioning {
			return;
		}
		if let Ok(mouse_event) = event.try_cast::<InputEventMouseButton>() {
			if mouse_event.is_pressed() && mouse_event.get_button_index() == MouseButton::LEFT {
				self.transitioning = true;
				let mut target = self.resolve_main_menu_scene_path(&owner);
				if target.is_empty() {
					target = GString::from("res://scenes/menus/main_menu/main_menu_with_animations.tscn");
				}
				if let Some(tree) = owner.get_tree() {
					if let Some(root) = tree.get_root() {
						if root.has_node("SceneLoader") {
							let mut scene_loader = root.get_node_as::<Node>("SceneLoader");
							scene_loader.call("load_scene", &[target.to_variant()]);
							return;
						}
					}
				}
				if let Some(mut tree) = owner.get_tree() {
					tree.call("change_scene_to_file", &[target.to_variant()]);
				}
			}
		}
	}
}

#[derive(GodotClass)]
#[class(base=RefCounted)]
struct MainMenuWithAnimationsLogicRust {
	background_image_path: GString,
	base: Base<RefCounted>,
}

#[godot_api]
impl IRefCounted for MainMenuWithAnimationsLogicRust {
	fn init(base: Base<RefCounted>) -> Self {
		Self {
			background_image_path: GString::from("res://assets/menu_custom/main_menu_bg_rustland_01.png"),
			base,
		}
	}
}

#[godot_api]
impl MainMenuWithAnimationsLogicRust {
	fn get_node(owner: &mut Gd<Node>, path: &str) -> Option<Gd<Node>> {
		if !owner.has_node(path) {
			return None;
		}
		Some(owner.get_node_as::<Node>(path))
	}

	fn load_texture(path: &GString) -> Option<Gd<Texture2D>> {
		let normalized = path.strip_edges(true, true);
		if normalized.is_empty() {
			return None;
		}
		let mut loader = ResourceLoader::singleton();
		let loaded = loader.load(&normalized)?;
		loaded.try_cast::<Texture2D>().ok()
	}

	fn get_playback(owner: &mut Gd<Node>) -> Option<Gd<Object>> {
		let tree = Self::get_node(owner, "MenuAnimationTree")?;
		let playback = tree.get("parameters/playback");
		if playback.is_nil() {
			return None;
		}
		Some(playback.to::<Gd<Object>>())
	}

	fn is_in_intro(owner: &mut Gd<Node>) -> bool {
		if let Some(mut playback) = Self::get_playback(owner) {
			return playback.call("get_current_node", &[]).to::<GString>() == GString::from("Intro");
		}
		false
	}

	fn event_skips_intro(event: Gd<InputEvent>) -> bool {
		if event.is_action_released("ui_accept") ||
			event.is_action_released("ui_select") ||
			event.is_action_released("ui_cancel") {
			return true;
		}
		if let Ok(mouse_event) = event.try_cast::<InputEventMouseButton>() {
			return !mouse_event.is_pressed();
		}
		false
	}

	#[func]
	fn configure(&mut self, background_image_path: GString) {
		self.background_image_path = background_image_path;
	}

	#[func]
	fn on_ready(&mut self, mut owner: Gd<Node>) {
		if let Some(mut new_game_button) = Self::get_node(
			&mut owner,
			"MenuContainer/MenuButtonsMargin/MenuButtonsContainer/MenuButtonsBoxContainer/NewGameButton",
		) {
			new_game_button.set("visible", &true.to_variant());
			new_game_button.set("text", &GString::from("Jogar").to_variant());
		}

		if let Some(mut background_rect) = Self::get_node(&mut owner, "BackgroundTextureRect") {
			if let Some(texture) = Self::load_texture(&self.background_image_path) {
				background_rect.set("texture", &texture.to_variant());
			}
			background_rect.set("expand_mode", &1_i64.to_variant());
			background_rect.set("stretch_mode", &6_i64.to_variant());
		}

		if let Some(mut title_margin) = Self::get_node(&mut owner, "MenuContainer/TitleMargin") {
			title_margin.set("visible", &false.to_variant());
		}
		if let Some(mut subtitle_margin) = Self::get_node(&mut owner, "MenuContainer/SubTitleMargin") {
			subtitle_margin.set("visible", &false.to_variant());
		}
		if let Some(mut buttons_margin) = Self::get_node(&mut owner, "MenuContainer/MenuButtonsMargin") {
			buttons_margin.call(
				"add_theme_constant_override",
				&[
					GString::from("margin_top").to_variant(),
					72_i64.to_variant(),
				],
			);
		}
	}

	#[func]
	fn on_input(&mut self, mut owner: Gd<Node>, event: Gd<InputEvent>) -> bool {
		if Self::is_in_intro(&mut owner) && Self::event_skips_intro(event) {
			self.intro_done(owner);
			return true;
		}
		false
	}

	#[func]
	fn intro_done(&mut self, mut owner: Gd<Node>) {
		if let Some(mut playback) = Self::get_playback(&mut owner) {
			playback.call("travel", &[GString::from("OpenMainMenu").to_variant()]);
		}
	}

	#[func]
	fn on_continue_game_button_pressed(&mut self, mut owner: Gd<Node>) {
		owner.call("load_game_scene", &[]);
	}

	#[func]
	fn on_new_game_button_pressed(&mut self, mut owner: Gd<Node>) {
		let game_path = owner.call("get_game_scene_path", &[]).to::<GString>();
		if game_path.is_empty() {
			godot_warn!("Configure AppConfig.game_scene_path para habilitar o botao Jogar.");
			return;
		}
		owner.call("new_game", &[]);
	}
}

#[derive(GodotClass)]
#[class(base=RefCounted)]
struct MainMenuCreditsWindowLogicRust {
	base: Base<RefCounted>,
}

#[godot_api]
impl IRefCounted for MainMenuCreditsWindowLogicRust {
	fn init(base: Base<RefCounted>) -> Self {
		Self { base }
	}
}

#[godot_api]
impl MainMenuCreditsWindowLogicRust {
	#[func]
	fn on_ready(&mut self, owner: Gd<Node>) {
		let instance_variant = owner.get("instance");
		if instance_variant.is_nil() {
			return;
		}
		let mut instance = instance_variant.to::<Gd<Object>>();
		if !instance.has_signal("end_reached") {
			return;
		}
		let close_callable = owner.callable("close");
		instance.connect("end_reached", &close_callable);
	}
}

#[derive(GodotClass)]
#[class(base=RefCounted)]
struct PauseMenuLogicRust {
	base: Base<RefCounted>,
}

#[godot_api]
impl IRefCounted for PauseMenuLogicRust {
	fn init(base: Base<RefCounted>) -> Self {
		Self { base }
	}
}

#[godot_api]
impl PauseMenuLogicRust {
	fn set_menu_buttons_focus_mode(&self, owner: Gd<Node>, focus_mode: i64) {
		if !owner.has_node("ContentContainer/BoxContainer/MenuButtonsMargin/MenuButtons") {
			return;
		}
		let menu_buttons = owner.get_node_as::<Node>("ContentContainer/BoxContainer/MenuButtonsMargin/MenuButtons");
		for mut child in menu_buttons.get_children().iter_shared() {
			if child.is_class("Control") {
				child.set("focus_mode", &focus_mode.to_variant());
			}
		}
	}

	fn mark_window_queue_free_on_hidden(&self, mut window: Gd<Control>, queue_free_on_hidden: bool) {
		window.call(
			"set_meta",
			&[
				StringName::from("queue_free_on_hidden").to_variant(),
				queue_free_on_hidden.to_variant(),
			],
		);
	}

	fn should_queue_free_on_hidden(&self, mut window: Gd<Node>) -> bool {
		let value = window.call(
			"get_meta",
			&[
				StringName::from("queue_free_on_hidden").to_variant(),
				false.to_variant(),
			],
		);
		value.to::<bool>()
	}

	fn get_autoload_string(&self, node_name: &str, key: &str, owner: &Gd<Node>) -> GString {
		if let Some(tree) = owner.get_tree() {
			if let Some(root) = tree.get_root() {
				if root.has_node(node_name) {
					let node = root.get_node_as::<Node>(node_name);
					let value = node.get(key);
					if !value.is_nil() {
						return value.to::<GString>();
					}
				}
			}
		}
		GString::new()
	}

	fn set_visible_if_node_exists(&self, owner: &mut Gd<Node>, path: &str, visible: bool) {
		if owner.has_node(path) {
			let mut node = owner.get_node_as::<Node>(path);
			node.set("visible", &visible.to_variant());
		}
	}

	#[func]
	fn get_main_menu_scene_path(&self, owner: Gd<Node>, main_menu_scene_path: GString) -> GString {
		if !main_menu_scene_path.is_empty() {
			return main_menu_scene_path;
		}
		self.get_autoload_string("AppConfig", "main_menu_scene_path", &owner)
	}

	#[func]
	fn close_window(&mut self, mut owner: Gd<Node>) {
		let open_window = owner.get("open_window");
		if open_window.is_nil() {
			return;
		}
		let mut window = open_window.to::<Gd<Node>>();
		if window.has_method("close") {
			window.call("close", &[]);
		} else {
			window.call("hide", &[]);
		}
		owner.set("open_window", &Variant::nil());
	}

	#[func]
	fn show_window(&mut self, mut owner: Gd<Node>, mut window: Gd<Control>, queue_free_on_hidden: bool) {
		self.set_menu_buttons_focus_mode(owner.clone(), 0);
		self.mark_window_queue_free_on_hidden(window.clone(), queue_free_on_hidden);
		let hidden_callable = owner.callable("_on_open_window_hidden");
		window.connect("hidden", &hidden_callable);
		window.show();
		owner.set("open_window", &window.to_variant());
	}

	#[func]
	fn on_open_window_hidden(&mut self, mut owner: Gd<Node>) {
		let open_window = owner.get("open_window");
		if open_window.is_nil() {
			self.set_menu_buttons_focus_mode(owner, 2);
			return;
		}
		let mut window = open_window.to::<Gd<Node>>();
		owner.set("open_window", &Variant::nil());
		if self.should_queue_free_on_hidden(window.clone()) {
			window.call_deferred("queue_free", &[]);
		}
		self.set_menu_buttons_focus_mode(owner, 2);
	}

	#[func]
	fn load_and_show_menu(&mut self, owner: Gd<Node>, packed_scene: Gd<PackedScene>, mut menu_container: Gd<Node>) {
		if let Some(instance) = packed_scene.instantiate() {
			if let Ok(mut window_instance) = instance.try_cast::<Control>() {
				window_instance.set_visible(false);
				menu_container.call("add_child", &[window_instance.clone().to_variant()]);
				self.show_window(owner, window_instance, true);
			}
		}
	}

	#[func]
	fn load_scene(&mut self, owner: Gd<Node>, scene_path: GString) {
		if let Some(mut tree) = owner.get_tree() {
			tree.set_pause(false);
			if let Some(root) = tree.get_root() {
				if root.has_node("SceneLoader") {
					let mut scene_loader = root.get_node_as::<Node>("SceneLoader");
					scene_loader.call("load_scene", &[scene_path.to_variant()]);
				}
			}
		}
	}

	#[func]
	fn refresh_exit_button(&self, mut owner: Gd<Node>) {
		let is_web = Os::singleton().has_feature("web");
		self.set_visible_if_node_exists(
			&mut owner,
			"ContentContainer/BoxContainer/MenuButtonsMargin/MenuButtons/ExitButton",
			!is_web,
		);
	}

	#[func]
	fn refresh_options_button(&self, mut owner: Gd<Node>) {
		let options_scene = owner.get("options_menu_scene");
		self.set_visible_if_node_exists(
			&mut owner,
			"ContentContainer/BoxContainer/MenuButtonsMargin/MenuButtons/OptionsButton",
			!options_scene.is_nil(),
		);
	}

	#[func]
	fn refresh_main_menu_button(&self, mut owner: Gd<Node>, main_menu_scene_path: GString) {
		self.set_visible_if_node_exists(
			&mut owner,
			"ContentContainer/BoxContainer/MenuButtonsMargin/MenuButtons/MainMenuButton",
			!main_menu_scene_path.is_empty(),
		);
	}
}
