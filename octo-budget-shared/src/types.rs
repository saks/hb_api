pub mod index_response {
    use serde::Serialize;

    #[derive(Serialize, Debug)]
    pub struct Data<M: Serialize> {
        pub total: i64,
        pub results: Vec<M>,
        pub next: bool,
        pub previous: bool,
    }
}
