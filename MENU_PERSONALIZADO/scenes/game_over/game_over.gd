extends Control

@export_file("*.tscn") var retry_scene_path: String = ""
@export_file("*.tscn") var main_menu_scene_path: String = "res://scenes/menus/main_menu/main_menu_with_animations.tscn"

func _go_to_scene(path: String) -> void:
	if path.is_empty():
		return
	if has_node("/root/SceneLoader"):
		get_node("/root/SceneLoader").call("load_scene", path)
		return
	get_tree().change_scene_to_file(path)

func _on_retry_button_pressed() -> void:
	if retry_scene_path.is_empty():
		retry_scene_path = main_menu_scene_path
	_go_to_scene(retry_scene_path)

func _on_menu_button_pressed() -> void:
	_go_to_scene(main_menu_scene_path)
