use proc_macro2::{Ident, TokenStream};
use quote::quote;

const RUST_TYPES: [&str; 12] = [
    "u8", "u8", "u16", "u32", "u64", "i8", "i16", "i32", "i64", "f16", "f32", "f64",
];

const TYPES: [&str; 12] = [
    "U8C", "U8", "U16", "U32", "U64", "I8", "I16", "I32", "I64", "F16", "F32", "F64",
];

pub fn typed_array_run(input: TokenStream) -> TokenStream {
    let mut cases = TokenStream::new();

    for (ty, t) in RUST_TYPES.iter().zip(TYPES.iter()) {
        let t = Ident::new(t, proc_macro2::Span::call_site());
        let ty = Ident::new(ty, proc_macro2::Span::call_site());

        cases.extend(quote! {
            Type::#t => {
                let slice = bytemuck::try_cast_slice::<u8, Packed<#ty>>(slice).map_err(bytemuck_err)?;
                type TY = #ty;
                #input
            }
        });
    }

    quote! {
        {
            let slice0 = self.buffer.get_slice()?;

            let slice = self.apply_offsets(&slice0)?;


            match self.ty {
                #cases
            }
        }
    }
}

pub fn typed_array_run_mut(input: TokenStream) -> TokenStream {
    let mut cases = TokenStream::new();

    for (ty, t) in RUST_TYPES.iter().zip(TYPES.iter()) {
        let t = Ident::new(t, proc_macro2::Span::call_site());
        let ty = Ident::new(ty, proc_macro2::Span::call_site());

        cases.extend(quote! {
            Type::#t => {
                let slice = bytemuck::try_cast_slice_mut::<u8, Packed<#ty>>(slice).map_err(bytemuck_err)?;
                type TY = #ty;
                #input
            }
        });
    }

    quote! {
        {
            let mut slice0 = self.buffer.get_slice_mut()?;

            let mut slice = self.apply_offsets_mut(&mut slice0)?;


            match self.ty {
                #cases
            }
        }


    }
}
