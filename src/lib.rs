#![feature(proc_macro)]
#![feature(nll)]

extern crate proc_macro;
extern crate matches;
extern crate itertools;
extern crate smallvec;

use matches::*;
use proc_macro::{
	TokenStream,
	TokenTree,
	Delimiter,
	Ident,
	Punct,
	Group,
	Spacing,
};
use itertools::Itertools;
use std::mem;
use smallvec::SmallVec;

macro_rules! err {
	($token:expr, $($msg:tt)*) => {{
		$token.span().error(format!($($msg)*)).emit();
		panic!("Errors encountered");
	}}
}

const EOIE: &'static str = "Unexpected end of input";

struct Struct {
	token_pos: usize,
	def: Vec<TokenTree>,
}

impl Struct {
	fn new(iter: &mut (impl Iterator<Item = TokenTree> + Clone)) -> Self {
		let mut def: Vec<_> = iter.take_while_ref(|t| match t {
			TokenTree::Group(g) if matches!(g.delimiter(), Delimiter::Brace) => false,
			_ => true,
		}).collect();
		def.push(iter.next().expect(EOIE));

		let token_pos = def.iter()
			.position(|t| matches!(t, TokenTree::Ident(i) if i.to_string() == "struct")).expect("`struct` keyword not found");

		Self {
			token_pos,
			def,
		}
	}
}

struct TypeConstraint {
	is_lifetime: bool,
	name: Ident,
	def: Vec<TokenTree>
}

struct TypeConstraints(Vec<TypeConstraint>);

fn iter_type(tokens: &mut impl Iterator<Item = TokenTree>, mut callback: impl FnMut(TokenTree), ender: char) {
	let mut nesting = 0usize;
	for t in tokens {
		match &t {
			TokenTree::Punct(p) if p.as_char() == '>' => nesting = nesting.checked_sub(1).unwrap_or_else(|| err!(t, "Too many >")),
			TokenTree::Punct(p) if p.as_char() == '<' => nesting += 1,
			TokenTree::Punct(p) if p.as_char() == ender && nesting == 0 => break,
			_ => {},
		}
		callback(t);
	}
}

impl TypeConstraints {
	fn new(tokens: impl Iterator<Item = TokenTree>) -> Self {
		let v = tokens.batching(|tokens| {
			loop {
				let (is_lifetime, name) = match tokens.next()? {
					TokenTree::Ident(i) => (false, i),
					TokenTree::Punct(ref p) if p.as_char() == '\'' => {
						(true, unwrap_match!(tokens.next().expect(EOIE), TokenTree::Ident(i) => i))
					},
					TokenTree::Punct(ref p) if p.as_char() == '<' => {
						let mut nesting = 1usize;
						while nesting != 0 {
							let t = tokens.next().expect(EOIE);
							match &t {
								TokenTree::Punct(p) if p.as_char() == '>' => nesting = nesting.checked_sub(1).unwrap_or_else(|| err!(t, "Too many >")),
								TokenTree::Punct(p) if p.as_char() == '<' => nesting += 1,
								_ => {},
							}
						}
						iter_type(tokens, |_| {}, ',');
						continue
					},
					_ => unimplemented!()
				};

				let mut def = Vec::new();
				match &tokens.next() {
					Some(TokenTree::Punct(p)) if p.as_char() == ':' => {},
					Some(TokenTree::Punct(p)) if p.as_char() == ',' => return Some(TypeConstraint {is_lifetime, name, def}),
					None => return Some(TypeConstraint {is_lifetime, name, def}),
					_ => unimplemented!()
				}

				match tokens.next().expect(EOIE) {
					TokenTree::Punct(ref p) if p.as_char() == ':' => {iter_type(tokens, |_| (), ','); continue},
					t => def.push(t),
				}

				iter_type(tokens, |t| def.push(t), ',');

				return Some(TypeConstraint {is_lifetime, name, def})
			}
		}).collect();

		TypeConstraints(v)
	}

	fn merge(&mut self, mut other: TypeConstraints) {
		for cons in self.0.iter_mut() {
			if let Some(partner) = other.0.iter_mut().find(|c| c.name.to_string() == cons.name.to_string()) {
				if cons.def.len() == 0 {
					mem::swap(&mut cons.def, &mut partner.def)
				} else {
					cons.def.push(TokenTree::Punct(Punct::new('+', Spacing::Alone)));
					cons.def.extend(mem::replace(&mut partner.def, Vec::new()));
				}
			}
		}
	}

	fn vars(&self) -> Vec<TokenTree> {
		let mut v = Vec::new();
		for cons in self.0.iter() {
			if cons.is_lifetime {
				v.push(TokenTree::Punct(Punct::new('\'', Spacing::Joint)));
			}
			v.push(TokenTree::Ident(cons.name.clone()));
			v.push(TokenTree::Punct(Punct::new(',', Spacing::Alone)));
		}
		v
	}
}

