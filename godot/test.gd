extends Node3D

func _ready() -> void:
	$GeneratedGridMap.generate_default()

func _on_generated_grid_map_generation_finished() -> void:
	$GeneratedGridMap.place_default()

	
	var list = $GeneratedGridMap.get_list("points")
	
	var pos = list[0]
	
	var player = preload("res://player/character.tscn").instantiate()
	
	add_child(player)
	
	player.global_position = Vector3(pos)
