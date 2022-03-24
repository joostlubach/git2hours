use crate::git::{CommitIterator, Commit};

pub struct Counter<'a> {
  commits: &'a mut CommitIterator<'a>,
  next_commit: Option<Commit>
}

impl<'a> Counter<'a> {

  pub fn new(commits: &'a mut CommitIterator<'a>) -> Counter {
    Counter { commits, next_commit: None }
  }

  fn count_hours(commit: &Commit, next_commit: Option<&Commit>) -> f32 {
    let capacity = next_commit
      .map(|next| Self::hours_between(&commit, next));

    let based_on_workload =
      (commit.insertion_count as f32) * MINUTES_FOR_INSERTION / 60f32 +
      (commit.deletion_count as f32) * MINUTES_FOR_INSERTION / 60f32;

    if let Some(capacity) = capacity {
      if capacity < based_on_workload { capacity } else { based_on_workload }
    } else {
      based_on_workload
    }
  }

  fn hours_between(prev: &Commit, next: &Commit) -> f32 {
    let minutes = prev.date.signed_duration_since(next.date).num_minutes() as f32;
    if minutes <= 0f32 {
      0f32
    } else {
      minutes / 60f32
    }
  }

}

impl<'a> Iterator for Counter<'a> {

  type Item = (Commit, f32);

  fn next(&mut self) -> Option<(Commit, f32)> {
    let commit = match self.next_commit.take() {
      Some(commit) => Some(commit),
      None => self.commits.next()
    };

    self.next_commit = self.commits.next();

    match commit {
      Some(commit) => {
        let hours = Self::count_hours(&commit, self.next_commit.as_ref());
        Some((commit, hours))
      },
      None => None
    }
  }



}

const MINUTES_FOR_INSERTION: f32 = 0.5f32;
const MINUTES_FOR_DELETION: f32 = 0.1f32;