#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TopicsSnapshotProtobufModel {
    #[prost(message, repeated, tag = "1")]
    pub data: Vec<TopicSnapshotProtobufModel>,
}

impl TopicsSnapshotProtobufModel {
    pub fn create_default() -> Self {
        Self { data: Vec::new() }
    }
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TopicSnapshotProtobufModel {
    #[prost(string, tag = "1")]
    pub topic_id: ::prost::alloc::string::String,

    #[prost(int64, tag = "2")]
    pub message_id: i64,

    #[prost(int32, tag = "3")]
    pub not_used: i32,

    #[prost(message, repeated, tag = "4")]
    pub queues: Vec<QueueSnapshotProtobufModel>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueueSnapshotProtobufModel {
    #[prost(string, tag = "1")]
    pub queue_id: ::prost::alloc::string::String,

    #[prost(message, repeated, tag = "2")]
    pub ranges: Vec<QueueRangeProtobufModel>,

    #[prost(int32, tag = "3")]
    pub queue_type: i32,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueueRangeProtobufModel {
    #[prost(int64, tag = "1")]
    pub from_id: i64,

    #[prost(int64, tag = "2")]
    pub to_id: i64,
}
