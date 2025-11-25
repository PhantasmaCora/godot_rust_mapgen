extends "res://addons/fpc/character.gd"

var lg_pos: Vector3

func _physics_process(delta): # Most things happen here.
	# Gravity
	if dynamic_gravity:
		gravity = ProjectSettings.get_setting("physics/3d/default_gravity")
	if not is_on_floor() and gravity and gravity_enabled:
		velocity.y -= gravity * delta

	handle_jumping()

	var input_dir = Vector2.ZERO

	if not immobile: # Immobility works by interrupting user input, so other forces can still be applied to the player
		input_dir = Input.get_vector(controls.LEFT, controls.RIGHT, controls.FORWARD, controls.BACKWARD)

	handle_movement(delta, input_dir)
	
	if not is_on_floor():
		pass

	handle_head_rotation()

	# The player is not able to stand up if the ceiling is too low
	low_ceiling = $CrouchCeilingDetection.is_colliding()

	handle_state(input_dir)
	if dynamic_fov: # This may be changed to an AnimationPlayer
		update_camera_fov()

	if view_bobbing:
		play_headbob_animation(input_dir)

	if jump_animation:
		play_jump_animation()

	if not is_on_floor() and velocity.y < 0 and not immobile:
		check_ledge_grab(delta)

	update_debug_menu_per_tick()

	was_on_floor = is_on_floor() # This must always be at the end of physics_process

func check_ledge_grab(_delta: float):
	$LedgeGrabHandler.global_rotation.y = $Head.global_rotation.y
	
	$LedgeGrabHandler/ShapeCast3D.target_position = Vector3.ZERO
	$LedgeGrabHandler/ShapeCast3D.force_shapecast_update()
	if $LedgeGrabHandler/ShapeCast3D.is_colliding(): return
	
	$LedgeGrabHandler/ShapeCast3D.target_position = Vector3(0.0, -0.8, 0.0)
	$LedgeGrabHandler/ShapeCast3D.force_shapecast_update()
	
	if $LedgeGrabHandler/ShapeCast3D.is_colliding():
		if $LedgeGrabHandler/ShapeCast3D.get_collision_normal(0).y < 0.6: return
		
		var p = $LedgeGrabHandler/ShapeCast3D.get_collision_point(0)
		lg_pos = p
		ledge_grab_tween()

func ledge_grab_tween():
	var tween = get_tree().create_tween()
	tween.tween_property(self, "global_position:y", lg_pos.y, 0.5)
	tween.set_parallel(true)
	tween.tween_property(self, "immobile", true, 0.0)
	tween.tween_property(self, "gravity_enabled", false, 0.0)
	tween.set_ease(Tween.EASE_IN)
	tween.tween_property(self, "global_position:x", lg_pos.x, 0.5)
	tween.tween_property(self, "global_position:z", lg_pos.z, 0.5)
	tween.set_parallel(false)
	tween.tween_property(self, "immobile", false, 0.0)
	tween.set_parallel(true)
	tween.tween_property(self, "gravity_enabled", true, 0.0)
