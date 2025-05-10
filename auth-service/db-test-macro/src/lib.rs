use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, Stmt};

#[proc_macro_attribute]
pub fn db_test(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the input token stream as a function
    let mut input_fn = parse_macro_input!(item as ItemFn);

    // Ensure the function is async
    if input_fn.sig.asyncness.is_none() {
        panic!("The #[db_test] attribute can only be applied to async functions");
    }

    // Create a statement to add at the end of the function body
    let cleanup_stmt: Stmt = syn::parse_quote! {
        app.clean_up_db().await;
    };

    // Add the clean_up_db call to the end of the function body
    input_fn.block.stmts.push(cleanup_stmt);
    input_fn.attrs.push(syn::parse_quote!(#[tokio::test]));

    // Generate the output token stream
    quote! {
        #input_fn
    }
    .into()
}
