use proc_macro2::{Ident, TokenStream};
use quote::quote;

const TYPES: [&str; 11] = [
    "u8", "u16", "u32", "u64", "i8", "i16", "i32", "i64", "f16", "f32", "f64",
];

pub fn typed_array_run(input: TokenStream) -> TokenStream {
    let mut cases = TokenStream::new();

    for ty in TYPES.iter() {
        let t = Ident::new(&ty.to_uppercase(), proc_macro2::Span::call_site());
        let ty = Ident::new(ty, proc_macro2::Span::call_site());

        cases.extend(quote! {
            Type::#t => {
                let slice = bytemuck::cast_slice::<u8, #ty>(slice);
                type TY = #ty;
                #input
            }
        });
    }

    quote! {
        {
            let buf = self.get_buffer()?;
            let slice = buf.get_slice();

            let slice = self.apply_offsets(&slice)?;


            match self.ty {
                #cases
            }
        }
    }
}

pub fn typed_array_run_mut(input: TokenStream) -> TokenStream {
    let mut cases = TokenStream::new();

    for ty in TYPES.iter() {
        let t = Ident::new(&ty.to_uppercase(), proc_macro2::Span::call_site());
        let ty = Ident::new(ty, proc_macro2::Span::call_site());

        cases.extend(quote! {
            Type::#t => {
                let slice = bytemuck::cast_slice_mut::<u8, #ty>(slice);
                type TY = #ty;
                #input
            }
        });
    }

    quote! {
        {
            let buf = self.get_buffer()?;
            let mut slice = buf.get_slice_mut();

            let mut slice = self.apply_offsets_mut(&mut slice)?;


            match self.ty {
                #cases
            }
        }


    }
}
