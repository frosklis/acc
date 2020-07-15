use super::model::BalancedPosting;
use super::model::Transaction;
use super::printer::format_amount;
use super::printer_bal::group_postings_by_account;
use super::printer_bal::print_commodity_amount;

use colored::Colorize;

pub fn print(transactions: Vec<&Transaction<BalancedPosting>>) -> Result<(), String> {
	let postings = group_postings_by_account(transactions)?;
	let width = postings
		.values()
		.flat_map(|a| a.iter())
		.map(|(k, v)| k.chars().count() + format_amount(&v).chars().count())
		.max()
		.unwrap_or(0);
	for (account, amounts) in postings {
		let mut it = amounts.iter().peekable();
		while let Some((commodity, amount)) = it.next() {
			print_commodity_amount(commodity, amount, width);
			if it.peek().is_some() {
				println!();
			}
		}
		println!("{}", account.blue());
	}
	Ok(())
}
