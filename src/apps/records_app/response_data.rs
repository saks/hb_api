use db::models::Record as RecordModel;

#[derive(Serialize, Debug)]
pub struct ResponseData {
    count: i64,
    results: Vec<RecordModel>,
}

impl ResponseData {
    pub fn new((results, count): (Vec<RecordModel>, i64)) -> Self {
        Self { count, results }
    }
}
