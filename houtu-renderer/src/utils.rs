use std::collections::HashMap;

pub fn get_subdomain(
    subdomains: &Vec<&'static str>,
    key: &crate::quadtree::tile_key::TileKey,
) -> &'static str {
    return subdomains[(key.y + key.x + key.level) as usize % subdomains.len()];
}

pub fn map_to_param_str(map: &HashMap<&str, &str>) -> String {
    let mut params_str = String::new();
    map.iter().for_each(|(k, v)| {
        let param = format!("{}={}", k, v);
        if params_str == "" {
            params_str = format!("{}", param);
        } else {
            params_str = format!("{}&{}", params_str, param);
        }
    });
    return params_str;
}

pub fn key_value_iter_to_param_str(map: &Vec<(&str, &str)>) -> String {
    let mut params_str = String::new();
    map.iter().for_each(|(k, v)| {
        let param = format!("{}={}", k, v);
        if params_str == "" {
            params_str = format!("{}", param);
        } else {
            params_str = format!("{}&{}", params_str, param);
        }
    });
    return params_str;
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::map_to_param_str;

    #[test]
    fn test_map_to_param_str() {
        let map = HashMap::from([("key1", "value1"), ("key2", "value2")]);
        let param_str = map_to_param_str(&map);
        assert!(param_str == "key1=value1&key2=value2");
    }
}
