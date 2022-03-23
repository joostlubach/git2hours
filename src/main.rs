#![allow(dead_code)]

#[macro_use]
extern crate derive_builder;

mod git;
mod counter;

use std::error::Error;
use counter::Counter;
use git::CommitQuery;
// use counter::Counter;

fn main() -> Result<(), Box<dyn Error>> {
  let working_dir = std::env::current_dir()?;
  let working_dir = working_dir.to_str().expect("Cannot determine working directory");
  
  let mut query = CommitQuery::new(working_dir);
  query.authors(["Joost Lubach", "joostlubach"]);

  // let counter = Counter::new(&query.run()?);
  
  for commit in query.run()? {
    println!("{:?}", commit);
  }

  Ok(())
}
