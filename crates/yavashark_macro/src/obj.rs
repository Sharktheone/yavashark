use crate::config::Config;
use crate::custom_props::{match_list, match_prop, Act, List};
use proc_macro::TokenStream as TokenStream1;
use std::mem;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::spanned::Spanned;
use syn::{FieldMutability, Fields};
use syn::punctuated::Punctuated;
use crate::mutable_region::MutableRegion;

pub fn object(attrs: TokenStream1, item: TokenStream1) -> TokenStream1 {
    let mut input: syn::ItemStruct = syn::parse_macro_input!(item);
    let mut proto = false;
    let mut direct = Vec::new();
    let mut constructor = false;

    let conf = Config::new(Span::call_site());

    let realm = &conf.realm;
    let error = &conf.error;
    let variable = &conf.variable;
    let value = &conf.value;
    let value_result = &conf.value_result;
    let object_property = &conf.object_property;

    let mut gc = Vec::new();
    let mut mutable_region = Vec::new();

    let mut function = false;
    let mut to_string = false;
    let mut name = false;

    let attr_parser = syn::meta::parser(|meta| {
        if meta.path.is_ident("prototype") {
            proto = true;
            return Ok(());
        }
        if meta.path.is_ident("direct") {
            meta.parse_nested_meta(|meta| {
                let mut rename = None;

                let _ = meta.parse_nested_meta(|meta| {
                    rename = Some(meta.path);
                    Ok(())
                });

                direct.push((meta.path, rename));

                Ok(())
            })?;
            return Ok(());
        }

        if meta.path.is_ident("function") {
            function = true;
            return Ok(());
        }

        if meta.path.is_ident("constructor") {
            constructor = true;
            return Ok(());
        }

        if meta.path.is_ident("to_string") {
            to_string = true;
            return Ok(());
        }

        if meta.path.is_ident("name") {
            name = true;
            return Ok(());
        }

        Err(syn::Error::new(meta.path.span(), "Unknown attribute"))
    });

    syn::parse_macro_input!(attrs with attr_parser);

    let Fields::Named(fields) = &mut input.fields else {
        return syn::Error::new(input.span(), "Object must have named fields")
            .to_compile_error()
            .into();
    };

    for f in &mut fields.named {
        let mut err = None;
        f.attrs.retain_mut(|attr| {
            if attr.meta.path().is_ident("gc") {
                let mut ty = true;
                let mut func = None;
                let mut multi = false;

                if !matches!(attr.meta, syn::Meta::Path(_)) {
                    if let Err(e) = attr.parse_nested_meta(|meta| {
                        if meta.path.is_ident("untyped") {
                            ty = false;
                            return Ok(());
                        }

                        if meta.path.is_ident("func") {
                            func = Some(
                                meta.path
                                    .get_ident()
                                    .cloned()
                                    .ok_or(syn::Error::new(meta.path.span(), "Expected ident"))?,
                            );
                            return Ok(());
                        }

                        if meta.path.is_ident("multi") {
                            multi = true;
                            return Ok(());
                        }

                        Err(syn::Error::new(meta.path.span(), "Unknown attribute"))
                    }) {
                        err = Some(e);
                        return false;
                    };
                }

                let id = match f
                    .ident
                    .as_ref()
                    .ok_or(syn::Error::new(attr.meta.span(), "Expected ident"))
                {
                    Ok(id) => id,
                    Err(e) => {
                        err = Some(e);
                        return false;
                    }
                }
                .clone();

                gc.push((id, ty, multi, func));

                return false;
            }

            
            if attr.meta.path().is_ident("mutable") {
                let Ok(ident) = f.ident.clone().ok_or(syn::Error::new(attr.span(), "Expected ident")) else {
                    err = Some(syn::Error::new(attr.span(), "Expected ident"));
                    return false;
                };
                
                mutable_region.push(ident);
                return false;
            }
            
            true
        });

        if let Some(e) = err {
            return e.to_compile_error().into();
        }
    }
    
    let mut new_fields= Punctuated::new();
    
    let mut custom_mut = Vec::with_capacity(mutable_region.len());
    
    for field in mem::take(&mut fields.named) {
        if let Some(ident) = &field.ident {
            if !mutable_region.contains(ident) {
                new_fields.push(field);
            }  else {
                custom_mut.push(field);
                
            }
        } else {
            new_fields.push(field);
        }
        
    }
    
    fields.named = new_fields;
    
    
    let mutable_region = MutableRegion::with(Vec::new(), custom_mut, input.ident.clone());
    
    let region_ident = mutable_region.full_name();

    fields.named.push(syn::Field {
        attrs: Vec::new(),
        vis: syn::Visibility::Inherited,
        mutability: FieldMutability::None,
        ident: Some(Ident::new("inner", Span::call_site())),
        colon_token: None,
        ty: syn::Type::Path(syn::TypePath {
            qself: None,
            path: region_ident.into(),
        }),
    });

    for (path, _) in &direct {
        fields.named.push(syn::Field {
            attrs: Vec::new(),
            vis: syn::Visibility::Inherited,
            mutability: FieldMutability::None,
            ident: path.get_ident().cloned(),
            colon_token: None,
            ty: syn::Type::Path(syn::TypePath {
                qself: None,
                path: object_property.clone(),
            }),
        });
    }

    let struct_name = &input.ident;

    let properties_define = match_prop(&direct, Act::Set, value);
    let properties_variable_define = match_prop(&direct, Act::SetVar, value);
    let properties_resolve = match_prop(&direct, Act::None, value);
    let properties_get = match_prop(&direct, Act::Ref, value);
    let properties_contains = match_prop(&direct, Act::Contains, value);
    let properties_delete = match_prop(&direct, Act::Delete, value);

    let properties = match_list(&direct, List::Properties, value);
    let keys = match_list(&direct, List::Keys, value);
    let values = match_list(&direct, List::Values, value);
    let clear = match_list(&direct, List::Clear, value);

    let function = if function {
        quote! {
            fn call(&self, realm: &mut #realm, args: Vec< #value>, this: #value) -> #value_result {
                yavashark_value::Func::call(self, realm, args, this)
            }

            fn is_function(&self) -> bool {
                true
            }
        }
    } else {
        quote! {}
    };

    let custom_refs = if gc.is_empty() {
        TokenStream::new()
    } else {
        let len = gc.len();

        let refs = gc
            .into_iter()
            .map(|gc| {
                let mut func = if gc.1 {
                    if gc.2 {
                        Ident::new("gc_ref_multi", Span::call_site())
                    } else {
                        Ident::new("gc_ref", Span::call_site())
                    }
                } else if gc.2 {
                    Ident::new("gc_untyped_ref_multi", Span::call_site())
                } else {
                    Ident::new("gc_untyped_ref", Span::call_site())
                };

                if let Some(f) = gc.3 {
                    func = f;
                }

                let field = gc.0;

                quote! {
                    if let Some(r) = self.#field.#func() {
                        refs.push(r);
                    }
                }
            })
            .collect::<TokenStream>();

        quote! {
            unsafe fn custom_gc_refs(&self) -> Vec<yavashark_garbage::GcRef<yavashark_value::BoxedObj<#realm >>> {
                use yavashark_value::{CustomGcRef, CustomGcRefUntyped};
                let mut refs = Vec::with_capacity(#len);

                #refs

                refs
            }
        }
    };

    let constructor = if constructor {
        quote! {
            fn constructor(&self) -> Result<#object_property, #error> {
                yavashark_value::Constructor::get_constructor(self)
            }

            fn get_constructor_proto(&self, realm: &mut #realm) -> Result<Option<#value>, #error> {
                Some(yavashark_value::Constructor::proto(self, realm))
            }

            fn special_constructor(&self) -> bool {
                yavashark_value::Constructor::special_constructor(self)
            }

            fn get_constructor_value(&self, realm: &mut #realm) -> Result<Option<#value>, #error> {
                Some(yavashark_value::Constructor::value(self, realm))
            }
        }
    } else {
        quote! {
            fn constructor(&self) -> Result<#object_property, #error> {
                self.object.constructor()
            }
        }
    };

    let to_string = if to_string {
        quote! {
            fn to_string(&self, realm: &mut #realm) -> Result<String, #error> {
                self.override_to_string(realm)
            }

            fn to_string_internal(&self) -> Result<String, #error> {
                self.override_to_string_internal()
            }
        }
    } else {
        quote! {
            fn to_string(&self, realm: &mut #realm) -> Result<String, #error> {
                Ok(format!("[object {}]", self.name()))
            }

            fn to_string_internal(&self) -> Result<String, #error> {
                Ok(format!("[object {}]", self.name()))
            }
        }
    };

    let name = if name {
        quote! {
            fn name(&self) -> String {
                yavashark_value::CustomName::custom_name(self)
            }
        }
    } else {
        quote! {

            fn name(&self) -> String {
                self.object.name()
            }
        }
    };
    
    let region_code = mutable_region.generate(&conf, true);

    let expanded = quote! {
        #input
        #region_code

        impl yavashark_value::Obj<#realm> for #struct_name {
            fn define_property(&self, name: #value, value: #value) -> Result<(), #error> {
                #properties_define
                self.object.define_property(name, value)
            }

            fn define_variable(&self, name: #value, value: #variable) -> Result<(), #error> {
                #properties_variable_define

                self.object.define_variable(name, value)
            }

            fn resolve_property(&self, name: &#value) -> Result<Option<#object_property>, #error> {
                #properties_resolve
                self.object.resolve_property(name)
            }

            fn get_property(&self, name: &#value) -> Result<Option<#value>, #error> {
                #properties_get
                self.object.get_property(name)
            }

            fn define_getter(&self, name: #value, value: #value) -> Result<(), #error> {
                self.object.define_getter(name, value)
            }

            fn define_setter(&self, name: #value, value: #value) -> Result<(), #error> {
                self.object.define_setter(name, value)
            }

            fn get_getter(&self, name: &#value) -> Result<Option<#value>, #error> {
                self.object.get_getter(name)
            }

            fn get_setter(&self, name: &#value) -> Result<Option<#value>, #error> {
                self.object.get_setter(name)
            }

            fn delete_property(&self, name: &#value) -> Result<Option<#value>, #error> {
                #properties_delete
                self.object.delete_property(name)
            }


            fn contains_key(&self, name: &#value) -> Result<bool, #error> {
                #properties_contains
                self.object.contains_key(name)
            }


            #to_string
            #name

            fn properties(&self) -> Result<Vec<(#value, #value)>, #error> {
                let mut props = self.object.properties()?;
                #properties

                Ok(props)
            }

            fn keys(&self) -> Result<Vec<#value>, #error> {
                let mut keys = self.object.keys()?;
                #keys

                Ok(keys)
            }

            fn values(&self) -> Result<Vec<#value>, #error> {
                let mut values = self.object.values()?;
                #values

                Ok(values)
            }
            fn get_array_or_done(&self, index: usize) -> Result<(bool, Option<#value>), #error> {
                self.object.get_array_or_done(index)
            }

            fn clear_values(&self) -> Result<(), #error> {
                #clear
                self.object.clear_values()
            }

            fn prototype(&self) -> Result<#object_property, #error> {
                self.object.prototype()
            }
            #constructor

            #function

            #custom_refs
        }
    };

    TokenStream1::from(expanded)
}
