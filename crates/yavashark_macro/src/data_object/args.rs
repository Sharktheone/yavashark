use darling::FromMeta;

#[derive(Debug, FromMeta)]
pub struct DataObjectArgs {
    #[darling(default)]
    pub error: Option<String>,
}
