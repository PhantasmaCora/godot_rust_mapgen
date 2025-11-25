shader_type spatial;
render_mode unshaded, depth_test_disabled;

uniform sampler2D s_norm : source_color, hint_normal_roughness_texture, filter_nearest, repeat_disable;
uniform sampler2D s_depth : source_color, hint_depth_texture, filter_nearest, repeat_disable;

uniform vec3 outline_color : source_color;

float vec_angle(vec3 a, vec3 b)
{
	return acos( dot(a, b) / length(a) / length(b) );
}

vec3 screen_to_view(vec2 screen_uv, mat4 inv_projection_matrix)
{
	vec3 ndc = vec3(screen_uv * 2.0 - 1.0, 1.0);
	vec4 view = inv_projection_matrix * vec4(ndc, 1.0);
	return view.xyz / -view.w;
}

vec3 get_normal(vec2 screen_uv)
{
	return texture(s_norm, screen_uv).rgb * 2.0 - 1.0;
}

float get_depth(vec2 screen_uv, mat4 inv_projection_matrix)
{
	float depth = texture(s_depth, screen_uv).r;
	vec3 ndc = vec3(screen_uv * 2.0 - 1.0, depth);
	vec4 view = inv_projection_matrix * vec4(ndc, 1.0);
	view.xyz /= -view.w;
	return view.z;
}

vec4 add_sobel(vec2 screen_uv, mat4 inv_projection_matrix, vec3 center_normal, float center_depth, vec2 center_uv, vec2 sobel_bias)
{
	float ldepth = get_depth(screen_uv, inv_projection_matrix);
	vec3 lnorm = get_normal(screen_uv);
	vec3 ndiff = lnorm - center_normal;
	float lnscalar = length(ndiff);

	if ( all( equal(lnorm, center_normal) ) && ldepth < 3900.0 ) {
		float a = PI / 2.0 - vec_angle(center_normal, screen_to_view(center_uv, inv_projection_matrix));
		float b = PI / 2.0 - vec_angle(lnorm, screen_to_view(screen_uv, inv_projection_matrix));

		ldepth /= sin(a);
		ldepth *= sin(b);
	}

	return vec4( ldepth * sobel_bias.x, ldepth * sobel_bias.y, lnscalar * sobel_bias.x, lnscalar * sobel_bias.y );
}

void fragment() {

	vec3 normal = get_normal(SCREEN_UV);
	float center_depth = get_depth(SCREEN_UV, INV_PROJECTION_MATRIX);

	vec2 texel_size = 1.0 / VIEWPORT_SIZE.xy;
	vec2 uvs[8]; // arrray containing the uvs of the surrounding pixel
	uvs[0] = vec2(SCREEN_UV.x, SCREEN_UV.y + texel_size.y );
	uvs[1] = vec2(SCREEN_UV.x, SCREEN_UV.y - texel_size.y );
	uvs[2] = vec2(SCREEN_UV.x + texel_size.x, SCREEN_UV.y);
	uvs[3] = vec2(SCREEN_UV.x - texel_size.x, SCREEN_UV.y);
	uvs[4] = vec2(SCREEN_UV.x + texel_size.x, SCREEN_UV.y + texel_size.y );
	uvs[5] = vec2(SCREEN_UV.x - texel_size.x, SCREEN_UV.y + texel_size.y );
	uvs[6] = vec2(SCREEN_UV.x + texel_size.x, SCREEN_UV.y - texel_size.y );
	uvs[7] = vec2(SCREEN_UV.x - texel_size.x, SCREEN_UV.y - texel_size.y );

	// Sobel operator
	vec4 sobel = vec4( 0.0, 0.0, 0.0, 0.0 );

	sobel += add_sobel(uvs[0], INV_PROJECTION_MATRIX, normal, center_depth, SCREEN_UV, vec2(0.0, 10.0) );
	sobel += add_sobel(uvs[1], INV_PROJECTION_MATRIX, normal, center_depth, SCREEN_UV, vec2(0.0, -10.0) );
	sobel += add_sobel(uvs[2], INV_PROJECTION_MATRIX, normal, center_depth, SCREEN_UV, vec2(10.0, 0.0) );
	sobel += add_sobel(uvs[3], INV_PROJECTION_MATRIX, normal, center_depth, SCREEN_UV, vec2(-10.0, 0.0) );
	sobel += add_sobel(uvs[4], INV_PROJECTION_MATRIX, normal, center_depth, SCREEN_UV, vec2(3.0, 3.0) );
	sobel += add_sobel(uvs[5], INV_PROJECTION_MATRIX, normal, center_depth, SCREEN_UV, vec2(-3.0, 3.0) );
	sobel += add_sobel(uvs[6], INV_PROJECTION_MATRIX, normal, center_depth, SCREEN_UV, vec2(3.0, -3.0) );
	sobel += add_sobel(uvs[7], INV_PROJECTION_MATRIX, normal, center_depth, SCREEN_UV, vec2(-3.0, -3.0) );

	vec2 sob_d = vec2(sobel.x, sobel.y);
	vec2 sob_n = vec2(sobel.z, sobel.w);

	float edge_d = length(sob_d) * 0.1;
	float edge_n = length(sob_n) * 0.1;

	ALBEDO = outline_color;
	ALPHA = clamp( edge_d + edge_n, 0.0, 1.0);
}