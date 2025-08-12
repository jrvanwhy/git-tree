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

//! A wrapper around `git log` that heuristically determines what set of commits
//! should be displayed.

// The "interesting branches" are all local branches and all remote branches
// that are tracked by a local branch. The "interesting commits" are the commits
// pointed to by the interesting branches plus the HEAD commit. This tool
// displays the interesting commits, their collective merge bases, and any
// commits on the paths between the merge bases and the interesting commits.

mod includes_excludes;
mod interesting_branches;
mod merge_bases;

use includes_excludes::includes_excludes;
use interesting_branches::interesting_branches;
use merge_bases::merge_bases;
use std::env::args_os;
use std::process::Command;

fn main() {
    // Capacity estimate is a guess -- 4x as large as a SHA-256 hash seems
    // reasonable (and is a power of two).
    let mut buffer = Vec::with_capacity(256);
    let interesting_branches = interesting_branches(&mut buffer);
    let merge_bases = merge_bases(&mut buffer, &interesting_branches);
    let (includes, excludes) = includes_excludes(buffer, interesting_branches, &merge_bases);
    Command::new("git")
        .arg("log")
        .args(args_os().skip(1))
        .args(includes)
        .arg("--not")
        .args(merge_bases.into_iter().map(|mut id| {
            id.push_str("^@");
            id
        }))
        .args(excludes)
        .spawn()
        .expect("Failed to run git")
        .wait()
        .expect("failed to wait for git");
}
