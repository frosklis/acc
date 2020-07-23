extern crate num;

use super::errors::Error;

use super::model::BalancedPosting;
use super::model::Comment;
use super::model::Transaction;
use super::model::UnbalancedPosting;

use std::collections::HashSet;
use std::ops::Neg;

pub fn balance_transactions(
	unbalanced_transactions: &[Transaction<UnbalancedPosting>],
	balanced_transactions: &mut Vec<Transaction<BalancedPosting>>,
) -> Result<(), Error> {
	for unbalanced_transaction in unbalanced_transactions {
		let mut blanaced_postings = Vec::with_capacity(unbalanced_transaction.postings.len());
		let mut balanced_empty_posting = false;

		for unbalanced_posting in &unbalanced_transaction.postings {
			if unbalanced_posting.commodity.is_some() && unbalanced_posting.amount.is_some() {
				blanaced_postings.push(BalancedPosting {
					account: unbalanced_posting.account.to_owned(),
					commodity: unbalanced_posting.commodity.as_ref().unwrap().to_owned(),
					amount: unbalanced_posting.amount.unwrap(),
					comments: unbalanced_posting
						.comments
						.iter()
						.map(|c| Comment {
							line: c.line,
							comment: c.comment.to_owned(),
						})
						.collect(),
				})
			} else {
				if balanced_empty_posting {
					return Err(Error {
						line: unbalanced_posting.line + 1,
						message: format!("Only one posting with null amount allowed per transaction",),
					});
				}
				let total_commodities = total_commodities(&unbalanced_transaction);
				if total_commodities.len() > 1 {
					return Err(Error {
						line: unbalanced_posting.line + 1,
						message: format!(
							"Multiple commodities in transaction with a null amount posting not allowed"
						),
					});
				}
				blanaced_postings.push(BalancedPosting {
					account: unbalanced_posting.account.to_owned(),
					commodity: total_commodities.iter().next().unwrap().to_string(),
					amount: total_amount(&unbalanced_transaction).neg(),
					comments: unbalanced_posting
						.comments
						.iter()
						.map(|c| Comment {
							line: c.line,
							comment: c.comment.to_owned(),
						})
						.collect(),
				});
				balanced_empty_posting = true;
			}
		}
		balanced_transactions.push(Transaction {
			line: unbalanced_transaction.line,
			date: unbalanced_transaction.date.to_owned(),
			state: unbalanced_transaction.state.clone(),
			code: unbalanced_transaction.code.clone(),
			description: unbalanced_transaction.description.to_owned(),
			comments: unbalanced_transaction
				.comments
				.iter()
				.map(|c| Comment {
					line: c.line,
					comment: c.comment.to_owned(),
				})
				.collect(),
			postings: blanaced_postings,
		})
	}
	Ok(())
}

fn total_commodities(unbalanced_transaction: &Transaction<UnbalancedPosting>) -> HashSet<String> {
	unbalanced_transaction
		.postings
		.iter()
		.flat_map(|p| p.commodity.to_owned())
		.collect::<HashSet<String>>()
}

fn total_amount(
	unbalanced_transaction: &Transaction<UnbalancedPosting>,
) -> num::rational::Rational64 {
	unbalanced_transaction
		.postings
		.iter()
		.map(|p| p.amount)
		.fold(num::rational::Rational64::from_integer(0), |acc, val| {
			acc + val.unwrap_or_else(|| num::rational::Rational64::from_integer(0))
		})
}
