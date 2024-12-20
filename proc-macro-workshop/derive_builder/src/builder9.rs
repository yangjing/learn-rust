use crate::builder7::{BuilderField, FieldType};
use crate::helper::extract_struct_fields;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{DeriveInput, Visibility};

pub fn expand(input: DeriveInput) -> syn::Result<TokenStream2> {
  let vis = &input.vis;
  let input_ident = &input.ident;
  let builder_ident = Ident::new(&format!("{}Builder", input_ident), Span::call_site());

  let fields = extract_struct_fields(input.data)?;

  let builder_fields: Vec<BuilderField> =
    fields.named.iter().map(BuilderField::try_from).collect::<syn::Result<_>>()?;

  let storage = make_storage(&builder_fields);
  let initializer = make_initializer(&builder_fields);
  let setters = make_setters(vis, &builder_fields);
  let build_fn = make_build_fn(vis, &input_ident, &builder_fields);

  Ok(quote! {
    #vis struct #builder_ident {
      #storage
    }

    impl #input_ident {
      #vis fn builder() -> #builder_ident {
        #builder_ident {
          #initializer
        }
      }
    }

    impl #builder_ident {
      #setters
      #build_fn
    }
  })
}

fn make_storage(fields: &[BuilderField]) -> TokenStream2 {
  fields
    .iter()
    .map(|field| {
      let ident = &field.ident;
      let storage = match &field.ty {
        FieldType::Plain(ty) | FieldType::Optional(ty) => quote!(core::option::Option<#ty>),
        FieldType::Repeated(_, ty) => quote!(#ty),
      };
      quote! {
        #ident: #storage,
      }
    })
    .collect()
}

fn make_initializer(fields: &[BuilderField]) -> TokenStream2 {
  fields
    .iter()
    .map(|field| {
      let ident = &field.ident;
      let init = match &field.ty {
        FieldType::Plain(_) | FieldType::Optional(_) => quote!(core::option::Option::None),
        FieldType::Repeated(_, ty) => quote!(<#ty>::new()),
      };
      quote! {
        #ident: #init,
      }
    })
    .collect()
}

fn make_setters(vis: &Visibility, fields: &[BuilderField]) -> TokenStream2 {
  fields
    .iter()
    .map(|field| {
      let ident = &field.ident;
      let plain_store = quote!(self.#ident = core::option::Option::Some(#ident));
      let repeated_store = quote!(self.#ident.push(#ident));
      let inner = |ty| quote!(<#ty as std::iter::IntoIterator>::Item);
      let (fn_name, arg, store) = match &field.ty {
        FieldType::Plain(ty) => (ident, quote!(#ty), plain_store),
        FieldType::Optional(ty) => (ident, quote!(#ty), plain_store),
        FieldType::Repeated(each, ty) => (each, inner(ty), repeated_store)
      };
      quote! {
        #vis fn #fn_name(&mut self, #ident: #arg) -> &mut Self {
          #store;
          self
        }
      }
    })
    .collect()
}

fn make_build_fn(vis: &Visibility, input_ident: &Ident, fields: &[BuilderField]) -> TokenStream2 {
  let required_field_checks = fields.iter().filter_map(|field| {
    let ident = &field.ident;
    let error = format!("value is not set: {}", ident);
    match &field.ty {
      FieldType::Plain(_) => Some(quote! {
        let #ident = self.#ident.take().ok_or_else(|| {
          std::boxed::Box::<dyn core::error::Error>::from(#error)
        })?;
      }),
      FieldType::Optional(_) | FieldType::Repeated(..) => None,
    }
  });

  let field_assignment = fields.iter().map(|field| {
    let ident = &field.ident;
    let expr = match &field.ty {
      FieldType::Plain(_) => quote!(#ident),
      FieldType::Optional(_) => quote!(self.#ident.take()),
      FieldType::Repeated(_, ty) => quote!(core::mem::replace(&mut self.#ident, <#ty>::new()))
    };
    quote! {
      #ident: #expr,
    }
  });

  quote! {
    #vis fn build(&mut self) -> core::result::Result<#input_ident, std::boxed::Box<dyn core::error::Error>> {
      #(#required_field_checks)*

      Ok(#input_ident {
        #(#field_assignment)*
      })
    }
  }
}
