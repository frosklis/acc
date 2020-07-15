extern crate num;

use super::model::BalancedPosting;
use super::model::Transaction;
use super::model::TransactionComment;
use super::model::UnbalancedPosting;

use std::collections::HashSet;
use std::ops::Neg;

pub fn balance_transactions<'a>(
	file: &str,
	unbalanced_transactions: &'a [Transaction<'a, UnbalancedPosting<'a>>],
) -> Result<Vec<Transaction<'a, BalancedPosting<'a>>>, String> {
	let mut balanced_transactions = Vec::with_capacity(unbalanced_transactions.len());

	for unbalanced_transaction in unbalanced_transactions {
		let mut blanaced_postings = Vec::with_capacity(unbalanced_transaction.postings.len());
		let mut balanced_empty_posting = false;

		for unbalanced_posting in &unbalanced_transaction.postings {
			if unbalanced_posting.commodity.is_some() && unbalanced_posting.amount.is_some() {
				blanaced_postings.push(BalancedPosting {
					account: unbalanced_posting.account,
					commodity: unbalanced_posting.commodity.unwrap(),
					amount: unbalanced_posting.amount.unwrap(),
				})
			} else {
				if balanced_empty_posting {
					return Err(format!("While parsing file {:?}, line {}:\nOnly one posting with null amount allowed per transaction",
					file,
						unbalanced_posting.line + 1
					));
				}
				let total_commodities = total_commodities(&unbalanced_transaction);
				if total_commodities.len() > 1 {
					return Err(format!("While parsing file {:?}, line {}:\nMultiple commodities in transaction with a null amount posting not allowed",
					file,
						unbalanced_posting.line + 1
					));
				}
				blanaced_postings.push(BalancedPosting {
					account: unbalanced_posting.account,
					commodity: total_commodities.iter().next().unwrap(),
					amount: total_amount(&unbalanced_transaction).neg(),
				});
				balanced_empty_posting = true;
			}
		}
		balanced_transactions.push(Transaction {
			line: unbalanced_transaction.line,
			date: unbalanced_transaction.date,
			state: unbalanced_transaction.state,
			description: unbalanced_transaction.description,
			comments: unbalanced_transaction
				.comments
				.iter()
				.map(|c| TransactionComment {
					line: c.line,
					comment: c.comment,
				})
				.collect(),
			postings: blanaced_postings,
		})
	}

	Ok(balanced_transactions)
}

fn total_commodities<'a>(
	unbalanced_transaction: &'a Transaction<UnbalancedPosting<'a>>,
) -> HashSet<&'a str> {
	unbalanced_transaction
		.postings
		.iter()
		.flat_map(|p| p.commodity)
		.collect::<HashSet<&str>>()
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
