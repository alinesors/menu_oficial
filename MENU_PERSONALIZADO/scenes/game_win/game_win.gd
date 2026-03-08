extends Control

@export_file("*.tscn") var main_menu_scene_path: String = "res://scenes/menus/main_menu/main_menu_with_animations.tscn"

func _on_menu_button_pressed() -> void:
	if main_menu_scene_path.is_empty():
		return
	if has_node("/root/SceneLoader"):
		get_node("/root/SceneLoader").call("load_scene", main_menu_scene_path)
		return
	get_tree().change_scene_to_file(main_menu_scene_path)
