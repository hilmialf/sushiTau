pub mod api {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct Order {
        pub id: String,
        pub table_id: SmallId,
        pub menu_id: SmallId,
        pub created_at: u64,
        pub processing_time: u64,
        pub status: OrderStatus
    }

    #[derive(PartialEq, Eq, Hash, Debug, Clone, Serialize, Deserialize)]
    pub enum OrderStatus {
        READY,
        PROCESSING,
        CANCELLED
    }


    #[derive(Debug)]
    pub struct Menu {
        pub id: SmallId,
        pub name: String,
    }

    pub type SmallId = u16;
}