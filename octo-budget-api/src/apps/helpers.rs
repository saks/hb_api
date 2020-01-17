pub fn sort_tags(redis_tags: &[String], user_tags: &[String]) -> Vec<String> {
    let mut result = Vec::with_capacity(user_tags.len());

    if user_tags.is_empty() {
        return result;
    }

    for item in redis_tags {
        if user_tags.contains(&item) {
            result.push(item.to_owned());
        }
    }

    for tag in user_tags {
        if !result.contains(&tag) {
            result.push(tag.to_owned());
        }
    }

    result
}

#[cfg(test)]
mod tests;
