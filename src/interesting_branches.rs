// Copyright 2025 Ryan Van Why
// Copyright 2024 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::collections::HashSet;
use std::io::{BufRead as _, BufReader};
use std::process::{Command, Stdio};

pub enum Error {
    /// `git` returned status code 128. This can indicate many things, but the
    /// most common is that the current directory is not in a git repository. In
    /// that case, `git` already printed a helpful error message, so printing
    /// another error message is not helpful. Unfortunately, there's not an easy
    /// way to detect that (reading git's stderr and stdout simultaneously
    /// requires nonblocking mode and a poll()-style syscall), so we just exit
    /// silently anytime `git` returns 128 and hope that `git` has already
    /// output a suitable error message to stdout.
    Git128,
}

/// Returns all interesting branches. Note that some commits may be in the list
/// multiple times under different names.
/// Precondition: `buffer` must be empty
/// Postcondition: `buffer` will be empty
#[allow(
    clippy::panic_in_result_fn,
    reason = "We'll decide how to handle non-128 statuses if we encounter them"
)]
pub fn interesting_branches(buffer: &mut Vec<u8>) -> Result<Vec<String>, Error> {
    // This considers a branch interesting if it is a local branch or if it has
    // the same name as a local branch.
    let mut git = Command::new("git")
        .args(["branch", "-a", "--format=%(refname)"])
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to run git");
    let mut locals = HashSet::new();
    let mut remotes = vec![];
    let mut reader = BufReader::new(git.stdout.as_mut().unwrap());
    while let Some(len) =
        reader.read_until(b'\n', buffer).expect("git stdout read failed").checked_sub(1)
    {
        if buffer.first_chunk() == Some(b"refs/remotes/") {
            remotes.push(buffer.get(b"refs/remotes/".len()..len).unwrap().to_vec());
        } else if buffer.first_chunk() == Some(b"refs/heads/") {
            locals.insert(buffer.get(b"refs/heads/".len()..len).unwrap().into());
        }
        buffer.clear();
    }
    drop(reader);
    let mut interesting = vec![];
    for remote in remotes {
        let Some(idx) = remote.iter().position(|&b| b == b'/') else { continue };
        #[allow(clippy::arithmetic_side_effects, reason = "idx is less than buffer.len()")]
        let (_, name) = remote.split_at(idx + 1);
        if locals.contains(name) {
            interesting.push(String::from_utf8(remote).expect("non-utf-8 branch"));
        }
    }
    interesting.extend(
        locals.into_iter().map(|local| String::from_utf8(local).expect("non-utf-8 branch")),
    );
    let status = git.wait().expect("failed to wait for git");
    if status.code() == Some(128) {
        return Err(Error::Git128);
    }
    assert!(status.success(), "git returned unsuccessful status {status}");
    Ok(interesting)
}
