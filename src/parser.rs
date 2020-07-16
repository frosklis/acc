extern crate num;

use super::model::Comment;
use super::model::Token;
use super::model::Transaction;
use super::model::UnbalancedPosting;

#[derive(Debug)]
struct Error {
	message: String,
}

pub fn parse_unbalanced_transactions(
	tokens: &[Token],
	transactions: &mut Vec<Transaction<UnbalancedPosting>>,
) -> Result<(), String> {
	match parse(tokens, transactions) {
		Err(err) => Err(format!("Parse Error : {}", err.message)),
		Ok(()) => Ok(()),
	}
}

fn parse(
	tokens: &[Token],
	transactions: &mut Vec<Transaction<UnbalancedPosting>>,
) -> Result<(), Error> {
	let mut parser = Parser {
		tokens,
		transactions,
		index: 0,
	};

	while parser.index < tokens.len() {
		parser.parse_transaction_header()?;
		parser.parse_transaction_comment()?;
		parser.parse_posting()?;
	}

	Ok(())
}

struct Parser<'a> {
	tokens: &'a [Token],
	transactions: &'a mut Vec<Transaction<UnbalancedPosting>>,
	index: usize,
}

impl<'a> Parser<'a> {
	fn parse_transaction_header(&mut self) -> Result<(), Error> {
		let date;
		let line;

		match self.tokens.get(self.index).unwrap() {
			Token::TransactionDate(file_line, value) => {
				self.index += 1;
				line = file_line;
				date = value.to_owned();
			}
			_ => return Ok(()),
		}

		let state;
		match self.tokens.get(self.index).unwrap() {
			Token::TransactionState(_, value) => {
				self.index += 1;
				state = value.clone();
			}
			_ => panic!("transaction state expected"),
		}

		let mut code: Option<String> = None;
		if let Token::TransactionCode(_, value) = self.tokens.get(self.index).unwrap() {
			self.index += 1;
			code = Some(value.to_owned());
		}

		let description;
		match self.tokens.get(self.index).unwrap() {
			Token::TransactionDescription(_, value) => {
				self.index += 1;
				description = value.to_owned();
			}
			_ => panic!("transaction description expected"),
		}

		self.transactions.push(Transaction {
			line: *line,
			date: date,
			state: state,
			code: code,
			description: description,
			comments: Vec::new(),
			postings: Vec::new(),
		});

		Ok(())
	}

	fn parse_transaction_comment(&mut self) -> Result<(), Error> {
		if let Some(token) = self.tokens.get(self.index) {
			if let Token::TransactionComment(line, value) = token {
				self
					.transactions
					.last_mut()
					.unwrap()
					.comments
					.push(Comment {
						line: *line,
						comment: value.to_owned(),
					});
				self.index += 1;
			}
		}
		Ok(())
	}

	fn parse_posting(&mut self) -> Result<(), Error> {
		let line;
		let account;

		match self.tokens.get(self.index) {
			None => return Ok(()),
			Some(token) => match token {
				Token::PostingAccount(file_line, value) => {
					self.index += 1;
					line = file_line;
					account = value;
				}
				_ => return Ok(()),
			},
		};

		match self.tokens.get(self.index) {
			None => {
				self
					.transactions
					.last_mut()
					.unwrap()
					.postings
					.push(UnbalancedPosting {
						line: *line,
						account: account.to_owned(),
						commodity: None,
						amount: None,
						comments: Vec::new(),
					});
				return Ok(());
			}
			Some(token) => match token {
				Token::PostingCommodity(_value, _line) => {}
				_ => {
					self
						.transactions
						.last_mut()
						.unwrap()
						.postings
						.push(UnbalancedPosting {
							line: *line,
							account: account.to_owned(),
							commodity: None,
							amount: None,
							comments: Vec::new(),
						});
					return Ok(());
				}
			},
		}

		let commodity = match self.tokens.get(self.index) {
			None => panic!("posting commodity not found"),
			Some(token) => match token {
				Token::PostingCommodity(_line, value) => {
					self.index += 1;
					value
				}
				_ => panic!("not a posting commodity"),
			},
		};

		let amount = match self.tokens.get(self.index) {
			None => panic!("posting amount not found"),
			Some(token) => match token {
				Token::PostingAmount(_line, value) => {
					self.index += 1;
					value
				}
				_ => panic!("not a posting amount"),
			},
		};

		self
			.transactions
			.last_mut()
			.unwrap()
			.postings
			.push(UnbalancedPosting {
				line: *line,
				account: account.to_owned(),
				commodity: Some(commodity.to_owned()),
				amount: Some(create_rational(&amount)?),
				comments: Vec::new(),
			});

		Ok(())
	}
}

fn create_rational(value: &str) -> Result<num::rational::Rational64, Error> {
	let (_, right) = if let Some(index) = value.find('.') {
		let (left, right) = value.split_at(index);
		let right = right.chars().skip(1).collect::<String>();
		(left, right)
	} else {
		(value, "".to_string())
	};
	let exponent: usize = right.chars().count();
	let numerator: i64 = value.replace('.', "").parse().unwrap();
	let denominator: i64 = 10_usize.pow(exponent as u32) as i64;
	Ok(num::rational::Rational64::new(numerator, denominator))
}
