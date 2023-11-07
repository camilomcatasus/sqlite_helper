extern crate proc_macro;
use quote::quote;
use proc_macro::TokenStream;
use syn::{ parse_macro_input, DeriveInput, Data, Fields, FieldsNamed, Ident};

#[proc_macro_derive(Queryable)]
pub fn print_tokens(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = parse_macro_input!(input as DeriveInput);
    let new_functions: proc_macro2::TokenStream;
    // Check if the input is a struct
    if let Data::Struct(data_struct) = ast.data {
        let struct_name = ast.ident;

        match data_struct.fields {
            Fields::Named(fields_named) => {
                let request = request_struct(&fields_named, &struct_name);
                let get_fn_tokens = body_get(&fields_named, &struct_name);
                let add_fn_tokens = body_add(&fields_named, &struct_name);
                let update_fn_tokens = body_update(&fields_named, &struct_name);
                new_functions = quote! {
                    #request

                    impl #struct_name {
                        #get_fn_tokens
                        #add_fn_tokens
                        #update_fn_tokens
                    }
                }
            }
            _ => panic!("Only structs with named fields are supported"),
        }
    } else {
        panic!("Only structs are supported");
    }
    return TokenStream::from(new_functions);
}

#[proc_macro_derive(Bindable)]
pub fn print_binding_tokens(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = parse_macro_input!(input as DeriveInput);
    let new_functions: proc_macro2::TokenStream;
    if let Data::Enum(enum_struct) = ast.data {
        let enum_name = ast.ident;

        new_functions = quote! {
            impl rusqlite::types::ToSql for #enum_name {
                fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
                    Ok(self.to_string().into())
                }
            }

            impl rusqlite::types::FromSql for #enum_name {
                fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
                    value.as_str()?.parse()
                        .map_err(|e| rusqlite::types::FromSqlError::Other(Box::new(e)))
                }
            }
        }
    } else {
        panic!("Only enums are supported");
    }
    return TokenStream::from(new_functions);
}

fn request_struct(fields_named: &FieldsNamed, struct_name: &Ident) -> proc_macro2::TokenStream {
    let request_struct: &Ident = &Ident::new(&format!("{}Request", struct_name), proc_macro2::Span::call_site());
    let idents: Vec<_> = fields_named.named.iter().map(|f| &f.ident).collect();
    let types: Vec<_> = fields_named.named.iter().map(|f| &f.ty).collect();
    quote! {
        #[derive(Default, Clone)]
        pub struct #request_struct {
            #(pub #idents : Option<#types>),*
        }

    }
}

fn body_get(fields_named: &FieldsNamed, struct_name: &Ident) -> proc_macro2::TokenStream {
    let request_struct: &Ident = &Ident::new(&format!("{}Request", struct_name), proc_macro2::Span::call_site());
    let struct_name_string = String::from(struct_name.to_string());
    let idents: Vec<_> = fields_named.named.iter().map(|f| &f.ident).collect();
    let index: Vec<_> = fields_named.named.iter().enumerate().map(|f| f.0).collect();
    let types: Vec<_> = fields_named.named.iter().map(|f| &f.ty).collect();
    let conditions: Vec<String> = idents.iter().map(|ident| format!("AND {} = ", ident.as_ref().unwrap())).collect();

    quote! {
        pub fn get(conn: &rusqlite::Connection, request: #request_struct) -> anyhow::Result<Self> {
            let mut count = 1;
            let mut query_string: String = format!("SELECT * FROM {} WHERE TRUE = TRUE", #struct_name_string);
            let mut to_sql_objects: Vec<&dyn rusqlite::ToSql> = Vec::new();
            #(
                let mut #idents: #types;
                if let Some(i) = request.#idents {
                    query_string = format!("{}\n{}?{}", query_string, #conditions, count);
                    #idents = i.clone();
                    to_sql_objects.push(&#idents);

                    count += 1;
                }
            )*

            let obj: #struct_name = conn.query_row((&query_string), rusqlite::params_from_iter(to_sql_objects), |row| {
                Ok(#struct_name {
                    #(#idents : row.get(#index)?,)*
                })
            })?;

            return Ok(obj);
        }
        
        pub fn get_many(conn: &rusqlite::Connection, request: #request_struct) -> anyhow::Result<Vec<Self>> {
            let mut count = 1;
            let mut query_string: String = format!("SELECT * FROM {} WHERE TRUE = TRUE", #struct_name_string);
            let mut to_sql_objects: Vec<&dyn rusqlite::ToSql> = Vec::new();
            #(
                let mut #idents: #types;
                if let Some(i) = request.#idents {
                    query_string = format!("{}\n{}?{}", query_string, #conditions, count);
                    #idents = i.clone();
                    to_sql_objects.push(&#idents);

                    count += 1;
                }
            )*
            let mut stmt = conn.prepare(&query_string)?;
            let obj_iter = stmt.query_map(rusqlite::params_from_iter(to_sql_objects), |row| {
                Ok(#struct_name {
                    #(#idents : row.get(#index)?,)*
                })
            })?;

            let obj_vector = obj_iter.map(|x| x.unwrap()).collect();

            return Ok(obj_vector);

        }
    }
}

