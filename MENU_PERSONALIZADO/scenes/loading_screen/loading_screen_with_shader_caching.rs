extends LoadingScreenWithShaderCachingRust

@export_range(0.5, 16.0, 0.1) var progress_lerp_speed: float = 5.5

var _displayed_progress: float = 0.0

@onready var _progress_bar: ProgressBar = $Control/VBoxContainer/ProgressBar
@onready var _progress_label: Label = $Control/VBoxContainer/ProgressLabel

func _ready() -> void:
	_displayed_progress = 0.0
	if is_instance_valid(_progress_bar):
		_progress_bar.value = 0.0
	var tree := get_tree()
	if tree != null and not tree.process_frame.is_connected(_on_process_frame):
		tree.process_frame.connect(_on_process_frame)

func _exit_tree() -> void:
	var tree := get_tree()
	if tree != null and tree.process_frame.is_connected(_on_process_frame):
		tree.process_frame.disconnect(_on_process_frame)

func _on_process_frame() -> void:
	_update_progress_fx(get_process_delta_time())

func _update_progress_fx(delta: float) -> void:
	if not is_instance_valid(_progress_bar) or not is_instance_valid(_progress_label):
		return
	var target_progress: float = clampf(float(_progress_bar.value), 0.0, 1.0)
	var blend: float = clampf(delta * progress_lerp_speed, 0.0, 1.0)
	_displayed_progress = lerpf(_displayed_progress, target_progress, blend)
	if absf(target_progress - _displayed_progress) < 0.001:
		_displayed_progress = target_progress
	_progress_bar.value = _displayed_progress

	var percent: int = int(round(_displayed_progress * 100.0))
	var base_text: String = _progress_label.text
	var split_index: int = base_text.find(" (")
	if split_index >= 0:
		base_text = base_text.substr(0, split_index)
	_progress_label.text = "%s (%d%%)" % [base_text, percent]
