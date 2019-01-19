pub fn sort_tags(redis_tags: Vec<String>, user_tags: Vec<String>) -> Vec<String> {
    let mut result = Vec::with_capacity(user_tags.len());

    if user_tags.is_empty() {
        return result;
    }

    for item in redis_tags {
        if user_tags.contains(&item) {
            result.push(item);
        }
    }

    for tag in user_tags {
        if !result.contains(&tag) {
            result.push(tag);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tags_vec;

    #[test]
    fn sorting_tags_with_empty_user_tags() {
        let user_tags = tags_vec![];
        let redis_tags = tags_vec!["foo"];
        let sorted = sort_tags(redis_tags, user_tags);

        assert_eq!(tags_vec![], sorted);
    }

    #[test]
    fn sorting_tags_with_user_tags_not_matching_ones_from_redis() {
        let user_tags = tags_vec!["bar"];
        let redis_tags = tags_vec!["foo"];
        let sorted = sort_tags(redis_tags, user_tags);

        assert_eq!(tags_vec!["bar"], sorted);
    }

    #[test]
    fn sorting_tags_with_order_defined_by_redis_tags() {
        let user_tags = tags_vec!["foo", "bar", "buz"];
        let redis_tags = tags_vec!["buz", "foo", "bar"];
        let sorted = sort_tags(redis_tags, user_tags);

        assert_eq!(tags_vec!["buz", "foo", "bar"], sorted);
    }
}
