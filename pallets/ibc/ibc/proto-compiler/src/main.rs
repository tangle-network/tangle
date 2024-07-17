// Copyright 2022 ComposableFi
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use argh::FromArgs;

mod cmd;
use cmd::{clone::CloneCmd, compile::CompileCmd};

#[derive(Debug, FromArgs)]
/// App
struct App {
	#[argh(subcommand)]
	cmd: Command,
}

#[derive(Debug, FromArgs)]
#[argh(subcommand)]
enum Command {
	Clone(CloneCmd),
	Compile(CompileCmd),
}

fn main() {
	let app: App = argh::from_env();

	match app.cmd {
		Command::Clone(clone) => clone.run(),
		Command::Compile(compile) => compile.run(),
	}
}
