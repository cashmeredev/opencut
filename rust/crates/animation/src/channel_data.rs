use scene::{AnimationChannel, ChannelData, CompositeChannelData};

const LEGACY_ANIMATION_STORAGE_KEYS: [&str; 2] = ["bindings", "channels"];

pub fn is_animation_storage_key(key: &str) -> bool {
    !LEGACY_ANIMATION_STORAGE_KEYS.contains(&key)
}

fn component_order(key: &str) -> usize {
    match key {
        "value" => 0,
        "r" => 1,
        "g" => 2,
        "b" => 3,
        "a" => 4,
        _ => 5,
    }
}

pub fn ordered_composite_entries(data: &CompositeChannelData) -> Vec<(&String, &AnimationChannel)> {
    let mut entries: Vec<(&String, &AnimationChannel)> = data.iter().collect();
    entries.sort_by_key(|(key, _)| component_order(key));
    entries
}

pub fn get_channels_from_data(data: Option<&ChannelData>) -> Vec<&AnimationChannel> {
    match data {
        Some(ChannelData::Single(channel)) => vec![channel],
        Some(ChannelData::Composite(components)) => ordered_composite_entries(components)
            .into_iter()
            .map(|(_, channel)| channel)
            .collect(),
        None => Vec::new(),
    }
}

pub fn get_channel_entries_from_data(data: Option<&ChannelData>) -> Vec<(String, &AnimationChannel)> {
    match data {
        Some(ChannelData::Single(channel)) => vec![("value".to_string(), channel)],
        Some(ChannelData::Composite(components)) => ordered_composite_entries(components)
            .into_iter()
            .map(|(key, channel)| (key.clone(), channel))
            .collect(),
        None => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use scene::{AnimationChannel, DiscreteChannel, ScalarChannel};
    use std::collections::BTreeMap;

    fn scalar_channel() -> AnimationChannel {
        AnimationChannel::Scalar(ScalarChannel {
            keys: Vec::new(),
            extrapolation: None,
        })
    }

    #[test]
    fn leaf_data_yields_value_entry() {
        let data = ChannelData::Single(scalar_channel());
        let entries = get_channel_entries_from_data(Some(&data));
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].0, "value");
    }

    #[test]
    fn composite_entries_follow_canonical_component_order() {
        let mut components = BTreeMap::new();
        components.insert("a".to_string(), scalar_channel());
        components.insert("b".to_string(), scalar_channel());
        components.insert("g".to_string(), scalar_channel());
        components.insert("r".to_string(), scalar_channel());
        components.insert(
            "mode".to_string(),
            AnimationChannel::Discrete(DiscreteChannel { keys: Vec::new() }),
        );
        let data = ChannelData::Composite(components);
        let entries = get_channel_entries_from_data(Some(&data));
        let keys: Vec<&str> = entries.iter().map(|(key, _)| key.as_str()).collect();
        assert_eq!(keys, ["r", "g", "b", "a", "mode"]);
    }

    #[test]
    fn legacy_storage_keys_are_rejected() {
        assert!(!is_animation_storage_key("bindings"));
        assert!(!is_animation_storage_key("channels"));
        assert!(is_animation_storage_key("opacity"));
    }
}
