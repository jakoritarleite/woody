use proc_macro::TokenStream;
use quote::quote;
use syn;
use syn_path::path;

#[proc_macro_derive(Component)]
pub fn derive_component(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();

    let struct_name = &ast.ident;

    let path = path!(::woody::ecs);

    TokenStream::from(quote! {
        impl #path::component::Component for #struct_name {}
    })
}
