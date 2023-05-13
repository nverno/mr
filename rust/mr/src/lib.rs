use mapreduce::TaskType;

pub mod mapreduce {
    tonic::include_proto!("mapreduce");
}
pub mod mr;

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
