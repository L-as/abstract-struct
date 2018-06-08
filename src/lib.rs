#![feature(proc_macro)]
#![feature(nll)]
#![feature(label_break_value)]

extern crate proc_macro;
extern crate matches;
extern crate itertools;

use matches::*;

use proc_macro::{
	TokenStream,
	TokenTree,
	Delimiter,
	Ident,
	Punct,
	Spacing,
};

use itertools::Itertools;

use std::{
	mem,
};

#[proc_macro]
pub fn abstract_struct(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let input: TokenStream = input.into();
	let mut iter = input.into_iter();

	let mut struct_def: Vec<_> = iter.take_while_ref(|t| match t {
		TokenTree::Group(g) if matches!(g.delimiter(), Delimiter::Brace) => false,
		_ => true,
	}).collect();
	struct_def.push(iter.next().expect("Unexpected end of input"));

	let struct_token_pos = struct_def.iter().position(|t| matches!(t, TokenTree::Ident(i) if i.to_string() == "struct")).unwrap();
	let struct_name = match &mut struct_def[struct_token_pos+1] {
		TokenTree::Ident(i) => i,
		_ => panic!(),
	};
	let trait_name = mem::replace(struct_name, Ident::new(&(struct_name.to_string() + "Concrete"), struct_name.span()));

	let where_token_pos = struct_def[struct_token_pos+2..].iter()
		.rposition(|t| matches!(t, TokenTree::Ident(i) if i.to_string() == "where"))
		.map(|pos| pos + struct_token_pos + 2);

	let generics = match &struct_def[struct_token_pos+2] {
		TokenTree::Punct(p) if p.as_char() == '<' => {
			Some(&struct_def[struct_token_pos+3..where_token_pos.unwrap_or(struct_def.len()-1)-1])
		},
		_ => None,
	};

	let mut trait_def: Vec<_> = iter.take_while_ref(|t| match t {
		TokenTree::Ident(i) => i.to_string() != "trait",
		_ => true,
	}).collect();
	let trait_token = iter.next().unwrap();
	let trait_token_span = trait_token.span();
	trait_def.push(trait_token);
	trait_def.push(TokenTree::Ident(trait_name.clone()));

	let body = match iter.next().unwrap() {
		TokenTree::Group(g) => g,
		_ => panic!()
	};

	trait_def.push(TokenTree::Group(body.clone()));

	let mut trait_impl = vec![TokenTree::Ident(Ident::new("impl", trait_token_span))];
	if let Some(generics) = generics {
		trait_impl.push(TokenTree::Punct(Punct::new('<', Spacing::Alone)));
		trait_impl.extend(generics.iter().cloned());
		trait_impl.push(TokenTree::Punct(Punct::new('>', Spacing::Alone)));
	}
	trait_impl.push(TokenTree::Ident(trait_name));
	trait_impl.push(TokenTree::Ident(Ident::new("for", trait_token_span)));
	trait_impl.push(struct_def[struct_token_pos+1].clone());
	if let Some(generics) = generics {
		// 1: inside constraint
		// 1+n: inside constraint inside N wrappings of <>
		let mut nesting = 0usize;
		let generic_arguments: Vec<_> = generics.iter().cloned().filter(|t| match t {
			TokenTree::Punct(p) if p.as_char() == '>' => {assert!(nesting > 0); nesting -= 1; false},
			TokenTree::Punct(p) if p.as_char() == '<' => {assert!(nesting > 0); nesting += 1; false},
			TokenTree::Punct(p) if p.as_char() == ':' => {assert!(nesting == 0); nesting = 1; false},
			TokenTree::Punct(p) if p.as_char() == ',' && nesting == 1 => {nesting = 0; true},
			_ => nesting == 0,
		}).collect();
		trait_impl.push(TokenTree::Punct(Punct::new('<', Spacing::Alone)));
		trait_impl.extend(generic_arguments);
		trait_impl.push(TokenTree::Punct(Punct::new('>', Spacing::Alone)));
	}
	if let Some(where_token_pos) = where_token_pos {
		let where_clause = &struct_def[where_token_pos+1..struct_def.len()-1];
		trait_impl.push(struct_def[where_token_pos].clone());
		trait_impl.extend(where_clause.iter().cloned());
	}
	trait_impl.push(TokenTree::Group(body));


	let stream: TokenStream = struct_def.into_iter()
		.chain(trait_def)
		.chain(trait_impl)
		.collect();
	println!("result: {}", stream);
	stream
}
