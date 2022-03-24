extern crate chrono;
extern crate regex;

use std::{io::Error, process::{Command, Stdio}};
use chrono::{DateTime, FixedOffset};

use super::CommitIterator;

// region: CommitQuery

#[derive(Builder)]
pub struct CommitQuery {
  pub working_dir: String,

  pub skip: u32,
  pub limit: Option<u32>,

  pub authors: Vec<String>
}

impl CommitQuery {

  pub fn new(working_dir: &str) -> CommitQuery {
    CommitQuery {
      working_dir: String::from(working_dir),
      skip: 0,
      limit: None,
      authors: vec![]
    }
  }

  pub fn skip(&mut self, skip: u32) -> &Self {
    self.skip = skip;
    self
  }

  pub fn limit(&mut self, limit: u32) -> &Self {
    self.limit = Some(limit);
    self
  }

  pub fn author(&mut self, author: &str) -> &Self {
    self.authors.push(String::from(author));
    self
  }

  pub fn authors<'a, I>(&mut self, authors: I) -> &Self
  where
    I: IntoIterator<Item = &'a str>
  {
    for author in authors {
      self.author(author);
    }
    self
  }

  fn build_command(&self) -> Command {
    let mut command = Command::new("git");
    command.args(["log", "--pretty=format:COMMIT\t%h\t%cd\t%an"]);
    command.args(["--abbrev-commit", "--date=iso-strict", "--shortstat"]);

    if let Some(limit) = self.limit {
      command.args(["-n", &limit.to_string()]);
    }

    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());

    command
  }

  pub fn run(&self) -> Result<CommitIterator, Error> {
    let mut child = self.build_command().spawn()?;

    if let Some(stdout) = child.stdout.take() {
      return Ok(CommitIterator::new(self, stdout))
    } else {
      return Ok(CommitIterator::empty(self))
    }
  }

  pub(super) fn match_commit(&self, commit: &Commit) -> bool {
    if self.authors.len() == 0 { return true }
    return self.authors.contains(&commit.author);
  }

}

// endregion

// region Support

pub enum ParseLineResult {
  NewCommit(Commit),
  Modifications(u32, u32),
  Unknown
}

#[derive(Debug)]
pub struct Commit {
  pub hash: String,
  pub author: String,
  pub date: DateTime<FixedOffset>,
  pub insertion_count: u32,
  pub deletion_count: u32,
}

// endregion