class_name MainMenu
extends Control

signal game_started
signal game_exited
signal sub_menu_opened
signal sub_menu_closed

@export_file("*.tscn") var game_scene_path: String = ""
@export var options_packed_scene: PackedScene
@export var credits_packed_scene: PackedScene
@export var confirm_exit: bool = true
@export var signal_game_start: bool = false
@export var signal_game_exit: bool = false

var sub_menu: Control = null

func _ready() -> void:
	_hide_exit_for_web()
	_hide_options_if_unset()
	_hide_credits_if_unset()
	_hide_new_game_if_unset()

func _input(event: InputEvent) -> void:
	if event.is_action_released("ui_cancel"):
		if sub_menu != null:
			_close_sub_menu()
		else:
			try_exit_game()
	if event.is_action_released("ui_accept"):
		if get_viewport().gui_get_focus_owner() == null and has_node("MenuContainer/MenuButtonsMargin/MenuButtonsContainer/MenuButtonsBoxContainer"):
			$MenuContainer/MenuButtonsMargin/MenuButtonsContainer/MenuButtonsBoxContainer.focus_first()

func _autoload_string(node_name: String, key: String) -> String:
	if has_node("/root/%s" % node_name):
		return get_node("/root/%s" % node_name).get(key)
	return ""

func get_game_scene_path() -> String:
	if not game_scene_path.is_empty():
		return game_scene_path
	return _autoload_string("AppConfig", "game_scene_path")

func _scene_loader_call(method_name: String, args: Array = []) -> void:
	if has_node("/root/SceneLoader"):
		get_node("/root/SceneLoader").callv(method_name, args)

func load_game_scene() -> void:
	var path := get_game_scene_path()
	if path.is_empty():
		return
	if signal_game_start:
		_scene_loader_call("load_scene", [path, true])
		game_started.emit()
	else:
		_scene_loader_call("load_scene", [path])

func new_game() -> void:
	load_game_scene()

func try_exit_game() -> void:
	if confirm_exit and has_node("ExitConfirmation") and not $ExitConfirmation.visible:
		$ExitConfirmation.show()
		return
	exit_game()

func exit_game() -> void:
	if OS.has_feature("web"):
		return
	if signal_game_exit:
		game_exited.emit()
		return
	get_tree().quit()

func _open_sub_menu(menu: PackedScene) -> Variant:
	var instance := menu.instantiate()
	if instance is Control:
		add_child(instance)
		if instance.has_signal("closed") and not instance.closed.is_connected(_on_sub_menu_window_closed):
			instance.closed.connect(_on_sub_menu_window_closed)
		if instance.has_signal("hidden") and not instance.hidden.is_connected(_on_sub_menu_window_closed):
			instance.hidden.connect(_on_sub_menu_window_closed)
		if has_node("MenuContainer"):
			$MenuContainer.hide()
		sub_menu = instance
		sub_menu_opened.emit()
		return instance
	return null

func _close_sub_menu() -> void:
	if sub_menu == null:
		return
	if is_instance_valid(sub_menu):
		sub_menu.queue_free()
	sub_menu = null
	if has_node("MenuContainer"):
		$MenuContainer.show()
	sub_menu_closed.emit()

func _on_sub_menu_window_closed() -> void:
	_close_sub_menu()

func _event_is_mouse_button_released(event: InputEvent) -> bool:
	return event is InputEventMouseButton and not event.pressed

func _hide_exit_for_web() -> void:
	if OS.has_feature("web") and has_node("MenuContainer/MenuButtonsMargin/MenuButtonsContainer/MenuButtonsBoxContainer/ExitButton"):
		$MenuContainer/MenuButtonsMargin/MenuButtonsContainer/MenuButtonsBoxContainer/ExitButton.hide()

func _hide_new_game_if_unset() -> void:
	if get_game_scene_path().is_empty() and has_node("MenuContainer/MenuButtonsMargin/MenuButtonsContainer/MenuButtonsBoxContainer/NewGameButton"):
		$MenuContainer/MenuButtonsMargin/MenuButtonsContainer/MenuButtonsBoxContainer/NewGameButton.hide()

func _hide_options_if_unset() -> void:
	if options_packed_scene == null and has_node("MenuContainer/MenuButtonsMargin/MenuButtonsContainer/MenuButtonsBoxContainer/OptionsButton"):
		$MenuContainer/MenuButtonsMargin/MenuButtonsContainer/MenuButtonsBoxContainer/OptionsButton.hide()

func _hide_credits_if_unset() -> void:
	if credits_packed_scene == null and has_node("MenuContainer/MenuButtonsMargin/MenuButtonsContainer/MenuButtonsBoxContainer/CreditsButton"):
		$MenuContainer/MenuButtonsMargin/MenuButtonsContainer/MenuButtonsBoxContainer/CreditsButton.hide()

func _on_new_game_button_pressed() -> void:
	new_game()

func _on_options_button_pressed() -> void:
	if options_packed_scene != null:
		_open_sub_menu(options_packed_scene)

func _on_credits_button_pressed() -> void:
	if credits_packed_scene != null:
		_open_sub_menu(credits_packed_scene)

func _on_exit_button_pressed() -> void:
	try_exit_game()

func _on_exit_confirmation_confirmed() -> void:
	exit_game()
