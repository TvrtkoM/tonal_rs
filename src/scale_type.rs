#[derive(Debug, Clone)]
pub struct ScaleType {
    pub(crate) name: String,
    pub(crate) set_num: i32,
    pub(crate) chroma: String,
    pub(crate) normalized: String,
    pub(crate) intervals: Vec<String>,
    pub(crate) aliases: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
pub struct ScaleTypeParts<'a> {
    pub name: &'a str,
    pub set_num: i32,
    pub chroma: &'a str,
    pub normalized: &'a str,
    pub intervals: &'a [String],
    pub aliases: &'a [String],
}
