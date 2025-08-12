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

use core::str;
use std::io::{BufRead as _, BufReader};
use std::process::{Command, Stdio};

/// Returns all merge bases of the interesting commits.
/// Precondition: `buffer` must be empty
/// Postcondition: `buffer` will be empty
pub fn merge_bases(buffer: &mut Vec<u8>, interesting_branches: &Vec<String>) -> Vec<String> {
    let mut git = Command::new("git")
        .args(["merge-base", "-a", "--octopus", "HEAD"])
        .args(interesting_branches)
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to run git");
    let mut merge_bases = Vec::with_capacity(1);
    let mut reader = BufReader::new(git.stdout.as_mut().unwrap());
    while let Some(len) =
        reader.read_until(b'\n', buffer).expect("git stdout read failed").checked_sub(1)
    {
        // Reserve enough space for the merge base plus a trailing ^@ (used in
        // the final `git log` invocation).
        #[allow(
            clippy::arithmetic_side_effects,
            reason = "len is < the size of an allocation so adding 2 shouldn't overflow usize"
        )]
        let mut merge_base = String::with_capacity(len + 2);
        merge_base
            .push_str(str::from_utf8(buffer.get(..len).unwrap()).expect("non-utf-8 git output"));
        merge_bases.push(merge_base);
        buffer.clear();
    }
    drop(reader);
    let status = git.wait().expect("failed to wait for git");
    assert!(status.success(), "git returned unsuccessful status {status}");
    merge_bases
}
