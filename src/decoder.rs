use serde_json::Map;

pub struct BencodedValue {
    pub value: serde_json::Value,
}

impl BencodedValue {
    pub fn decode(encoded_value: &str) -> Self {
        let (value, _) = Self::decode_bencoded_value(encoded_value);
        Self { value }
    }

    fn decode_bencoded_value(encoded_value: &str) -> (serde_json::Value, usize) {
        let mut chars = encoded_value.chars();
        let chr = chars.next().unwrap();

        if chr.is_digit(10) {
            Self::decode_bencoded_string(encoded_value)
        } else if chr == 'i' {
            Self::decode_bencoded_interger(encoded_value)
        } else if chr == 'l' {
            Self::decode_bencoded_array(encoded_value)
        } else if chr == 'd' {
            Self::decode_bencoded_dictionary(encoded_value)
        } else {
            panic!("Unhandled encoded value: {}", encoded_value)
        }
    }

    // example: 5:hello
    fn decode_bencoded_string(encoded_value: &str) -> (serde_json::Value, usize) {
        let colon_index = encoded_value.find(':').unwrap();
        let number_string = &encoded_value[..colon_index];
        let number = number_string.parse::<i64>().unwrap();
        let string = &encoded_value[colon_index + 1..colon_index + 1 + number as usize];

        return (
            serde_json::Value::String(string.to_string()),
            colon_index + 1 + number as usize,
        );
    }

    // example: i52e
    fn decode_bencoded_interger(encoded_value: &str) -> (serde_json::Value, usize) {
        let e_index = encoded_value.find('e').unwrap();
        let number_string = &encoded_value[1..e_index];
        let number = number_string.parse::<i64>().unwrap();

        return (serde_json::Value::Number(number.into()), e_index + 1);
    }

    // example l5:helloi52ee
    fn decode_bencoded_array(encoded_value: &str) -> (serde_json::Value, usize) {
        let mut result = Vec::new();
        let mut read = 1;

        while read < encoded_value.len() - 1 {
            let (value, size) = Self::decode_bencoded_value(&encoded_value[read..]);
            result.push(value);
            read += size;
            if encoded_value.as_bytes()[read] == b'e' {
                break;
            }
        }

        return (serde_json::Value::Array(result), read + 1);
    }

    // example d3:foo3:bar5:helloi52ee
    fn decode_bencoded_dictionary(encoded_value: &str) -> (serde_json::Value, usize) {
        let mut map: Map<String, serde_json::Value> = Map::new();
        let mut read = 1;
        let mut read_key = true;
        let mut curr_key: String = String::new();

        while read < encoded_value.len() - 1 {
            if read_key {
                let (key, size) = Self::decode_bencoded_value(&encoded_value[read..]);
                if let serde_json::Value::String(key_str) = key {
                    curr_key = key_str;
                } else {
                    panic!("Key is not a string.");
                }
                read += size;
            } else {
                let (value, size) = Self::decode_bencoded_value(&encoded_value[read..]);
                map.insert(curr_key.clone(), value);
                read += size;
            }
            if encoded_value.as_bytes()[read] == b'e' {
                break;
            }
            read_key = !read_key;
        }

        return (serde_json::Value::Object(map), read + 1);
    }
}
