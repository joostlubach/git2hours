use crate::git::CommitIterator;

pub struct Counter<'a> {
  commits: &'a CommitIterator<'a>
}

impl<'a> Counter<'a> {

  pub fn new(commits: &'a CommitIterator<'a>) -> Counter {
    Counter { commits }
  }

}