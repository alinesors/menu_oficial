@tool
extends "res://addons/removed_template/base/nodes/windows/overlaid_window_scene_container.gd"

@onready var _rust := MainMenuCreditsWindowLogicRust.new()

func _ready() -> void:
	super._ready()
	_rust.on_ready(self)
