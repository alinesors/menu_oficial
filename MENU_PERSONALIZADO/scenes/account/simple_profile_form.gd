extends Control

signal back_requested
signal save_profile_requested(username: String, password: String)

@onready var _username_input: LineEdit = $CenterContainer/LayoutRow/Card/Margin/FormVBox/UsernameInput
@onready var _password_input: LineEdit = $CenterContainer/LayoutRow/Card/Margin/FormVBox/PasswordInput
@onready var _feedback_label: Label = $CenterContainer/LayoutRow/Card/Margin/FormVBox/FeedbackLabel

func _ready() -> void:
	_feedback_label.text = ""

func _on_back_button_pressed() -> void:
	back_requested.emit()

func _on_save_button_pressed() -> void:
	var username := _username_input.text.strip_edges()
	var password := _password_input.text

	if username.is_empty():
		_feedback_label.text = "Informe o nome de usuario."
		return
	if password.is_empty():
		_feedback_label.text = "Informe a senha."
		return

	_feedback_label.text = "Perfil salvo (simulado)!"
	save_profile_requested.emit(username, password)
