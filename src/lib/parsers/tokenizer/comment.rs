use super::super::super::model::Comment;
use super::chars;
use super::Tokenizer;

pub(super) fn tokenize_journal_comment(tokenizer: &mut Tokenizer) -> Result<(), String> {
	if let Some(comment) = tokenize_comment(tokenizer)? {
		// println!("journal comment : {}", comment);
	}
	Ok(())
}

pub(super) fn tokenize_indented_comment(tokenizer: &mut Tokenizer) -> Result<(), String> {
	if let Some(comment) = tokenize_comment(tokenizer)? {
		match tokenizer.transactions.last_mut() {
			None => return Err(String::from("invalid position for comment")),
			Some(transaction) => match transaction.unbalanced_postings.last_mut() {
				None => transaction.comments.push(Comment {
					line: tokenizer.line_index + 1,
					comment,
				}),
				Some(p) => p.comments.push(Comment {
					line: tokenizer.line_index + 1,
					comment,
				}),
			},
		}
	}
	Ok(())
}

fn tokenize_comment(tokenizer: &mut Tokenizer) -> Result<Option<String>, String> {
	if chars::consume(tokenizer, |c| c == ';') {
		chars::consume(tokenizer, char::is_whitespace);

		let mut comment = String::new();

		while let Some(&c) = tokenizer.line_characters.get(tokenizer.line_position) {
			comment.push(c);
			tokenizer.line_position += 1;
		}

		return Ok(Some(comment));
	}
	Ok(None)
}
