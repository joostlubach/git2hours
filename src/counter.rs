use crate::git::{CommitIterator, Commit};

pub struct Counter<'a> {
  commits: &'a mut CommitIterator<'a>
}

impl<'a> Counter<'a> {

  pub fn new(commits: &'a mut CommitIterator<'a>) -> Counter {
    Counter { commits }
  }

  fn count_hours(commit: &Commit) -> f32 {
    (commit.insertion_count as f32) * MINUTES_FOR_INSERTION / 60f32 +
    (commit.deletion_count as f32) * MINUTES_FOR_INSERTION / 60f32
  }

}

impl<'a> Iterator for Counter<'a> {

  type Item = (Commit, f32);

  fn next(&mut self) -> Option<(Commit, f32)> {
    match self.commits.next() {
      Some(commit) => {
        let hours = Self::count_hours(&commit);
        Some((commit, hours))
      },
      None => None
    }
  }



}

const MINUTES_FOR_INSERTION: f32 = 0.5f32;
const MINUTES_FOR_DELETION: f32 = 0.1f32;