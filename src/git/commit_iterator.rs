extern crate chrono;
extern crate regex;

use std::{io::{BufReader, BufRead, Lines}, process::{ChildStdout}};
use chrono::{DateTime};
use lazy_static::lazy_static;
use regex::Regex;

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
          match self.parse_line(&line) {
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

  fn parse_line(&self, line: &str) -> ParseLineResult {
    // A commit consists of one or two lines. The first line is a summary, which is formatted as:
    // COMMIT\t<HASH>\t<DATE>\t<AUTHOR>
    //
    // The second line is a line containing insertions and deletions, and looks like:
    //  <N> files changed, <N> insertions(+), <N> deletions(-)
    
    // First split everything into parts separated by tabs. That way, we can quickly check for the
    // COMMIT marker to identify the summary line.
    let parts: Vec<&str> = line.split('\t').collect();

    if parts[0] == "COMMIT" {
      match DateTime::parse_from_rfc3339(parts[2]) {
        Ok(date) => ParseLineResult::NewCommit(Commit {
          hash: String::from(parts[1]),
          author: String::from(parts[3]),
          date: date,
          insertion_count: 0,
          deletion_count: 0
        }),
        _ => ParseLineResult::Unknown
      }
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

// region Support

lazy_static! {
  static ref MODIFICATIONS_REGEX: Regex = Regex::new(
    r"(\d+) insertions\(\+\).*?(\d+) deletions\(\-\)"
  ).unwrap();
}

// endregion