fn body_add(fields_named: &FieldsNamed, struct_name: &Ident) -> proc_macro2::TokenStream {
    let struct_name_string = String::from(struct_name.to_string());
    let idents: Vec<_> = fields_named.named.iter().map(|f| &f.ident).collect();
    
    let vals: Vec<String> = idents.iter()
        .enumerate()
        .map(|(i, _)| format!("?{}", i + 1)).collect();
    let joined_vals = vals.join(", ");

    let var_strings: Vec<_> = idents.iter().filter_map(|&opt| opt.as_ref()).map(|ident| ident.to_string()).collect();
    let joined_vars: String = var_strings.join(", ");
    let query_string: String = format!("INSERT INTO {} ({}) VALUES ({});", struct_name_string, joined_vars, joined_vals);
    
     
    quote! {
        pub fn add(&self, conn: &rusqlite::Connection) -> anyhow::Result<usize> {
            let query_string: &str = #query_string;
            let stmt: usize = conn.execute(query_string, rusqlite::params! [#( self.#idents),*])?;  
            return Ok(stmt);
        }
    }
}

fn body_update(fields_named: &FieldsNamed, struct_name: &Ident) -> proc_macro2::TokenStream {
    let struct_name_string = String::from(struct_name.to_string());
    let idents: Vec<_> = fields_named.named.iter().map(|f| &f.ident).collect();
    let first_ident = idents.get(0).unwrap();
    
    let var_strings: Vec<_> = idents.iter().filter_map(|&opt| opt.as_ref()).map(|ident| ident.to_string()).collect();
    let mut up_strings: Vec<String> = Vec::new();

    for index in 1..idents.len() {
        up_strings.push(format!("{} = ?{}", var_strings.get(index).unwrap(), index));
    }

    let joined_up_strings: String = up_strings.join(",\n");
    let query_string: String = format!("UPDATE {} SET {} WHERE {} = {{}} ;", struct_name_string, joined_up_strings, var_strings.get(0).unwrap());
    let skipped_idents: Vec<_> = idents.iter().skip(1).map(|f| *f).collect();
    
     
    quote! {
        pub fn update(&self, conn: &rusqlite::Connection) -> anyhow::Result<usize> {
            let query_string: String = format!(#query_string, self.#first_ident);
            let stmt: usize = conn.execute(&query_string, rusqlite::params![#( self.#skipped_idents),*])?;  
            return Ok(stmt);
        }
    }
}

#[proc_macro_derive(LibSqlQueryable)]
pub fn libsql_macro(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = parse_macro_input!(input as DeriveInput);
    let new_functions: proc_macro2::TokenStream;
    // Check if the input is a struct
    if let Data::Struct(data_struct) = ast.data {
        let struct_name = ast.ident;

        match data_struct.fields {
            Fields::Named(fields_named) => {
                let request = request_struct(&fields_named, &struct_name);
                let get_fn_tokens = libsql_body_get(&fields_named, &struct_name);
                let add_fn_tokens = libsql_body_add(&fields_named, &struct_name);
                let update_fn_tokens = libsql_body_update(&fields_named, &struct_name);
                new_functions = quote! {
                    #request

                    impl #struct_name {
                        #get_fn_tokens
                        #add_fn_tokens
                        #update_fn_tokens
                    }
                }
            }
            _ => panic!("Only structs with named fields are supported"),
        }
    } else {
        panic!("Only structs are supported");
    }

    println!("{}", new_functions);
    return TokenStream::from(new_functions);
}

fn libsql_body_get(fields_named: &FieldsNamed, struct_name: &Ident) -> proc_macro2::TokenStream {
    let request_struct: &Ident = &Ident::new(&format!("{}Request", struct_name), proc_macro2::Span::call_site());
    let struct_name_string = String::from(struct_name.to_string());
    let idents: Vec<_> = fields_named.named.iter().map(|f| &f.ident).collect();
    let types: Vec<_> = fields_named.named.iter().map(|f| &f.ty).collect();
    let conditions: Vec<String> = idents.iter().map(|ident| format!("AND {} = ", ident.as_ref().unwrap())).collect();

    quote! {
        pub async fn get(client: &libsql_client::Client, request: #request_struct) -> anyhow::Result<Self> {
            let mut query_string: String = format!("SELECT * FROM {} WHERE TRUE = TRUE", #struct_name_string);
            let mut to_sql_objects: Vec<libsql_client::Value> = Vec::new();
            #(
                let mut #idents: #types;
                if let Some(i) = request.#idents {
                    query_string = format!("{}\n{}?", query_string, #conditions);
                    #idents = i.clone();
                    to_sql_objects.push(#idents.into());
                }
            )*

            let obj: #struct_name = client.execute(libsql_client::Statement::with_args(&query_string, &to_sql_objects)).await?
                .rows
                .iter()
                .next()
                .map(libsql_client::de::from_row::<#struct_name>)
                .transpose()?
                .context("No rows returned")?;
            return Ok(obj);
        }

        pub async fn get_many(client: &libsql_client::Client, request: #request_struct) -> anyhow::Result<Vec<Self>> {
            let mut query_string: String = format!("SELECT * FROM {} WHERE TRUE = TRUE", #struct_name_string);
            let mut to_sql_objects: Vec<libsql_client::Value> = Vec::new();
            #(
                let mut #idents: #types;
                if let Some(i) = request.#idents {
                    query_string = format!("{}\n{}?", query_string, #conditions);
                    #idents = i.clone();
                    to_sql_objects.push(#idents.into());
                }
            )*

            let obj_vector = client.execute(libsql_client::Statement::with_args(&query_string, &to_sql_objects)).await?
                .rows
                .iter()
                .map(libsql_client::de::from_row)
                .collect::<Result<Vec<#struct_name>, _>>()?;

            return Ok(obj_vector);
        }
    }
}

fn libsql_body_add(fields_named: &FieldsNamed, struct_name: &Ident) -> proc_macro2::TokenStream {
    let struct_name_string = String::from(struct_name.to_string());
    let idents: Vec<_> = fields_named.named.iter().map(|f| &f.ident).collect();
    
    let vals: Vec<String> = idents.iter()
        .enumerate()
        .map(|_| "?".to_string()).collect();
    let joined_vals = vals.join(", ");

    let var_strings: Vec<_> = idents.iter().filter_map(|&opt| opt.as_ref()).map(|ident| ident.to_string()).collect();
    let joined_vars: String = var_strings.join(", ");
    let query_string: String = format!("INSERT INTO {} ({}) VALUES ({});", struct_name_string, joined_vars, joined_vals);
    
     
    quote! {
        pub async fn add(&self, client: &libsql_client::Client) -> anyhow::Result<usize> {
            let query_string: &str = #query_string;
            let mut params: Vec<libsql_client::Value> = Vec::new();
            #(params.push(self.#idents.clone().into());)*
            let stmt = client.execute(libsql_client::Statement::with_args(query_string,  &params)).await?;
            return Ok(stmt.rows_affected as usize);
        }
    }
}

fn libsql_body_update(fields_named: &FieldsNamed, struct_name: &Ident) -> proc_macro2::TokenStream {
    let struct_name_string = String::from(struct_name.to_string());
    let idents: Vec<_> = fields_named.named.iter().map(|f| &f.ident).collect();
    let first_ident = idents.get(0).unwrap();
    
    let var_strings: Vec<_> = idents.iter().filter_map(|&opt| opt.as_ref()).map(|ident| ident.to_string()).collect();
    let mut up_strings: Vec<String> = Vec::new();

    for index in 1..idents.len() {
        up_strings.push(format!("{} = ?", var_strings.get(index).unwrap()));
    }

    let joined_up_strings: String = up_strings.join(",\n");
    let query_string: String = format!("UPDATE {} SET {} WHERE {} = {{}} ;", struct_name_string, joined_up_strings, var_strings.get(0).unwrap());
    let skipped_idents: Vec<_> = idents.iter().skip(1).map(|f| *f).collect();
    
     
    quote! {
        pub async fn update(&self, client: &libsql_client::Client) -> anyhow::Result<usize> {
            let query_string: String = format!(#query_string, self.#first_ident);
            let mut params: Vec<libsql_client::Value> = Vec::new();
            #(params.push(self.#skipped_idents.clone().into());)*
            let stmt = client.execute(libsql_client::Statement::with_args(&query_string, &params)).await?;  
            return Ok(stmt.rows_affected as usize);
        }
    }
}
