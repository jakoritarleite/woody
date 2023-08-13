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

        impl #path::component::Bundle for #struct_name {
            fn components_ids() -> Vec<std::any::TypeId> {
                vec![ std::any::TypeId::of::<#struct_name>() ]
            }

            fn components(
                self,
                storage: &mut #path::archetypes::ArchetypeStorage,
                row_indexes: &mut impl FnMut(usize)
            ) {
                let row_index = storage.init_component(self);

                row_indexes(row_index);
            }
        }
    })
}
