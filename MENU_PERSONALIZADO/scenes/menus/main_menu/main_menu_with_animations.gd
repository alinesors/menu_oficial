extends "res://scenes/menus/main_menu/main_menu.gd"

@export_file("*.png", "*.jpg", "*.jpeg", "*.webp") var background_image_path: String = "res://assets/menu_custom/main_menu_bg_rustland_01.png"

var _rust: Object = null

func _ready() -> void:
	super._ready()
	if ClassDB.class_exists("MainMenuWithAnimationsLogicRust"):
		_rust = ClassDB.instantiate("MainMenuWithAnimationsLogicRust")
	if _rust != null:
		_rust.call("configure", background_image_path)
		_rust.call("on_ready", self)
		return

	var new_game_button := $MenuContainer/MenuButtonsMargin/MenuButtonsContainer/MenuButtonsBoxContainer/NewGameButton
	new_game_button.visible = true
	new_game_button.text = "Jogar"

	if ResourceLoader.exists(background_image_path):
		var tex := load(background_image_path)
		if tex is Texture2D:
			$BackgroundTextureRect.texture = tex
	$BackgroundTextureRect.expand_mode = TextureRect.EXPAND_IGNORE_SIZE
	$BackgroundTextureRect.stretch_mode = TextureRect.STRETCH_KEEP_ASPECT_COVERED

	$MenuContainer/TitleMargin.visible = false
	$MenuContainer/SubTitleMargin.visible = false
	$MenuContainer/MenuButtonsMargin.add_theme_constant_override("margin_top", 72)

func _input(event: InputEvent) -> void:
	if _rust != null:
		if _rust.call("on_input", self, event):
			get_viewport().set_input_as_handled()
			return
		super._input(event)
		return

	if _is_in_intro() and _event_skips_intro(event):
		intro_done()
		get_viewport().set_input_as_handled()
		return
	super._input(event)

func _get_playback() -> AnimationNodeStateMachinePlayback:
	return $MenuAnimationTree["parameters/playback"]

func _is_in_intro() -> bool:
	var playback := _get_playback()
	return playback.get_current_node() == "Intro"

func _event_skips_intro(event: InputEvent) -> bool:
	if event.is_action_released("ui_accept") or event.is_action_released("ui_select") or event.is_action_released("ui_cancel"):
		return true
	if event is InputEventMouseButton:
		return not event.pressed
	return false

func intro_done() -> void:
	if _rust != null:
		_rust.call("intro_done", self)
		return

	var playback := _get_playback()
	playback.travel("OpenMainMenu")

func _on_continue_game_button_pressed() -> void:
	if _rust != null:
		_rust.call("on_continue_game_button_pressed", self)
		return

	load_game_scene()

func _on_new_game_button_pressed() -> void:
	if _rust != null:
		_rust.call("on_new_game_button_pressed", self)
		return

	if get_game_scene_path().is_empty():
		push_warning("Configure AppConfig.game_scene_path para habilitar o botao Jogar.")
		return
	new_game()
