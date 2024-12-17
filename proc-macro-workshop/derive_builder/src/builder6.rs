use crate::helper::{extract_inner_ty, extract_struct_fields};
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{DeriveInput, Field, Type, Visibility};

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
  let build_fn = make_build_fn(vis, input_ident, &builder_fields);

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

enum FieldType {
  Plain(Type),
  Optional(Type),
}

struct BuilderField {
  ident: Ident,
  ty: FieldType,
}

impl BuilderField {
  fn new(ident: Ident, ty: FieldType) -> Self {
    BuilderField { ident, ty }
  }

  fn try_from(field: &Field) -> syn::Result<Self> {
    let ident = field.ident.clone().unwrap();

    if let Type::Path(ty) = &field.ty {
      if let Some(segment) = ty.path.segments.last() {
        if segment.ident == "Option" {
          let inner_ty = extract_inner_ty(&segment.arguments)?;
          return Ok(BuilderField::new(ident, FieldType::Optional(inner_ty.clone())));
        }
      }
    }

    Ok(BuilderField::new(ident, FieldType::Plain(field.ty.clone())))
  }
}

fn make_storage(fields: &[BuilderField]) -> TokenStream2 {
  fields
    .iter()
    .map(|field| {
      let ident = &field.ident;
      let storage = match &field.ty {
        FieldType::Plain(ty) | FieldType::Optional(ty) => quote!(Option::<#ty>),
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
      quote! {
        #ident: None,
      }
    })
    .collect()
}

fn make_setters(vis: &Visibility, fields: &[BuilderField]) -> TokenStream2 {
  fields
    .iter()
    .map(|field| {
      let ident = &field.ident;
      let ident_type = match &field.ty {
        FieldType::Plain(ty) => ty,
        FieldType::Optional(ty) => ty,
      };
      quote! {
        #vis fn #ident(&mut self, #ident: #ident_type) -> &mut Self {
          self.#ident = Some(#ident);
          self
        }
      }
    })
    .collect()
}

fn make_build_fn(vis: &Visibility, input_ident: &Ident, fields: &[BuilderField]) -> TokenStream2 {
  let required_field_checks = fields.iter().filter_map(|field| {
    let ident = &field.ident;
    match &field.ty {
      FieldType::Plain(_) => Some(quote! {
        let #ident = self.#ident.take().ok_or_else(|| {
          Box::<dyn core::error::Error>::from(format!("value is not set: {}", stringify!(#ident)))
        })?;
      }),
      FieldType::Optional(_) => None,
    }
  });

  let field_assignment = fields.iter().map(|field| {
    let ident = &field.ident;
    let expr = match &field.ty {
      FieldType::Plain(_) => quote!(#ident),
      FieldType::Optional(_) => quote!(self.#ident.take()),
    };
    quote! {
      #ident: #expr,
    }
  });

  quote! {
    #vis fn build(&mut self) -> Result<#input_ident, Box<dyn core::error::Error>> {
      #(#required_field_checks)*

      Ok(#input_ident {
        #(#field_assignment)*
      })
    }
  }
}
