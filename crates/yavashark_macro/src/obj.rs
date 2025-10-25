pub mod args;

use crate::config::Config;
use crate::custom_props::{match_list, match_prop, Act, List};
use crate::mutable_region::MutableRegion;
use crate::obj::args::{ItemArgs, ObjArgs};
use darling::ast::NestedMeta;
use darling::FromMeta;
use proc_macro::TokenStream as TokenStream1;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use std::mem;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{FieldMutability, Fields, Token};

pub fn object(attrs: TokenStream1, item: TokenStream1) -> TokenStream1 {
    let attr_args = match NestedMeta::parse_meta_list(attrs.clone().into()) {
        Ok(v) => v,
        Err(e) => {
            return TokenStream1::from(darling::Error::from(e).write_errors());
        }
    };

    let args = match ObjArgs::from_list(&attr_args) {
        Ok(args) => args,
        Err(e) => return e.write_errors().into(),
    };

    let mut input: syn::ItemStruct = syn::parse_macro_input!(item);

    let conf = Config::new(Span::call_site());

    let realm = &conf.realm;
    let variable = &conf.variable;
    let value = &conf.value;
    let value_result = &conf.value_result;
    let mut_obj = &conf.mut_obj;
    let env = &conf.env_path;
    let internal_property_key = &conf.internal_property_key;
    let property_key = &conf.property_key;
    let primitive_value = &conf.primitive_value;
    let res = &conf.res;
    let object_or_null = &conf.object_or_null;
    let object_handle = &conf.object_handle;
    let property = &conf.property;
    let define_property_result = &conf.define_property_result;

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

    let mutable_region = MutableRegion::with(
        direct_region,
        custom_mut,
        input.ident.clone(),
        args.extends.is_some(),
    );

    let region_ident = mutable_region.full_name();

    let inner_path: syn::Path = syn::parse_quote!(::core::cell::RefCell<#region_ident>);

    fields.named.insert(
        0,
        syn::Field {
            attrs: Vec::new(),
            vis: syn::Visibility::Public(Token![pub](Span::call_site())),
            mutability: FieldMutability::None,
            ident: Some(Ident::new("inner", Span::call_site())),
            colon_token: None,
            ty: syn::Type::Path(syn::TypePath {
                qself: None,
                path: inner_path,
            }),
        },
    );

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

    let (obj_path, inner_drop, inner_borrow, inner_borrow_mut) = if let Some(extends) = args.extends
    {
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

        (
            quote! { self.extends },
            quote! { drop(inner);},
            TokenStream::new(),
            TokenStream::new(),
        )
    } else {
        (
            quote! { inner.object },
            TokenStream::new(),
            quote! { let inner = self.inner.borrow(); },
            quote! { let mut inner = self.inner.borrow_mut(); },
        )
    };

    let struct_name = &input.ident;

    let properties_define = match_prop(direct, Act::Set, &conf);
    let properties_variable_define = match_prop(direct, Act::SetVar, &conf);
    let properties_resolve = match_prop(direct, Act::None, &conf);
    let properties_get = match_prop(direct, Act::Get, &conf);
    let properties_contains = match_prop(direct, Act::Contains, &conf);
    let properties_delete = match_prop(direct, Act::Delete, &conf);

    let properties = match_list(direct, List::Properties, &conf);
    let keys = match_list(direct, List::Keys, &conf);
    let values = match_list(direct, List::Values, &conf);
    let clear = match_list(direct, List::Clear, &conf);

    let function = if args.function {
        quote! {
            fn call(&self, args: ::std::vec::Vec< #value>, this: #value, realm: &mut #realm) -> #value_result {
                #env::value::Func::call(self, realm, args, this)
            }

            fn is_callable(&self) -> bool {
                true
            }
        }
    } else {
        quote! {}
    };

    // let custom_refs = if item_args.gc.is_empty() {
    //     TokenStream::new()
    // } else {
    //     let len = item_args.gc.len();
    //
    //     let refs = item_args
    //         .gc
    //         .into_iter()
    //         .map(|gc| {
    //             let mut func = if gc.ty {
    //                 if gc.multi {
    //                     Ident::new("gc_ref_multi", Span::call_site())
    //                 } else {
    //                     Ident::new("gc_ref", Span::call_site())
    //                 }
    //             } else if gc.multi {
    //                 Ident::new("gc_untyped_ref_multi", Span::call_site())
    //             } else {
    //                 Ident::new("gc_untyped_ref", Span::call_site())
    //             };
    //
    //             if let Some(f) = gc.func {
    //                 func = f;
    //             }
    //
    //             let field = gc.name;
    //
    //             quote! {
    //                 if let Some(r) = self.#field.#func() {
    //                     refs.push(r);
    //                 }
    //             }
    //         })
    //         .collect::<TokenStream>();
    //
    //     quote! {
    //         //TODO
    //     }
    // };

    let constructor = if args.constructor {
        quote! {
            fn construct(&self, args: ::std::vec::Vec<#value>, realm: &mut #realm) -> #res<#object_handle> {
                #env::value::Constructor::construct(self, realm, args)
            }

            fn is_constructable(&self) -> bool {
                #env::value::Constructor::is_constructable(self)
            }

            // fn construct_proto(&self) -> Result<#object_property, #error> {
            //     #env::value::Constructor::constructor_proto(self)
            // }
        }
    } else {
        TokenStream::new()
    };

    let name = if args.name {
        quote! {
            fn name(&self) -> String {
                #env::value::CustomName::custom_name(self)
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
                fn primitive(&self, realm: &mut #realm) -> #res<::core::option::Option<#primitive_value>> {
                    let inner = self.inner.borrow();

                    ::core::result::Result::Ok(::core::option::Option::Some(inner.#primitive.clone().into()))
                }
            }
        } else {
            quote! {
                fn primitive(&self, realm: &mut #realm) -> #res<::core::option::Option<#primitive_value>> {
                    ::core::result::Result::Ok(::core::option::Option::Some(self.#primitive.clone().into()))
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

        impl #env::value::Obj for #struct_name {
            fn define_property(&self, name: #internal_property_key, value: #value, realm: &mut #realm) -> #res<#define_property_result> {
                let mut inner = self.inner.borrow_mut();
                #properties_define

                #inner_drop
                #obj_path.define_property(name, value, realm)
            }

            fn define_property_attributes(&self, name: #internal_property_key, value: #variable, realm: &mut #realm) -> #res<#define_property_result> {
                let mut inner = self.inner.borrow_mut();
                #properties_variable_define

                #inner_drop
                #obj_path.define_property_attributes(name, value, realm)
            }

            fn resolve_property(&self, name: #internal_property_key, realm: &mut #realm) -> #res<Option<#property>> {
                let inner = self.inner.borrow();
                #properties_resolve

                #obj_path.resolve_property(name, realm)
            }

            fn get_own_property(&self, name: #internal_property_key, realm: &mut #realm) -> #res<Option<#property>> {
                let inner = self.inner.borrow();
                #properties_get

                #obj_path.get_own_property(name, realm)
            }

            fn define_getter(&self, name: #internal_property_key, getter: #object_handle, realm: &mut #realm) -> #res {
                #inner_borrow_mut
                #obj_path.define_getter(name, getter, realm)
            }

            fn define_setter(&self, name: #internal_property_key, setter: #object_handle, realm: &mut #realm) -> #res {
                #inner_borrow_mut
                #obj_path.define_setter(name, setter, realm)
            }

            fn delete_property(&self, name: #internal_property_key, realm: &mut #realm) -> #res<Option<#property>> {
                let mut inner = self.inner.borrow_mut();
                #properties_delete

                #inner_drop
                #obj_path.delete_property(name, realm)
            }


            fn contains_own_key(&self, name: #internal_property_key, realm: &mut Realm) -> #res<bool> {
                let mut inner = self.inner.borrow_mut();
                #properties_contains

                #inner_drop
                #obj_path.contains_own_key(name, realm)
            }


            fn contains_key(&self, name: #internal_property_key, realm: &mut Realm) -> #res<bool> {
                let mut inner = self.inner.borrow_mut();
                #properties_contains

                #inner_drop
                #obj_path.contains_key(name, realm)
            }


            fn properties(&self, realm: &mut #realm) -> #res<::std::vec::Vec<(#property_key, #value)>> {
                let inner = self.inner.borrow();
                let mut props = #obj_path.properties(realm)?;
                #properties

                Ok(props)
            }

            fn keys(&self, realm: &mut #realm) -> #res<::std::vec::Vec<#property_key>> {
                let inner = self.inner.borrow();
                let mut keys = #obj_path.keys(realm)?;
                #keys

                Ok(keys)
            }

            fn values(&self, realm: &mut #realm) -> #res<::std::vec::Vec<#value>> {
                let inner = self.inner.borrow();
                let mut values = #obj_path.values(realm)?;
                #values

                Ok(values)
            }


            fn enumerable_properties(&self, realm: &mut #realm) -> #res<::std::vec::Vec<(#property_key, #value)>> {
                let inner = self.inner.borrow();
                let mut props = #obj_path.properties(realm)?;
                #properties

                Ok(props)
            }

            fn enumerable_keys(&self, realm: &mut #realm) -> #res<::std::vec::Vec<#property_key>> {
                let inner = self.inner.borrow();
                let mut keys = #obj_path.keys(realm)?;
                #keys

                Ok(keys)
            }

            fn enumerable_values(&self, realm: &mut #realm) -> #res<::std::vec::Vec<#value>> {
                let inner = self.inner.borrow();
                let mut values = #obj_path.values(realm)?;
                #values

                Ok(values)
            }

            fn clear_properties(&self, realm: &mut #realm) -> #res {
                let mut inner = self.inner.borrow_mut();
                #clear

                #inner_drop
                #obj_path.clear_properties(realm)
            }

            fn get_array_or_done(&self, index: usize, realm: &mut #realm) -> #res<(bool, Option<#value>)> {
                let mut inner = self.inner.borrow_mut();
                #obj_path.get_array_or_done(index, realm)
            }


            fn prototype(&self, realm: &mut #realm) -> #res<#object_or_null> {
                #inner_borrow
                #obj_path.prototype(realm)
            }

            fn set_prototype(&self, proto: #object_or_null, realm: &mut #realm) -> #res {
                #inner_borrow_mut
                #obj_path.set_prototype(proto, realm)
            }

            #constructor

            #function

            #primitive

            #name

            #downcast

            fn is_extensible(&self) -> bool {
                #inner_borrow
                #obj_path.is_extensible()
            }

            fn prevent_extensions(&self) -> #res {
                #inner_borrow_mut
                #obj_path.prevent_extensions()
            }

            fn is_frozen(&self) -> bool {
                #inner_borrow
                #obj_path.is_frozen()
            }

            fn freeze(&self) -> #res {
                #inner_borrow_mut
                #obj_path.freeze()
            }

            fn is_sealed(&self) -> bool {
                #inner_borrow
                #obj_path.is_sealed()
            }

            fn seal(&self) -> #res {
                #inner_borrow_mut
                #obj_path.seal()
            }

            fn gc_refs(&self) -> ::std::vec::Vec<yavashark_garbage::GcRef<#env::value::BoxedObj>> {
                #inner_borrow
                #obj_path.gc_refs()
            }
        }
    };

    TokenStream1::from(expanded)
}
