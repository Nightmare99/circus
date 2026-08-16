pub fn slugify(input: &str) -> String {
    let mut slug = String::new();
    let mut last_dash = false;
    for c in input.to_lowercase().chars() {
        if c.is_ascii_alphanumeric() {
            slug.push(c);
            last_dash = false;
        } else if !last_dash && !slug.is_empty() {
            slug.push('-');
            last_dash = true;
        }
    }
    while slug.ends_with('-') {
        slug.pop();
    }
    if slug.is_empty() {
        "org".to_string()
    } else {
        slug
    }
}

pub async fn unique_org_slug(pool: &sqlx::PgPool, name: &str) -> Result<String, sqlx::Error> {
    let base = slugify(name);
    let mut candidate = base.clone();
    let mut n = 2;
    while db::orgs::find_by_slug(pool, &candidate).await?.is_some() {
        candidate = format!("{base}-{n}");
        n += 1;
    }
    Ok(candidate)
}

/// A short opaque random token (~244 bits of entropy) for invite links.
pub fn random_token() -> String {
    format!(
        "{}{}",
        uuid::Uuid::new_v4().simple(),
        uuid::Uuid::new_v4().simple()
    )
}
