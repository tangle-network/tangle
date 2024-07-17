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

/// Downcast the given arguments to the associated enum variant.
///
/// ## Note
/// Only works for enums whose variants only hold a single value.
///
/// ## Example
///
/// ```rust
/// use ibc::downcast;
///
/// enum Foo {
///     Bar(u32),
///     Baz(bool),
/// }
///
/// let bar = Foo::Bar(42);
/// let baz = Foo::Baz(true);
///
/// if let Some(value) = downcast!(bar => Foo::Bar) {
///     println!("value is a u32: {}", value);
/// }
///
/// if let Some(value) = downcast!(baz => Foo::Baz) {
///     println!("value is a bool: {}", value);
/// }
///
/// if let Some((a, b)) = downcast!(bar => Foo::Bar, baz => Foo::Baz) {
///     println!("a is a u32: {}", a);
///     println!("b is a bool: {}", b);
/// }
/// ```
#[macro_export]
macro_rules! downcast {
    ( $e1:expr => $p1:path, $( $e:expr => $p:path ),+ $(,)? ) => {
        $crate::downcast!($e1 => $p1).zip($crate::downcast!($($e => $p),+))
    };

    ($e:expr => $p:path) => {
        match $e {
            $p(e) => Some(e),
            #[allow(unreachable_patterns)]
            _ => None,
        }
    };

    () => {
        None
    }
}
