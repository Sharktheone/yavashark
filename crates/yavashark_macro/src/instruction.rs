use darling::ast::NestedMeta;
use darling::FromMeta;
use proc_macro2::{Ident, TokenStream};
use quote::quote;

#[derive(Debug, FromMeta)]
pub struct InstructionArgs {
    pub name: Ident,
    pub inputs: u8,
    pub output: bool,
    
}

const IN_TYPES: [&str; 5] = ["Variable", "Reg", "Acc", "Stack", "Const"];
const OUT_TYPES: [&str; 4] = ["Variable", "Reg", "Acc", "Stack"];


pub struct Inst {
    name: String,
    inputs: Vec<&'static str>,
}


pub fn instruction(input: TokenStream) -> syn::Result<TokenStream> {
    let attr_args = match NestedMeta::parse_meta_list(input.clone()) {
        
        Ok(v) => v,
        Err(e) => {
            return Err(e);
        }
    };

    let args = match InstructionArgs::from_list(&attr_args) {
        Ok(args) => args,
        Err(e) => return Err(e.into()),
    };
    
    
    let num_insts = (args.inputs as usize).pow(IN_TYPES.len() as u32) * if args.output { OUT_TYPES.len() } else { 1 };
    
    let mut insts = Vec::with_capacity(num_insts);
    
    for i in 0..num_insts {
        let mut inst = Inst {
            name: args.name.to_string(),
            inputs: Vec::with_capacity(args.inputs as usize),
        };
        
        let mut i = i;
        
        for _ in 0..args.inputs {
            inst.inputs.push(IN_TYPES[i % IN_TYPES.len()]);
            i /= IN_TYPES.len();
        }
        
        insts.push(inst);
    }
    
    let variants = insts.iter().map(|inst| {
        let name = Ident::new(&inst.name, proc_macro2::Span::call_site());
        let inputs = inst.inputs.iter().map(|input| {
            Ident::new(input, proc_macro2::Span::call_site())
        });
        
        quote! {
            #name {
                #(#inputs),*
            }
        }
    });
    
    
    
    let output = quote! {
        #(#variants),*
    };
    
    Ok(output)
}