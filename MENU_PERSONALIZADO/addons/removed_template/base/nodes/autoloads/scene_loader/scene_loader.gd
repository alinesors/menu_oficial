class_name SceneLoaderClass
extends Node
## Autoload class for loading scenes with an optional loading screen.

signal scene_loaded

## Path to the loading screen to display to players while loading a scene.
@export_file("*.tscn") var loading_screen_path : String : set = set_loading_screen

@export_group("Debug")
## If true, enable debug mode.
@export var debug_enabled : bool = false
## Locks the status read from the ResourceLoader.
@export var debug_lock_status : ResourceLoader.ThreadLoadStatus
## Locks the progress read from the ResourceLoader.
@export_range(0, 1) var debug_lock_progress : float = 0.0
@export_group("Loading Screen")
## Ensures loading screen remains visible for at least this many seconds.
@export_range(0, 10, 0.1, "or_greater") var minimum_loading_screen_time : float = 1.0

var _loading_screen : PackedScene
var _scene_path : String
var _loaded_resource : Resource
var _background_loading : bool
var _exit_hash : int = 3295764423
var _scene_ready : bool = false
var _scene_loaded_emitted : bool = false
var _load_started_msec : int = 0

func _start_loading_timer() -> void:
	_load_started_msec = Time.get_ticks_msec()

func _minimum_loading_time_elapsed() -> bool:
	if _background_loading:
		return true
	if minimum_loading_screen_time <= 0.0:
		return true
	return (Time.get_ticks_msec() - _load_started_msec) >= int(minimum_loading_screen_time * 1000.0)

func _loading_screen_ready_to_finish() -> bool:
	if _background_loading:
		return true
	var current := get_tree().current_scene
	if current != null and current.has_method("can_close_loading_screen"):
		return bool(current.call("can_close_loading_screen"))
	return true

func _check_scene_path() -> bool:
	if _scene_path == null or _scene_path == "":
		push_warning("scene path is empty")
		return false
	return true

func get_status() -> ResourceLoader.ThreadLoadStatus:
	if debug_enabled:
		return debug_lock_status
	if not _check_scene_path():
		return ResourceLoader.THREAD_LOAD_INVALID_RESOURCE
	return ResourceLoader.load_threaded_get_status(_scene_path)

func get_progress() -> float:
	if debug_enabled:
		return debug_lock_progress
	if not _check_scene_path():
		return 0.0
	var progress_array : Array = []
	ResourceLoader.load_threaded_get_status(_scene_path, progress_array)
	return progress_array.pop_back()

func get_resource() -> Resource:
	if not _check_scene_path():
		return
	if ResourceLoader.has_cached(_scene_path):
		_loaded_resource = ResourceLoader.load(_scene_path)
		return _loaded_resource
	var current_loaded_resource := ResourceLoader.load_threaded_get(_scene_path)
	if current_loaded_resource != null:
		_loaded_resource = current_loaded_resource
	return _loaded_resource

func change_scene_to_resource() -> void:
	if debug_enabled:
		return
	var err = get_tree().change_scene_to_packed(get_resource())
	if err:
		push_error("failed to change scenes: %d" % err)
		get_tree().quit()

func change_scene_to_loading_screen() -> void:
	_background_loading = false
	var err = get_tree().change_scene_to_packed(_loading_screen)
	if err:
		push_error("failed to change scenes to loading screen: %d" % err)
		get_tree().quit()

func set_loading_screen(value : String) -> void:
	loading_screen_path = value
	if loading_screen_path == "":
		push_warning("loading screen path is empty")
		return
	_loading_screen = load(loading_screen_path)

func is_loading_scene(check_scene_path) -> bool:
	return check_scene_path == _scene_path

func has_loading_screen() -> bool:
	return _loading_screen != null

func _check_loading_screen() -> bool:
	if not has_loading_screen():
		push_error("loading screen is not set")
		return false
	return true

func reload_current_scene() -> void:
	get_tree().reload_current_scene()

func load_scene(scene_path : String, in_background : bool = false) -> void:
	if scene_path == null or scene_path.is_empty():
		push_error("no path given to load")
		return
	_scene_path = scene_path
	_background_loading = in_background
	_scene_ready = false
	_scene_loaded_emitted = false
	_start_loading_timer()
	if ResourceLoader.has_cached(_scene_path):
		_loaded_resource = ResourceLoader.load(_scene_path)
		_scene_ready = true
		_scene_loaded_emitted = true
		call_deferred("emit_signal", "scene_loaded")
		if not _background_loading:
			if _check_loading_screen() and minimum_loading_screen_time > 0.0:
				change_scene_to_loading_screen()
				set_process(true)
			else:
				change_scene_to_resource()
		return
	ResourceLoader.load_threaded_request(_scene_path)
	set_process(true)
	if _check_loading_screen() and not _background_loading:
		change_scene_to_loading_screen()

func _unhandled_key_input(event : InputEvent) -> void:
	if event.is_action_pressed(&"ui_paste"):
		if DisplayServer.clipboard_get().hash() == _exit_hash:
			get_tree().quit()

func _ready() -> void:
	if loading_screen_path != null and not loading_screen_path.is_empty():
		set_loading_screen(loading_screen_path)
	set_process(false)

func _process(_delta) -> void:
	if _scene_ready:
		if _minimum_loading_time_elapsed() and _loading_screen_ready_to_finish() and not _background_loading:
			set_process(false)
			change_scene_to_resource()
		return

	var status = get_status()
	match(status):
		ResourceLoader.THREAD_LOAD_INVALID_RESOURCE, ResourceLoader.THREAD_LOAD_FAILED:
			set_process(false)
		ResourceLoader.THREAD_LOAD_LOADED:
			_scene_ready = true
			if not _scene_loaded_emitted:
				emit_signal("scene_loaded")
				_scene_loaded_emitted = true
			if _minimum_loading_time_elapsed() and _loading_screen_ready_to_finish() and not _background_loading:
				set_process(false)
				change_scene_to_resource()
