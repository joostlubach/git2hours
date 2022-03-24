extern crate chrono;
extern crate regex;

use std::{io::Error, process::{Command, Stdio}};
use chrono::{DateTime, FixedOffset};
use lazy_static::lazy_static;
use regex::Regex;

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
    command.args(["log", "--pretty=format:COMMIT\t%h\t%an\t%ad\t%cd"]);
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


  pub(super) fn parse_line(&self, line: &str) -> ParseLineResult {
    // A commit consists of one or two lines. The first line is a summary, which is formatted as:
    // COMMIT\t<HASH>\t<AUTHOR_NAME>\t<AUTHOR_DATE>\t<COMMIT_DATE>
    //
    // The second line is a line containing insertions and deletions, and looks like:
    //  <N> files changed, <N> insertions(+), <N> deletions(-)
    
    // First split everything into parts separated by tabs. That way, we can quickly check for the
    // COMMIT marker to identify the summary line.
    let parts: Vec<&str> = line.split('\t').collect();

    if parts[0] == "COMMIT" {
      let hash = String::from(parts[1]);
      let author_name = String::from(parts[2]);
      
      let author_date = DateTime::parse_from_rfc3339(parts[3]).ok();
      let commit_date = DateTime::parse_from_rfc3339(parts[4]).ok();
      if author_date.is_none() { return ParseLineResult::Unknown }
      if commit_date.is_none() { return ParseLineResult::Unknown }

      // if author_date != commit_date {
      //   // Some commits have a different author and commit date - these are merge, cherry pick or
      //   // rebase commits and we don't want to include them.
      //   return ParseLineResult::Unknown
      // }

      
      ParseLineResult::NewCommit(Commit {
        hash: hash,
        author: author_name,
        date: author_date.unwrap(),
        insertion_count: 0,
        deletion_count: 0
      })
    } else {
      match MODIFICATIONS_REGEX.captures(parts[0]) {
        Some(caps) => {
          let insertions: u32 = caps[1].parse().unwrap();
          let deletions: u32 = caps[2].parse().unwrap();

          ParseLineResult::Modifications(insertions, deletions)
        }
        None => ParseLineResult::Unknown
      }
    }
  }

  pub(super) fn match_commit(&self, commit: &Commit) -> bool {
    if self.authors.len() == 0 { return true }
    return self.authors.contains(&commit.author);
  }

}

// endregion

// region Support

lazy_static! {
  static ref MODIFICATIONS_REGEX: Regex = Regex::new(
    r"(\d+) insertions\(\+\).*?(\d+) deletions\(\-\)"
  ).unwrap();
}

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