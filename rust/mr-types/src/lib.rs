// map functions return a vector of KeyValue
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct KeyValue {
    pub key: String,
    pub value: String,
}

impl Ord for KeyValue {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (&self.key, &self.value).cmp(&(&other.key, &other.value))
    }
}

impl PartialOrd for KeyValue {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for KeyValue {
    fn eq(&self, other: &Self) -> bool {
        (&self.key, &self.value) == (&other.key, &other.value)
    }
}

impl Eq for KeyValue {}

pub type MapFunc = unsafe fn(String, String) -> Vec<KeyValue>;
pub type ReduceFunc = unsafe fn(String, Vec<String>) -> String;
