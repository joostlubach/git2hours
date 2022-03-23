#![allow(dead_code)]

#[macro_use]
extern crate derive_builder;

mod git;
mod counter;

use std::error::Error;
use counter::Counter;
use git::CommitQuery;
use itertools::Itertools;

fn main() -> Result<(), Box<dyn Error>> {
  let working_dir = std::env::current_dir()?;
  let working_dir = working_dir.to_str().expect("Cannot determine working directory");
  
  let mut query = CommitQuery::new(working_dir);
  query.authors(["Joost Lubach", "joostlubach"]);

  let mut commits = query.run()?;
  let counter = Counter::new(&mut commits);

  let grouped_by_day = counter.group_by(|(commit, _)| {
    commit.date.date()
  });

  for (date, commits) in &grouped_by_day {
    let hours: f32 = commits.map(|(_, hours)| hours).sum();
    println!("{}: {:.0} hours", date.format("%Y-%m-%d [%a]"), hours);
  }
  
  Ok(())
}
