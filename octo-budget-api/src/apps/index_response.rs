use serde::Serialize;
use serde_derive::Serialize;

#[derive(Serialize, Debug)]
pub struct Data<M: Serialize> {
    pub total: i64,
    pub results: Vec<M>,
    pub next: bool,
    pub previous: bool,
}
