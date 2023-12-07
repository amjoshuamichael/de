use serde::*;

#[derive(Default, Serialize, Deserialize)]
struct One {
    l: f32,
    r: f32,
    #[serde(flatten)] two_instance: Two,
}

#[derive(Default, Serialize, Deserialize)]
struct Two {
    o: f32,
    p: f32,
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn ron_serialization() {
        let one = One::default();
        let one_serialized = ron::ser::to_string(&one).expect("serialization error");
        ron::de::from_str::<One>(&*one_serialized).expect("deserialization error");
    }

    use serde_json as json;

    #[test]
    fn json_serialization() {
         let one = One::default();
        let one_serialized = json::ser::to_string(&one).expect("serialization error");
        json::de::from_str::<One>(&*one_serialized).expect("deserialization error");
    }
}
