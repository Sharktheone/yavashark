use proc_macro::{TokenStream as TokenStream1};
use proc_macro2::TokenStream;
use syn::{ImplItem, LitBool};
use syn::spanned::Spanned;

pub fn properties(_: TokenStream1, item: TokenStream1) -> TokenStream1 {
    
    let item: syn::ItemImpl = syn::parse_macro_input!(item);
    
    let mut call = None;
    let mut constructor = None;
    
    struct Property {
        name: syn::Ident,
        writable: bool,
        enumerable: bool,
        configurable: bool,
    }
    
    let mut properties = Vec::new();
    
    for func in &item.items {
        let ImplItem::Fn(func) = func else {
            continue;
        };
        
        let mut writable = false;
        let mut enumerable = false;
        let mut configurable = false;
        
        let mut name = None;
        
        for attr in &func.attrs {
            if attr.path().is_ident("call") {
                call = Some(func.sig.ident.clone());
                continue;
            }
            if attr.path().is_ident("constructor") {
                constructor = Some(func.sig.ident.clone());
                continue;
            }
            
            if attr.path().is_ident("attributes") {
                let attr_parser = syn::meta::parser(|meta| {
                    if meta.path.is_ident("writable") {
                        writable = meta.value()?.parse::<LitBool>()?.value();
                        return Ok(())
                    }
                    
                    if meta.path.is_ident("enumerable") {
                        enumerable = meta.value()?.parse::<LitBool>()?.value();
                        return Ok(())
                    }
                    
                    if meta.path.is_ident("configurable") {
                        configurable = meta.value()?.parse::<LitBool>()?.value();
                        return Ok(())
                    }
                    
                    Err(syn::Error::new(meta.path.span(), "Unknown attribute"))
                });
                
                attr.parse_args_with(attr_parser).expect("Failed to parse attributes");
                continue;
            }
            
            if attr.path().is_ident("name") {
                name = Some(attr.meta.path().get_ident().expect("Expected identifier").clone());
                continue;
            }
            
        }
    
        let Some(name) = name else {
            continue;
        };
        
        let prop = Property {
            name,
            writable,
            enumerable,
            configurable,
        };
        
        properties.push(prop);
    }
    
    
    for func in &item.items {
        let ImplItem::Fn(_func) = func else {
            continue;
        };
        
        
        
        todo!("Walk through the function block and replace Self::DIRECT_PROPERTIES with the properties");
    }
    
    
    
    
    TokenStream::new().into()
}