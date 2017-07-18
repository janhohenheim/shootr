extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;
extern crate specs;

use proc_macro::TokenStream;
use specs::{Component, VecStorage};



#[proc_macro_derive(Component)]
pub fn component(input: TokenStream) -> TokenStream {
    let s = input.to_string();
    let ast = syn::parse_derive_input(&s).unwrap();

    let gen = impl_component(&ast);
    gen.parse().unwrap()
}

fn impl_component(ast: &syn::MacroInput) -> quote::Tokens {
    let name = &ast.ident;
    quote! {
        impl Component for #name {
            type Storage = VecStorage<Self>;
        }

    }
}
