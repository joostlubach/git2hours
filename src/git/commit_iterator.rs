extern crate chrono;
extern crate regex;

use std::{io::{BufReader, BufRead, Lines}, process::{ChildStdout}};

use super::{CommitQuery, commit_query::{Commit, ParseLineResult}};

// region: CommitIterator

pub struct CommitIterator<'a> {
  query: &'a CommitQuery,
  stdout: Option<Lines<BufReader<ChildStdout>>>,
  next_commit: Option<Commit>,
}

impl<'a> CommitIterator<'a> {

  pub(super) fn new(query: &'a CommitQuery, stdout: ChildStdout) -> CommitIterator {
    CommitIterator {
      query,
      stdout: Some(BufReader::new(stdout).lines()),
      next_commit: None
    }
  }

  pub(super) fn empty(query: &'a CommitQuery) -> CommitIterator {
    CommitIterator {
      query,
      stdout: None,
      next_commit: None
    }
  }

  fn read_next_commit(&mut self) -> Option<Commit> {
    let result = if let Some(commit) = self.next_commit.take() {
      // If we have some next commit waiting, use it now.
      ParseLineResult::NewCommit(commit)
    } else {
      // If not, parse the next line if found.
      self.parse_next_line()
    };

    if let ParseLineResult::NewCommit(mut commit) = result {
      // We've landed at a next commit. Immediately read the following line, to check if there's a 
      // modifications line. If not, we leave all the modification counts at 0. If so, apply the
      // modifications.

      match self.parse_next_line() {
        ParseLineResult::Modifications(insertion_count, deletion_count) => {
          // We've found a line of modification counts. Update the current commit and return it.

          commit.insertion_count = insertion_count;
          commit.deletion_count = deletion_count;
          Some(commit)
        },

        ParseLineResult::NewCommit(next_commit) => {
          // If it's a summary again, we've already read the next commit (we didn't know). Store
          // it around, but return the current commit.
          self.next_commit = Some(next_commit);
          Some(commit)
        }

        ParseLineResult::Unknown => {
          // There are no commits left. Return the current commit as-is.
          Some(commit)
        }
      }
    } else {
      None
    }
  }

  fn read_next_line(&mut self) -> Option<String> {
    match self.stdout.as_mut()?.next() {
      Some(Ok(line)) => Some(line),
      _ => None
    }
  }

  /// Loops through next lines, until a proper result is found. If not found, a
  /// [ParselineResult::Unknown] is returned.
  fn parse_next_line(&mut self) -> ParseLineResult {

    loop {
      match self.read_next_line() {
        Some(line) => {
          match self.query.parse_line(&line) {
            ParseLineResult::Unknown => { continue },
            other => return other
          }
        }
        None => {
          return ParseLineResult::Unknown
        }
      }
    }
  }

}

impl<'a> Iterator for CommitIterator<'a> {
  type Item = Commit;

  fn next(&mut self) -> Option<Commit> {
    loop {
      match self.read_next_commit() {
        Some(commit) => {
          if self.query.match_commit(&commit) {
            return Some(commit)
          } else {
            continue
          }
        }
        None => {
          return None 
        }
      }
    }
  }

}

// endregion