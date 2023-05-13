use mapreduce::TaskType;

pub mod mapreduce {
    tonic::include_proto!("mapreduce");
}
pub mod mr;
pub mod mrapps;

impl TryFrom<i32> for TaskType {
    type Error = ();

    fn try_from(v: i32) -> Result<Self, Self::Error> {
        match v {
            x if x == TaskType::None as i32 => Ok(TaskType::None),
            x if x == TaskType::Map as i32 => Ok(TaskType::Map),
            x if x == TaskType::Reduce as i32 => Ok(TaskType::Reduce),
            _ => Err(()),
        }
    }
}

// map functions return a vector of KeyValue
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct KeyValue {
    key: String,
    value: String,
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
