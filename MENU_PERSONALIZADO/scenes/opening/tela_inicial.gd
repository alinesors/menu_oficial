extends Control

@export_file("*.tscn") var main_menu_scene_path: String = ""
@export_file("*.png", "*.jpg", "*.jpeg", "*.webp") var background_image_path: String = "res://assets/menu_custom/opening_bg_01.png"
@export var prompt_text: String = "Clique na tela para iniciar"

var _rust: Object = null
var _fallback_transitioning: bool = false
var _fallback_blink_time: float = 0.0

@onready var _prompt_label: Label = $PromptLabel
@onready var _background_rect: TextureRect = $BackgroundTextureRect

func _ready() -> void:
	if ClassDB.class_exists("TelaInicialLogicRust"):
		_rust = ClassDB.instantiate("TelaInicialLogicRust")
	if _rust != null:
		_rust.call("configure", main_menu_scene_path, background_image_path, prompt_text)
		_rust.call("on_ready", self)
		return
	_fallback_ready()

func _process(delta: float) -> void:
	if _rust != null:
		_rust.call("on_process", self, delta)
		return
	_fallback_process(delta)

func _input(event: InputEvent) -> void:
	_handle_click_event(event)

func _unhandled_input(event: InputEvent) -> void:
	_handle_click_event(event)

func _gui_input(event: InputEvent) -> void:
	_handle_click_event(event)

func _handle_click_event(event: InputEvent) -> void:
	if _rust != null:
		_rust.call("on_unhandled_input", self, event)
		return
	_fallback_unhandled_input(event)

func _fallback_ready() -> void:
	if ResourceLoader.exists(background_image_path):
		var tex := load(background_image_path)
		if tex is Texture2D:
			_background_rect.texture = tex
	_prompt_label.text = prompt_text

func _fallback_process(delta: float) -> void:
	_fallback_blink_time += delta
	var alpha := 0.42 + 0.58 * (0.5 + 0.5 * sin(_fallback_blink_time * 2.8))
	var c := _prompt_label.modulate
	c.a = alpha
	_prompt_label.modulate = c

func _resolve_main_menu_path() -> String:
	if not main_menu_scene_path.is_empty():
		return main_menu_scene_path
	if has_node("/root/AppConfig"):
		var app_config: Node = get_node("/root/AppConfig")
		var value_variant: Variant = app_config.get("main_menu_scene_path")
		if value_variant is String:
			var value: String = value_variant
			if not value.is_empty():
				return value
	return "res://scenes/menus/main_menu/main_menu_with_animations.tscn"

func _fallback_unhandled_input(event: InputEvent) -> void:
	if _fallback_transitioning:
		return
	if event is InputEventMouseButton and event.pressed and event.button_index == MOUSE_BUTTON_LEFT:
		_fallback_transitioning = true
		var target := _resolve_main_menu_path()
		if has_node("/root/SceneLoader"):
			get_node("/root/SceneLoader").call("load_scene", target)
		else:
			get_tree().change_scene_to_file(target)
