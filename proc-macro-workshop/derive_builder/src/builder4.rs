use crate::builder3::{BuilderField, FieldType};
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

fn make_storage(fields: &[BuilderField]) -> TokenStream2 {
  fields
    .iter()
    .map(|field| {
      let ident = &field.ident;
      let storage = match &field.ty {
        FieldType::Plain(ty) | FieldType::Optional(ty) => quote!(Option<#ty>),
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

/// 负责根据输入的结构体字段信息生成一个构建函数，该函数将检查所有必需的字段是否已设置值，
/// 并将所有字段（包括可选字段）从构建器实例中提取出来，最终构建并返回一个结构体实例
///
/// # Parameters
/// - `vis`: 指定生成函数的可见性，例如`pub`或私有
/// - `input_ident`: 结构体的标识符，即结构体的名称
/// - `fields`: 构建器字段的切片，包含所有需要处理的字段信息
///
/// # Returns
/// - 返回一个`TokenStream2`，包含了构建函数的源代码，可以直接用于生成Rust代码
fn make_build_fn(vis: &Visibility, input_ident: &Ident, fields: &[BuilderField]) -> TokenStream2 {
  // 遍历所有字段，为每个非可选字段生成检查代码，确保它们在构建之前已被设置
  let required_field_checks = fields.iter().filter_map(|field| {
    let ident = &field.ident;
    match &field.ty {
      FieldType::Plain(_) => Some(quote! {
        let #ident = self.#ident.ok_or_else(|| {
          Box::<dyn core::error::Error>::from(format!("value is not set: {}", stringify!(#ident)))
        })?;
      }),
      FieldType::Optional(_) => None,
    }
  });

  // 遍历所有字段，生成字段赋值代码，将构建器中的字段值转移到新构建的结构体实例中
  let field_assignment = fields.iter().map(|field| {
    let ident = &field.ident;
    let expr = match &field.ty {
      FieldType::Plain(_) => quote!(#ident),
      FieldType::Optional(_) => quote!(self.#ident),
    };
    quote! {
      #ident: #expr,
    }
  });

  // 生成并返回完整的构建函数代码
  quote! {
    #vis fn build(self) -> Result<#input_ident, Box<dyn core::error::Error>> {
      #(#required_field_checks)*

      Ok(#input_ident {
        #(#field_assignment)*
      })
    }
  }
}