fn strip_trait_body(body: Group, type_constraints: &TypeConstraints) -> Group {
	macro_rules! some {
		($expr:expr) => {{
			let mut v = SmallVec::new();
			v.push($expr);
			v
		}}
	}

	macro_rules! none {
		() => {{SmallVec::new()}}
	}

	enum State {
		None,
		Assoc,
		AssocVal,
		Fn,
	}
	let mut state = State::None;

	let iter = body.stream().into_iter();
	let mut new_body = Group::new(Delimiter::Brace, iter.flat_map(|t| {
		let a: SmallVec<[TokenTree; 1]> = match state {
			State::None => match &t {
				TokenTree::Ident(i) if i.to_string() == "type" || i.to_string() == "const" => {
					state = State::Assoc;
					some!(t)
				},
				TokenTree::Ident(i) if i.to_string() == "fn" => {
					state = State::Fn;
					some!(t)
				},
				_ => some!(t)
			},
			State::Assoc => match &t {
				TokenTree::Group(g) if matches!(g.delimiter(), Delimiter::Brace) => {
					let ty = unwrap_match!(g.stream().into_iter().next().unwrap_or_else(|| err!(t, "No type specified")), TokenTree::Ident(i) => i);
					let s = ty.to_string();
					type_constraints.0.iter().find(|c| c.name.to_string() == s)
						.unwrap_or_else(|| err!(ty, "Type not a generic parameter"))
						.def
						.clone()
						.into()
				},
				TokenTree::Punct(p) if p.as_char() == '=' => {
					state = State::AssocVal;
					none!()
				},
				_ => some!(t)
			},
			State::AssocVal => match &t {
				TokenTree::Punct(p) if p.as_char() == ';' => {
					state = State::None;
					some!(t)
				},
				_ => none!()
			},
			State::Fn => match &t {
				TokenTree::Group(g) if matches!(g.delimiter(), Delimiter::Brace) => {
					state = State::None;
					some!(TokenTree::Punct(Punct::new(';', Spacing::Alone)))
				},
				_ => some!(t)
			},
		};
		a
	}).collect());
	new_body.set_span(body.span());
	new_body
}

fn strip_trait_impl_body(body: Group) -> Group {
	enum State {
		None,
		Assoc,
		AssocType,
	}
	let mut state = State::None;

	let iter = body.stream().into_iter();
	let mut new_body = Group::new(Delimiter::Brace, iter.filter_map(|t| match state {
		State::None => match &t {
			TokenTree::Ident(i) if i.to_string() == "type" => {
				state = State::Assoc;
				Some(t)
			},
			_ => Some(t)
		},
		State::Assoc => {
			state = State::AssocType;
			Some(t)
		},
		State::AssocType => match &t {
			TokenTree::Punct(p) if p.as_char() == '=' => {
				state = State::None;
				Some(t)
			},
			_ => None
		},
	}).collect());
	new_body.set_span(body.span());
	new_body
}

#[proc_macro]
pub fn abstract_struct(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let input: TokenStream = input.into();
	let mut iter = input.into_iter();

	let mut strct = Struct::new(&mut iter);
	let struct_name = unwrap_match!(&mut strct.def[strct.token_pos+1], TokenTree::Ident(i) => i);
	let trait_name = mem::replace(struct_name, Ident::new(&(struct_name.to_string() + "Concrete"), struct_name.span()));

	let where_token_pos = strct.def[strct.token_pos+2..].iter()
		.rposition(|t| matches!(t, TokenTree::Ident(i) if i.to_string() == "where"))
		.map(|pos| pos + strct.token_pos + 2);

	let generics = match &strct.def[strct.token_pos+2] {
		TokenTree::Punct(p) if p.as_char() == '<' => {
			Some(&strct.def[strct.token_pos+3..where_token_pos.unwrap_or(strct.def.len()-1)-1])
		},
		_ => None,
	};

	let mut type_constraints = TypeConstraints::new(generics.unwrap_or(&[]).iter().cloned());
	if let Some(where_token_pos) = where_token_pos {
		type_constraints.merge(
			TypeConstraints::new(strct.def[where_token_pos+1..strct.def.len()-1].iter().cloned())
		);
	}

	let mut trait_def: Vec<_> = iter.take_while_ref(|t| match t {
		TokenTree::Ident(i) => i.to_string() != "trait",
		_ => true,
	}).collect();
	let is_unsafe = match trait_def.last() {
		Some(TokenTree::Ident(last)) if last.to_string() == "unsafe" => true,
		_ => false,
	};
	let trait_token = iter.next().expect(EOIE);
	let trait_token_span = trait_token.span();
	trait_def.push(trait_token);
	trait_def.push(TokenTree::Ident(trait_name.clone()));

	let trait_body = unwrap_match!(iter.next().expect(EOIE), TokenTree::Group(g) => g);
	let trait_impl_body = trait_body.clone();

	let trait_body = strip_trait_body(trait_body, &type_constraints);
	let trait_impl_body = strip_trait_impl_body(trait_impl_body);

	trait_def.push(TokenTree::Group(trait_body));

	let mut trait_impl = Vec::new();
	if is_unsafe {
		trait_impl.push(trait_def[trait_def.len()-4].clone());
	}
	trait_impl.push(TokenTree::Ident(Ident::new("impl", trait_token_span)));
	if let Some(generics) = generics {
		trait_impl.push(TokenTree::Punct(Punct::new('<', Spacing::Alone)));
		trait_impl.extend(generics.iter().cloned());
		trait_impl.push(TokenTree::Punct(Punct::new('>', Spacing::Alone)));
	}
	trait_impl.push(TokenTree::Ident(trait_name));
	trait_impl.push(TokenTree::Ident(Ident::new("for", trait_token_span)));
	trait_impl.push(strct.def[strct.token_pos+1].clone());
	trait_impl.push(TokenTree::Punct(Punct::new('<', Spacing::Alone)));
	trait_impl.extend(type_constraints.vars());
	trait_impl.push(TokenTree::Punct(Punct::new('>', Spacing::Alone)));
	if let Some(where_token_pos) = where_token_pos {
		let where_clause = &strct.def[where_token_pos+1..strct.def.len()-1];
		trait_impl.push(strct.def[where_token_pos].clone());
		trait_impl.extend(where_clause.iter().cloned());
	}
	trait_impl.push(TokenTree::Group(trait_impl_body));

	let stream: TokenStream = strct.def.into_iter()
		.chain(trait_def)
		.chain(trait_impl)
		.collect();
	println!("result: {}", stream);
	stream
}
