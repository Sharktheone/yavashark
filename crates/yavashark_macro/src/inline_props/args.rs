use darling::ast::NestedMeta;
use darling::FromMeta;

#[derive(Debug, FromMeta)]
struct InnerInlinePropsArgs {
    #[darling(default)]
    pub enumerable: Option<bool>,
    #[darling(default)]
    pub configurable: Option<bool>,
    #[darling(default)]
    pub readonly: Option<bool>,
    #[darling(default)]
    pub partial: Option<bool>,
}

#[derive(Debug, Clone, Copy)]
pub struct InlinePropsArgs {
    pub enumerable: bool,
    pub configurable: bool,
    pub readonly: bool,
    pub partial: bool,
}

impl FromMeta for InlinePropsArgs {
    fn from_list(meta: &[NestedMeta]) -> darling::Result<Self> {
        let inner = InnerInlinePropsArgs::from_list(meta)?;

        Ok(InlinePropsArgs {
            enumerable: inner.enumerable.unwrap_or(true),
            configurable: inner.configurable.unwrap_or(true),
            readonly: inner.readonly.unwrap_or(false),
            partial: inner.partial.unwrap_or(false),
        })
    }
}