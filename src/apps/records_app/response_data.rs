use db::models::Record as RecordModel;

#[derive(Serialize, Debug)]
pub struct ResponseData {
    pub total: i64,
    pub results: Vec<RecordModel>,
    pub next: bool,
    pub previous: bool,
}

#[cfg(test)]
mod test {
    // use super::*;

    // #[test]
    // fn first_page_has_no_prevoius() {
    //     let data = ResponseData::new((vec![], 10, 10, 10));
    //     assert!(!data.previous);
    // }
    //
    // #[test]
    // fn first_page_has_prevoius() {
    //     let data = ResponseData::new(vec![], 10, 2, 10);
    //     assert!(data.previous);
    // }
    //
    // #[test]
    // fn last_page_has_no_next() {
    //     let data = ResponseData::new(vec![], 10, 2, 10);
    //     assert!(data.previous);
    // }
}
