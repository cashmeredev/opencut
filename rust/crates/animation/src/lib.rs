mod bezier;
mod channel_data;
mod channel_layout;
mod color;
mod effect_param_channel;
mod graph_channels;
mod graphic_param_channel;
mod id;
mod interpolation;
mod keyframe_query;
mod keyframes;
mod params;
mod path;
mod property_groups;
mod resolve;
mod types;
mod values;

pub use bezier::{
    BEZIER_SOLVE_ITERATIONS, BezierHandle, bezier_point, default_left_handle,
    default_right_handle, segment_handles, solve_bezier_progress_for_time,
};
pub use channel_data::{
    get_channel_entries_from_data, get_channels_from_data, is_animation_storage_key,
    ordered_composite_entries,
};
pub use channel_layout::{
    ChannelComponentDefinition, ChannelLayout, ChannelValueKind, boolean_channel_layout,
    color_channel_layout, number_channel_layout, string_channel_layout,
};
pub use color::{LinearRgba, format_linear_rgba, parse_color_to_linear_rgba};
pub use effect_param_channel::{
    EFFECT_PARAM_PATH_PREFIX, EFFECT_PARAM_PATH_SUFFIX, build_effect_param_path,
    is_effect_param_path, parse_effect_param_path, remove_effect_param_keyframe,
    resolve_effect_params_at_time,
};
pub use graph_channels::{
    ChannelEasingMode, EditableScalarChannels, ScalarGraphChannel, ScalarGraphKeyframeContext,
    get_editable_scalar_channel, get_editable_scalar_channels, get_scalar_keyframe_context,
};
pub use graphic_param_channel::{
    GRAPHIC_PARAM_PATH_PREFIX, build_graphic_param_path, is_graphic_param_path,
    parse_graphic_param_path, resolve_graphic_params_at_time,
};
pub use id::generate_uuid;
pub use interpolation::{
    channel_value_at_time, discrete_channel_value_at_time, is_scalar_channel, normalize_channel,
    normalize_discrete_channel, normalize_scalar_channel, scalar_channel_value_at_time,
    scalar_segment_interpolation,
};
pub use keyframe_query::{
    get_element_keyframes, get_keyframe_at_time, get_keyframe_by_id, has_keyframes_for_path,
};
pub use keyframes::{
    SplitChannelResult, clamp_animations_to_duration, clone_animations, get_channel,
    remove_element_keyframe, remove_keyframe, retime_element_keyframe, retime_keyframe,
    set_binding_component_channel, set_channel, split_animations_at_time,
    split_animations_at_time_with_options, to_animation, update_scalar_keyframe_curve,
    upsert_discrete_channel_key, upsert_keyframe, upsert_path_keyframe, upsert_scalar_channel_key,
};
pub use params::{
    NumberParamDefinition, ParamDefinition, ParamValueKind, SelectOption, coerce_param_value,
    get_param_channel_layout, get_param_default_interpolation, get_param_numeric_range,
    get_param_value_kind, snap_to_step,
};
pub use path::is_animation_path;
pub use property_groups::{
    ANIMATION_PROPERTY_GROUPS, GroupKeyframeRef, animation_property_group_paths,
    get_group_keyframes_at_time, has_group_keyframe_at_time, is_animation_property_path,
};
pub use resolve::{get_element_local_time, resolve_animation_path_value_at_time};
pub use types::{
    AnimationInterpolation, ElementKeyframe, NumericSpec, ScalarCurveKeyframePatch,
};
pub use values::{resolve_color_at_time, resolve_number_at_time, resolve_opacity_at_time};
