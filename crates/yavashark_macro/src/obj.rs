pub mod args;

use crate::config::Config;
use crate::custom_props::{match_list, match_prop, Act, List};
use crate::mutable_region::MutableRegion;
use proc_macro::TokenStream as TokenStream1;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use std::mem;
use darling::ast::NestedMeta;
use darling::FromMeta;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{FieldMutability, Fields, Token};
use crate::obj::args::{ItemArgs, ObjArgs};

pub fn object(attrs: TokenStream1, item: TokenStream1) -> TokenStream1 {
    let attr_args = match NestedMeta::parse_meta_list(attrs.clone().into()) {
        Ok(v) => v,
        Err(e) => { return TokenStream1::from(darling::Error::from(e).write_errors()); }
    };
    
    let args = match ObjArgs::from_list(&attr_args) {
        Ok(args) => args,
        Err(e) => return e.write_errors().into(),
    };

    let mut input: syn::ItemStruct = syn::parse_macro_input!(item);

    let conf = Config::new(Span::call_site());

    let realm = &conf.realm;
    let error = &conf.error;
    let variable = &conf.variable;
    let value = &conf.value;
    let value_result = &conf.value_result;
    let object_property = &conf.object_property;
    let mut_obj = &conf.mut_obj;


    let Fields::Named(fields) = &mut input.fields else {
        return syn::Error::new(input.span(), "Object must have named fields")
            .to_compile_error()
            .into();
    };
    
    let item_args = match ItemArgs::from(fields) {
        Ok(args) => args,
        Err(e) => return e.to_compile_error().into(),
    };



    let mut new_fields = Punctuated::new();

    let mut custom_mut = Vec::with_capacity(item_args.mutable_region.len());

    for field in mem::take(&mut fields.named) {
        if let Some(ident) = &field.ident {
            if !item_args.mutable_region.contains(ident) {
                new_fields.push(field);
            } else {
                custom_mut.push(field);
            }
        } else {
            new_fields.push(field);
        }
    }

    fields.named = new_fields;
    let direct = &args.direct.fields;

    let mut direct_region = Vec::with_capacity(direct.len());

    for item in direct {
        direct_region.push(item.field.clone());
    }

    let mutable_region = MutableRegion::with(direct_region, custom_mut, input.ident.clone(), args.extends.is_some());

    let region_ident = mutable_region.full_name();

    let inner_path: syn::Path = syn::parse_quote!(::core::cell::RefCell<#region_ident>);
    
    fields.named.push(syn::Field {
        attrs: Vec::new(),
        vis: syn::Visibility::Public(Token![pub](Span::call_site())),
        mutability: FieldMutability::None,
        ident: Some(Ident::new("inner", Span::call_site())),
        colon_token: None,
        ty: syn::Type::Path(syn::TypePath {
            qself: None,
            path: inner_path,
        }),
    }); 
    
    let downcast = if args.extends.is_some() {
        quote! {
            unsafe fn inner_downcast(&self, ty: ::core::any::TypeId) -> ::core::option::Option<::core::ptr::NonNull<()>> {
                if ty == ::core::any::TypeId::of::<Self>() {
                    ::core::option::Option::Some(::core::ptr::NonNull::from(self).cast())
                } else {
                    self.extends.inner_downcast(ty)
                }
            }
        }
    } else {
        TokenStream::new()
    };

    let (obj_path, inner_drop, inner_borrow, inner_borrow_mut) = if let Some(extends) = args.extends {
        fields.named.push(syn::Field {
            attrs: Vec::new(),
            vis: syn::Visibility::Public(Token![pub](Span::call_site())),
            mutability: FieldMutability::None,
            ident: Some(Ident::new("extends", Span::call_site())),
            colon_token: None,
            ty: syn::Type::Path(syn::TypePath {
                qself: None,
                path: extends,
            }),
        });

        (quote! { self.extends }, quote! { drop(inner);}, TokenStream::new(), TokenStream::new())
    } else {
        (quote! { inner.object }, TokenStream::new(), quote! { let inner = self.inner.borrow(); }, quote! { let mut inner = self.inner.borrow_mut(); })
    };

    let struct_name = &input.ident;

    let properties_define = match_prop(direct, Act::Set, value);
    let properties_variable_define = match_prop(direct, Act::SetVar, value);
    let properties_resolve = match_prop(direct, Act::None, value);
    let properties_get = match_prop(direct, Act::Get, value);
    let properties_contains = match_prop(direct, Act::Contains, value);
    let properties_delete = match_prop(direct, Act::Delete, value);

    let properties = match_list(direct, List::Properties, value);
    let keys = match_list(direct, List::Keys, value);
    let values = match_list(direct, List::Values, value);
    let clear = match_list(direct, List::Clear, value);

    let function = if args.function {
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

    let custom_refs = if item_args.gc.is_empty() {
        TokenStream::new()
    } else {
        let len = item_args.gc.len();

        let refs = item_args.gc
            .into_iter()
            .map(|gc| {
                let mut func = if gc.ty {
                    if gc.multi {
                        Ident::new("gc_ref_multi", Span::call_site())
                    } else {
                        Ident::new("gc_ref", Span::call_site())
                    }
                } else if gc.multi {
                    Ident::new("gc_untyped_ref_multi", Span::call_site())
                } else {
                    Ident::new("gc_untyped_ref", Span::call_site())
                };

                if let Some(f) = gc.func {
                    func = f;
                }

                let field = gc.name;
                
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

    let constructor = if args.constructor {
        quote! {
            fn construct(&self, realm: &mut #realm, args: Vec<#value>) -> Result<#value, #error> {
                yavashark_value::Constructor::construct(self, realm, args)
            }

            fn is_constructor(&self) -> bool {
                yavashark_value::Constructor::is_constructor(self)
            }

            // fn construct_proto(&self) -> Result<#object_property, #error> {
            //     yavashark_value::Constructor::constructor_proto(self)
            // }
        }
    } else {
        TokenStream::new()
    };

    let to_string = if args.to_string {
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

    let name = if args.name {
        quote! {
            fn name(&self) -> String {
                yavashark_value::CustomName::custom_name(self)
            }
        }
    } else {
        quote! {

            fn name(&self) -> String {
                stringify!(#struct_name).to_owned()
            }
        }
    };

    let primitive = if let Some(primitive) = item_args.primitive {
        let is_mutable = mutable_region.contains(&primitive);

        if is_mutable {
            quote! {
                fn primitive(&self) -> ::core::option::Option<#value> {
                    let inner = self.inner.borrow();

                    ::core::option::Option::Some(inner.#primitive.clone().into())
                }
            }
        } else {
            quote! {
                fn primitive(&self) -> ::core::option::Option<#value> {
                    ::core::option::Option::Some(self.#primitive.clone().into())
                }
            }
        }
    } else {
        TokenStream::new()
    };

    let region_code = mutable_region.generate(&conf, true);
    
    
   
    let expanded = quote! {
        use #mut_obj as _;
        #input
        #region_code

        impl yavashark_value::Obj<#realm> for #struct_name {
            fn define_property(&self, name: #value, value: #value) -> Result<(), #error> {
                let mut inner = self.inner.borrow_mut();
                #properties_define
                
                #inner_drop
                #obj_path.define_property(name, value)
            }

            fn define_variable(&self, name: #value, value: #variable) -> Result<(), #error> {
                let mut inner = self.inner.borrow_mut();
                #properties_variable_define

                #inner_drop
                #obj_path.define_variable(name, value)
            }

            fn resolve_property(&self, name: &#value) -> Result<Option<#object_property>, #error> {
                let inner = self.inner.borrow();
                #properties_resolve

                #obj_path.resolve_property(name)
            }

            fn get_property(&self, name: &#value) -> Result<Option<#object_property>, #error> {
                let inner = self.inner.borrow();
                #properties_get

                #obj_path.get_property(name)
            }

            fn define_getter(&self, name: #value, value: #value) -> Result<(), #error> {
                #inner_borrow_mut
                #obj_path.define_getter(name, value)
            }

            fn define_setter(&self, name: #value, value: #value) -> Result<(), #error> {
                #inner_borrow_mut
                #obj_path.define_setter(name, value)
            }

            fn get_getter(&self, name: &#value) -> Result<Option<#value>, #error> {
                let inner = self.inner.borrow();
                #obj_path.get_getter(name)
            }

            fn get_setter(&self, name: &#value) -> Result<Option<#value>, #error> {
                let inner = self.inner.borrow();
                #obj_path.get_setter(name)
            }

            fn delete_property(&self, name: &#value) -> Result<Option<#value>, #error> {
                let mut inner = self.inner.borrow_mut();
                #properties_delete
                
                #inner_drop
                #obj_path.delete_property(name)
            }


            fn contains_key(&self, name: &#value) -> Result<bool, #error> {
                let mut inner = self.inner.borrow_mut();
                #properties_contains
                
                #inner_drop
                #obj_path.contains_key(name)
            }


            #to_string
            #name

            fn properties(&self) -> Result<Vec<(#value, #value)>, #error> {
                let inner = self.inner.borrow();
                let mut props = #obj_path.properties()?;
                #properties

                Ok(props)
            }

            fn keys(&self) -> Result<Vec<#value>, #error> {
                let inner = self.inner.borrow();
                let mut keys = #obj_path.keys()?;
                #keys

                Ok(keys)
            }

            fn values(&self) -> Result<Vec<#value>, #error> {
                let inner = self.inner.borrow();
                let mut values = #obj_path.values()?;
                #values

                Ok(values)
            }
            fn get_array_or_done(&self, index: usize) -> Result<(bool, Option<#value>), #error> {
                let inner = self.inner.borrow();
                #obj_path.get_array_or_done(index)
            }

            fn clear_values(&self) -> Result<(), #error> {
                let mut inner = self.inner.borrow_mut();
                #clear
                
                #inner_drop
                #obj_path.clear_values()
            }

            fn prototype(&self) -> Result<#object_property, #error> {
                #inner_borrow
                #obj_path.prototype()
            }

            fn set_prototype(&self, proto: #object_property) -> Result<(), #error> {
                #inner_borrow_mut
                #obj_path.set_prototype(proto)
            }

            #constructor

            #function

            #custom_refs

            #primitive
            
            #downcast
        }
    };

    TokenStream1::from(expanded)
}
