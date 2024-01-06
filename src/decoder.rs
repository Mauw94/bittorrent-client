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
            number as usize,
        );
    }
}
