// Copyright 2012 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

/*!
 * Simple getopt alternative.
 *
 * Construct a vector of options, either by using reqopt, optopt, and optflag
 * or by building them from components yourself, and pass them to getopts,
 * along with a vector of actual arguments (not including argv[0]). You'll
 * either get a failure code back, or a match. You'll have to verify whether
 * the amount of 'free' arguments in the match is what you expect. Use opt_*
 * accessors to get argument values out of the matches object.
 *
 * Single-character options are expected to appear on the command line with a
 * single preceding dash; multiple-character options are expected to be
 * proceeded by two dashes. Options that expect an argument accept their
 * argument following either a space or an equals sign. Single-character
 * options don't require the space.
 *
 * # Example
 *
 * The following example shows simple command line parsing for an application
 * that requires an input file to be specified, accepts an optional output
 * file name following -o, and accepts both -h and --help as optional flags.
 *
 *     use std;
 *     import std::getopts::{optopt,optflag,getopts,opt_present,opt_maybe_str,
 *         fail_str};
 *
 *     fn do_work(in: str, out: Option<str>) {
 *         // ...
 *     }
 *
 *     fn print_usage(program: str) {
 *         io::println("Usage: " + program + " [options]");
 *         io::println("-o\t\tOutput");
 *         io::println("-h --help\tUsage");
 *     }
 *
 *     fn main(args: ~[str]) {
 *         check !args.is_empty()
 *
 *         let program : str = vec::head(args);
 *
 *         let opts = ~[
 *             optopt("o"),
 *             optflag("h"),
 *             optflag("help")
 *         ];
 *         let matches = match getopts(vec::tail(args), opts) {
 *             result::ok(m) { m }
 *             result::err(f) { die!(fail_str(f)) }
 *         };
 *         if opt_present(matches, "h") || opt_present(matches, "help") {
 *             print_usage(program);
 *             return;
 *         }
 *         let output = opt_maybe_str(matches, "o");
 *         let input = if !matches.free.is_empty() {
 *             matches.free[0]
 *         } else {
 *             print_usage(program);
 *             return;
 *         };
 *         do_work(input, output);
 *     }
 */

use core::cmp::Eq;
use core::prelude::*;
use core::result::{Err, Ok};
use core::result;
use core::option;
use core::option::{Some, None};
use core::str;
use core::vec;

#[deriving_eq]
pub enum Name {
    Long(~str),
    Short(char),
}

#[deriving_eq]
pub enum HasArg { Yes, No, Maybe, }

#[deriving_eq]
pub enum Occur { Req, Optional, Multi, }

/// A description of a possible option
#[deriving_eq]
pub struct Opt {
    name: Name,
    hasarg: HasArg,
    occur: Occur
}

fn mkname(nm: &str) -> Name {
    let unm = str::from_slice(nm);
    return if nm.len() == 1u {
            Short(str::char_at(unm, 0u))
        } else { Long(unm) };
}

/// Create an option that is required and takes an argument
pub fn reqopt(name: &str) -> Opt {
    return Opt {name: mkname(name), hasarg: Yes, occur: Req};
}

/// Create an option that is optional and takes an argument
pub fn optopt(name: &str) -> Opt {
    return Opt {name: mkname(name), hasarg: Yes, occur: Optional};
}

/// Create an option that is optional and does not take an argument
pub fn optflag(name: &str) -> Opt {
    return Opt {name: mkname(name), hasarg: No, occur: Optional};
}

/// Create an option that is optional and does not take an argument
pub fn optflagmulti(name: &str) -> Opt {
    return Opt {name: mkname(name), hasarg: No, occur: Multi};
}

/// Create an option that is optional and takes an optional argument
pub fn optflagopt(name: &str) -> Opt {
    return Opt {name: mkname(name), hasarg: Maybe, occur: Optional};
}

/**
 * Create an option that is optional, takes an argument, and may occur
 * multiple times
 */
pub fn optmulti(name: &str) -> Opt {
    return Opt {name: mkname(name), hasarg: Yes, occur: Multi};
}

#[deriving_eq]
enum Optval { Val(~str), Given, }

/**
 * The result of checking command line arguments. Contains a vector
 * of matches and a vector of free strings.
 */
#[deriving_eq]
pub struct Matches {
    opts: ~[Opt],
    vals: ~[~[Optval]],
    free: ~[~str]
}

fn is_arg(arg: &str) -> bool {
    return arg.len() > 1 && arg[0] == '-' as u8;
}

fn name_str(nm: &Name) -> ~str {
    return match *nm {
      Short(ch) => str::from_char(ch),
      Long(copy s) => s
    };
}

fn find_opt(opts: &[Opt], nm: Name) -> Option<uint> {
    vec::position(opts, |opt| opt.name == nm)
}

/**
 * The type returned when the command line does not conform to the
 * expected format. Pass this value to <fail_str> to get an error message.
 */
#[deriving_eq]
pub enum Fail_ {
    ArgumentMissing(~str),
    UnrecognizedOption(~str),
    OptionMissing(~str),
    OptionDuplicated(~str),
    UnexpectedArgument(~str),
}

/// Convert a `fail_` enum into an error string
pub fn fail_str(f: Fail_) -> ~str {
    return match f {
        ArgumentMissing(ref nm) => {
            ~"Argument to option '" + *nm + ~"' missing."
        }
        UnrecognizedOption(ref nm) => {
            ~"Unrecognized option: '" + *nm + ~"'."
        }
        OptionMissing(ref nm) => {
            ~"Required option '" + *nm + ~"' missing."
        }
        OptionDuplicated(ref nm) => {
            ~"Option '" + *nm + ~"' given more than once."
        }
        UnexpectedArgument(ref nm) => {
            ~"Option " + *nm + ~" does not take an argument."
        }
    };
}

/**
 * The result of parsing a command line with a set of options
 * (result::t<Matches, Fail_>)
 */
pub type Result = result::Result<Matches, Fail_>;

/**
 * Parse command line arguments according to the provided options
 *
 * On success returns `ok(Opt)`. Use functions such as `opt_present`
 * `opt_str`, etc. to interrogate results.  Returns `err(Fail_)` on failure.
 * Use <fail_str> to get an error message.
 */
pub fn getopts(args: &[~str], opts: &[Opt]) -> Result {
    unsafe {
        let n_opts = opts.len();
        fn f(_x: uint) -> ~[Optval] { return ~[]; }
        let vals = vec::cast_to_mut(vec::from_fn(n_opts, f));
        let mut free: ~[~str] = ~[];
        let l = args.len();
        let mut i = 0;
        while i < l {
            let cur = args[i];
            let curlen = cur.len();
            if !is_arg(cur) {
                free.push(cur);
            } else if cur == ~"--" {
                let mut j = i + 1;
                while j < l { free.push(args[j]); j += 1; }
                break;
            } else {
                let mut names;
                let mut i_arg = None;
                if cur[1] == '-' as u8 {
                    let tail = str::slice(cur, 2, curlen);
                    let tail_eq = str::splitn_char(tail, '=', 1);
                    if tail_eq.len() <= 1 {
                        names = ~[Long(tail)];
                    } else {
                        names =
                            ~[Long(tail_eq[0])];
                        i_arg = Some(tail_eq[1]);
                    }
                } else {
                    let mut j = 1;
                    let mut last_valid_opt_id = None;
                    names = ~[];
                    while j < curlen {
                        let range = str::char_range_at(cur, j);
                        let opt = Short(range.ch);

                        /* In a series of potential options (eg. -aheJ), if we
                           see one which takes an argument, we assume all
                           subsequent characters make up the argument. This
                           allows options such as -L/usr/local/lib/foo to be
                           interpreted correctly
                        */

                        match find_opt(opts, opt) {
                          Some(id) => last_valid_opt_id = Some(id),
                          None => {
                            let arg_follows =
                                last_valid_opt_id.is_some() &&
                                match opts[last_valid_opt_id.get()]
                                  .hasarg {

                                  Yes | Maybe => true,
                                  No => false
                                };
                            if arg_follows && j < curlen {
                                i_arg = Some(cur.slice(j, curlen));
                                break;
                            } else {
                                last_valid_opt_id = None;
                            }
                          }
                        }
                        names.push(opt);
                        j = range.next;
                    }
                }
                let mut name_pos = 0;
                for names.each() |nm| {
                    name_pos += 1;
                    let optid = match find_opt(opts, *nm) {
                      Some(id) => id,
                      None => return Err(UnrecognizedOption(name_str(nm)))
                    };
                    match opts[optid].hasarg {
                      No => {
                        if !i_arg.is_none() {
                            return Err(UnexpectedArgument(name_str(nm)));
                        }
                        vals[optid].push(Given);
                      }
                      Maybe => {
                        if !i_arg.is_none() {
                            vals[optid].push(Val(i_arg.get()));
                        } else if name_pos < names.len() ||
                                      i + 1 == l || is_arg(args[i + 1]) {
                            vals[optid].push(Given);
                        } else { i += 1; vals[optid].push(Val(args[i])); }
                      }
                      Yes => {
                        if !i_arg.is_none() {
                            vals[optid].push(Val(i_arg.get()));
                        } else if i + 1 == l {
                            return Err(ArgumentMissing(name_str(nm)));
                        } else { i += 1; vals[optid].push(Val(args[i])); }
                      }
                    }
                }
            }
            i += 1;
        }
        i = 0u;
        while i < n_opts {
            let n = vals[i].len();
            let occ = opts[i].occur;
            if occ == Req {
                if n == 0 {
                    return Err(OptionMissing(name_str(&(opts[i].name))));
                }
            }
            if occ != Multi {
                if n > 1 {
                    return Err(OptionDuplicated(name_str(&(opts[i].name))));
                }
            }
            i += 1;
        }
        return Ok(Matches {opts: vec::from_slice(opts),
                   vals: vec::cast_from_mut(move vals),
                   free: free});
    }
}

fn opt_vals(mm: &Matches, nm: &str) -> ~[Optval] {
    return match find_opt(mm.opts, mkname(nm)) {
      Some(id) => mm.vals[id],
      None => {
        error!("No option '%s' defined", nm);
        die!()
      }
    };
}

fn opt_val(mm: &Matches, nm: &str) -> Optval { return opt_vals(mm, nm)[0]; }

/// Returns true if an option was matched
pub fn opt_present(mm: &Matches, nm: &str) -> bool {
    !opt_vals(mm, nm).is_empty()
}

/// Returns the number of times an option was matched
pub fn opt_count(mm: &Matches, nm: &str) -> uint {
    opt_vals(mm, nm).len()
}

/// Returns true if any of several options were matched
pub fn opts_present(mm: &Matches, names: &[~str]) -> bool {
    for vec::each(names) |nm| {
        match find_opt(mm.opts, mkname(*nm)) {
          Some(_) => return true,
          None    => ()
        }
    }
    false
}


/**
 * Returns the string argument supplied to a matching option
 *
 * Fails if the option was not matched or if the match did not take an
 * argument
 */
pub fn opt_str(mm: &Matches, nm: &str) -> ~str {
    return match opt_val(mm, nm) { Val(copy s) => s, _ => die!() };
}

/**
 * Returns the string argument supplied to one of several matching options
 *
 * Fails if the no option was provided from the given list, or if the no such
 * option took an argument
 */
pub fn opts_str(mm: &Matches, names: &[~str]) -> ~str {
    for vec::each(names) |nm| {
        match opt_val(mm, *nm) {
          Val(copy s) => return s,
          _ => ()
        }
    }
    die!();
}


/**
 * Returns a vector of the arguments provided to all matches of the given
 * option.
 *
 * Used when an option accepts multiple values.
 */
pub fn opt_strs(mm: &Matches, nm: &str) -> ~[~str] {
    let mut acc: ~[~str] = ~[];
    for vec::each(opt_vals(mm, nm)) |v| {
        match *v { Val(copy s) => acc.push(s), _ => () }
    }
    return acc;
}

/// Returns the string argument supplied to a matching option or none
pub fn opt_maybe_str(mm: &Matches, nm: &str) -> Option<~str> {
    let vals = opt_vals(mm, nm);
    if vec::len::<Optval>(vals) == 0u { return None::<~str>; }
    return match vals[0] {
        Val(copy s) => Some(s),
        _ => None
    };
}


/**
 * Returns the matching string, a default, or none
 *
 * Returns none if the option was not present, `def` if the option was
 * present but no argument was provided, and the argument if the option was
 * present and an argument was provided.
 */
pub fn opt_default(mm: &Matches, nm: &str, def: &str) -> Option<~str> {
    let vals = opt_vals(mm, nm);
    if vec::len::<Optval>(vals) == 0u { return None::<~str>; }
    return match vals[0] { Val(copy s) => Some::<~str>(s),
                           _      => Some::<~str>(str::from_slice(def)) }
}

#[deriving_eq]
enum FailType {
    ArgumentMissing_,
    UnrecognizedOption_,
    OptionMissing_,
    OptionDuplicated_,
    UnexpectedArgument_,
}

/** A module which provides a way to specify descriptions and
 *  groups of short and long option names, together.
 */
pub mod groups {
    use getopts::{HasArg, Long, Maybe, Multi, No, Occur, Opt, Optional, Req};
    use getopts::{Result, Short, Yes};

    use core::prelude::*;
    use core::str;
    use core::vec;

    /** one group of options, e.g., both -h and --help, along with
     * their shared description and properties
     */
    #[deriving_eq]
    pub struct OptGroup {
        short_name: ~str,
        long_name: ~str,
        hint: ~str,
        desc: ~str,
        hasarg: HasArg,
        occur: Occur
    }

    /// Create a long option that is required and takes an argument
    pub fn reqopt(short_name: &str, long_name: &str,
                  desc: &str, hint: &str) -> OptGroup {
        let len = short_name.len();
        assert len == 1 || len == 0;
        return OptGroup { short_name: str::from_slice(short_name),
                long_name: str::from_slice(long_name),
                hint: str::from_slice(hint),
                desc: str::from_slice(desc),
                hasarg: Yes,
                occur: Req};
    }

    /// Create a long option that is optional and takes an argument
    pub fn optopt(short_name: &str, long_name: &str,
                  desc: &str, hint: &str) -> OptGroup {
        let len = short_name.len();
        assert len == 1 || len == 0;
        return OptGroup {short_name: str::from_slice(short_name),
                long_name: str::from_slice(long_name),
                hint: str::from_slice(hint),
                desc: str::from_slice(desc),
                hasarg: Yes,
                occur: Optional};
    }

    /// Create a long option that is optional and does not take an argument
    pub fn optflag(short_name: &str, long_name: &str,
                   desc: &str) -> OptGroup {
        let len = short_name.len();
        assert len == 1 || len == 0;
        return OptGroup {short_name: str::from_slice(short_name),
                long_name: str::from_slice(long_name),
                hint: ~"",
                desc: str::from_slice(desc),
                hasarg: No,
                occur: Optional};
    }

    /// Create a long option that is optional and takes an optional argument
    pub fn optflagopt(short_name: &str, long_name: &str,
                      desc: &str, hint: &str) -> OptGroup {
        let len = short_name.len();
        assert len == 1 || len == 0;
        return OptGroup {short_name: str::from_slice(short_name),
                long_name: str::from_slice(long_name),
                hint: str::from_slice(hint),
                desc: str::from_slice(desc),
                hasarg: Maybe,
                occur: Optional};
    }

    /**
     * Create a long option that is optional, takes an argument, and may occur
     * multiple times
     */
    pub fn optmulti(short_name: &str, long_name: &str,
                    desc: &str, hint: &str) -> OptGroup {
        let len = short_name.len();
        assert len == 1 || len == 0;
        return OptGroup {short_name: str::from_slice(short_name),
                long_name: str::from_slice(long_name),
                hint: str::from_slice(hint),
                desc: str::from_slice(desc),
                hasarg: Yes,
                occur: Multi};
    }

    // translate OptGroup into Opt
    // (both short and long names correspond to different Opts)
    pub fn long_to_short(lopt: &OptGroup) -> ~[Opt] {
        match ((*lopt).short_name.len(),
               (*lopt).long_name.len()) {

           (0,0) => die!(~"this long-format option was given no name"),

           (0,_) => ~[Opt {name:   Long(((*lopt).long_name)),
                           hasarg: (*lopt).hasarg,
                           occur:  (*lopt).occur}],

           (1,0) => ~[Opt {name: Short(str::char_at((*lopt).short_name, 0)),
                           hasarg: (*lopt).hasarg,
                           occur:  (*lopt).occur}],

           (1,_) => ~[Opt {name: Short(str::char_at((*lopt).short_name, 0)),
                           hasarg: (*lopt).hasarg,
                           occur:  (*lopt).occur},
                      Opt {name:   Long(((*lopt).long_name)),
                           hasarg: (*lopt).hasarg,
                           occur:  (*lopt).occur}],

           (_,_) => die!(~"something is wrong with the long-form opt")
        }
    }

    /*
     * Parse command line args with the provided long format options
     */
    pub fn getopts(args: &[~str], opts: &[OptGroup]) -> ::getopts::Result {
        ::getopts::getopts(args, vec::flat_map(opts, long_to_short))
    }

    /**
     * Derive a usage message from a set of long options
     */
    pub fn usage(brief: &str, opts: &[OptGroup]) -> ~str {

        let desc_sep = ~"\n" + str::repeat(~" ", 24);

        let rows = vec::map(opts, |optref| {
            let short_name = (*optref).short_name;
            let long_name = (*optref).long_name;
            let hint = (*optref).hint;
            let desc = (*optref).desc;
            let hasarg = (*optref).hasarg;

            let mut row = str::repeat(~" ", 4);

            // short option
            row += match short_name.len() {
                0 => ~"",
                1 => ~"-" + short_name + " ",
                _ => die!(~"the short name should only be 1 char long"),
            };

            // long option
            row += match long_name.len() {
                0 => ~"",
                _ => ~"--" + long_name + " ",
            };

            // arg
            row += match hasarg {
                No    => ~"",
                Yes   => hint,
                Maybe => ~"[" + hint + ~"]",
            };

            // here we just need to indent the start of the description
            let rowlen = row.len();
            row += if rowlen < 24 {
                str::repeat(~" ", 24 - rowlen)
            } else {
                desc_sep
            };

            // wrapped description
            row += str::connect(str::split_within(desc, 54), desc_sep);

            row
        });

        return str::from_slice(brief)    +
               ~"\n\nOptions:\n"         +
               str::connect(rows, ~"\n") +
               ~"\n\n";
    }
} // end groups module

#[cfg(test)]
mod tests {
    use core::prelude::*;

    use opt = getopts;
    use getopts::groups::OptGroup;
    use getopts::*;

    use core::result::{Err, Ok};
    use core::result;

    pub fn check_fail_type(f: Fail_, ft: FailType) {
        match f {
          ArgumentMissing(_) => assert ft == ArgumentMissing_,
          UnrecognizedOption(_) => assert ft == UnrecognizedOption_,
          OptionMissing(_) => assert ft == OptionMissing_,
          OptionDuplicated(_) => assert ft == OptionDuplicated_,
          UnexpectedArgument(_) => assert ft == UnexpectedArgument_
        }
    }


    // Tests for reqopt
    #[test]
    pub fn test_reqopt_long() {
        let args = ~[~"--test=20"];
        let opts = ~[reqopt(~"test")];
        let rs = getopts(args, opts);
        match rs {
          Ok(ref m) => {
            assert (opt_present(m, ~"test"));
            assert (opt_str(m, ~"test") == ~"20");
          }
          _ => { die!(~"test_reqopt_long failed"); }
        }
    }

    #[test]
    pub fn test_reqopt_long_missing() {
        let args = ~[~"blah"];
        let opts = ~[reqopt(~"test")];
        let rs = getopts(args, opts);
        match rs {
          Err(copy f) => check_fail_type(f, OptionMissing_),
          _ => die!()
        }
    }

    #[test]
    pub fn test_reqopt_long_no_arg() {
        let args = ~[~"--test"];
        let opts = ~[reqopt(~"test")];
        let rs = getopts(args, opts);
        match rs {
          Err(copy f) => check_fail_type(f, ArgumentMissing_),
          _ => die!()
        }
    }

    #[test]
    pub fn test_reqopt_long_multi() {
        let args = ~[~"--test=20", ~"--test=30"];
        let opts = ~[reqopt(~"test")];
        let rs = getopts(args, opts);
        match rs {
          Err(copy f) => check_fail_type(f, OptionDuplicated_),
          _ => die!()
        }
    }

    #[test]
    pub fn test_reqopt_short() {
        let args = ~[~"-t", ~"20"];
        let opts = ~[reqopt(~"t")];
        let rs = getopts(args, opts);
        match rs {
          Ok(ref m) => {
            assert (opt_present(m, ~"t"));
            assert (opt_str(m, ~"t") == ~"20");
          }
          _ => die!()
        }
    }

    #[test]
    pub fn test_reqopt_short_missing() {
        let args = ~[~"blah"];
        let opts = ~[reqopt(~"t")];
        let rs = getopts(args, opts);
        match rs {
          Err(copy f) => check_fail_type(f, OptionMissing_),
          _ => die!()
        }
    }

    #[test]
    pub fn test_reqopt_short_no_arg() {
        let args = ~[~"-t"];
        let opts = ~[reqopt(~"t")];
        let rs = getopts(args, opts);
        match rs {
          Err(copy f) => check_fail_type(f, ArgumentMissing_),
          _ => die!()
        }
    }

    #[test]
    pub fn test_reqopt_short_multi() {
        let args = ~[~"-t", ~"20", ~"-t", ~"30"];
        let opts = ~[reqopt(~"t")];
        let rs = getopts(args, opts);
        match rs {
          Err(copy f) => check_fail_type(f, OptionDuplicated_),
          _ => die!()
        }
    }


    // Tests for optopt
    #[test]
    pub fn test_optopt_long() {
        let args = ~[~"--test=20"];
        let opts = ~[optopt(~"test")];
        let rs = getopts(args, opts);
        match rs {
          Ok(ref m) => {
            assert (opt_present(m, ~"test"));
            assert (opt_str(m, ~"test") == ~"20");
          }
          _ => die!()
        }
    }

    #[test]
    pub fn test_optopt_long_missing() {
        let args = ~[~"blah"];
        let opts = ~[optopt(~"test")];
        let rs = getopts(args, opts);
        match rs {
          Ok(ref m) => assert (!opt_present(m, ~"test")),
          _ => die!()
        }
    }

    #[test]
    pub fn test_optopt_long_no_arg() {
        let args = ~[~"--test"];
        let opts = ~[optopt(~"test")];
        let rs = getopts(args, opts);
        match rs {
          Err(copy f) => check_fail_type(f, ArgumentMissing_),
          _ => die!()
        }
    }

    #[test]
    pub fn test_optopt_long_multi() {
        let args = ~[~"--test=20", ~"--test=30"];
        let opts = ~[optopt(~"test")];
        let rs = getopts(args, opts);
        match rs {
          Err(copy f) => check_fail_type(f, OptionDuplicated_),
          _ => die!()
        }
    }

    #[test]
    pub fn test_optopt_short() {
        let args = ~[~"-t", ~"20"];
        let opts = ~[optopt(~"t")];
        let rs = getopts(args, opts);
        match rs {
          Ok(ref m) => {
            assert (opt_present(m, ~"t"));
            assert (opt_str(m, ~"t") == ~"20");
          }
          _ => die!()
        }
    }

    #[test]
    pub fn test_optopt_short_missing() {
        let args = ~[~"blah"];
        let opts = ~[optopt(~"t")];
        let rs = getopts(args, opts);
        match rs {
          Ok(ref m) => assert (!opt_present(m, ~"t")),
          _ => die!()
        }
    }

    #[test]
    pub fn test_optopt_short_no_arg() {
        let args = ~[~"-t"];
        let opts = ~[optopt(~"t")];
        let rs = getopts(args, opts);
        match rs {
          Err(copy f) => check_fail_type(f, ArgumentMissing_),
          _ => die!()
        }
    }

    #[test]
    pub fn test_optopt_short_multi() {
        let args = ~[~"-t", ~"20", ~"-t", ~"30"];
        let opts = ~[optopt(~"t")];
        let rs = getopts(args, opts);
        match rs {
          Err(copy f) => check_fail_type(f, OptionDuplicated_),
          _ => die!()
        }
    }


    // Tests for optflag
    #[test]
    pub fn test_optflag_long() {
        let args = ~[~"--test"];
        let opts = ~[optflag(~"test")];
        let rs = getopts(args, opts);
        match rs {
          Ok(ref m) => assert (opt_present(m, ~"test")),
          _ => die!()
        }
    }

    #[test]
    pub fn test_optflag_long_missing() {
        let args = ~[~"blah"];
        let opts = ~[optflag(~"test")];
        let rs = getopts(args, opts);
        match rs {
          Ok(ref m) => assert (!opt_present(m, ~"test")),
          _ => die!()
        }
    }

    #[test]
    pub fn test_optflag_long_arg() {
        let args = ~[~"--test=20"];
        let opts = ~[optflag(~"test")];
        let rs = getopts(args, opts);
        match rs {
          Err(copy f) => {
            log(error, fail_str(f));
            check_fail_type(f, UnexpectedArgument_);
          }
          _ => die!()
        }
    }

    #[test]
    pub fn test_optflag_long_multi() {
        let args = ~[~"--test", ~"--test"];
        let opts = ~[optflag(~"test")];
        let rs = getopts(args, opts);
        match rs {
          Err(copy f) => check_fail_type(f, OptionDuplicated_),
          _ => die!()
        }
    }

    #[test]
    pub fn test_optflag_short() {
        let args = ~[~"-t"];
        let opts = ~[optflag(~"t")];
        let rs = getopts(args, opts);
        match rs {
          Ok(ref m) => assert (opt_present(m, ~"t")),
          _ => die!()
        }
    }

    #[test]
    pub fn test_optflag_short_missing() {
        let args = ~[~"blah"];
        let opts = ~[optflag(~"t")];
        let rs = getopts(args, opts);
        match rs {
          Ok(ref m) => assert (!opt_present(m, ~"t")),
          _ => die!()
        }
    }

    #[test]
    pub fn test_optflag_short_arg() {
        let args = ~[~"-t", ~"20"];
        let opts = ~[optflag(~"t")];
        let rs = getopts(args, opts);
        match rs {
          Ok(ref m) => {
            // The next variable after the flag is just a free argument

            assert (m.free[0] == ~"20");
          }
          _ => die!()
        }
    }

    #[test]
    pub fn test_optflag_short_multi() {
        let args = ~[~"-t", ~"-t"];
        let opts = ~[optflag(~"t")];
        let rs = getopts(args, opts);
        match rs {
          Err(copy f) => check_fail_type(f, OptionDuplicated_),
          _ => die!()
        }
    }

    // Tests for optflagmulti
    #[test]
    pub fn test_optflagmulti_short1() {
        let args = ~[~"-v"];
        let opts = ~[optflagmulti(~"v")];
        let rs = getopts(args, opts);
        match rs {
          Ok(ref m) => {
            assert (opt_count(m, ~"v") == 1);
          }
          _ => die!()
        }
    }

    #[test]
    pub fn test_optflagmulti_short2a() {
        let args = ~[~"-v", ~"-v"];
        let opts = ~[optflagmulti(~"v")];
        let rs = getopts(args, opts);
        match rs {
          Ok(ref m) => {
            assert (opt_count(m, ~"v") == 2);
          }
          _ => die!()
        }
    }

    #[test]
    pub fn test_optflagmulti_short2b() {
        let args = ~[~"-vv"];
        let opts = ~[optflagmulti(~"v")];
        let rs = getopts(args, opts);
        match rs {
          Ok(ref m) => {
            assert (opt_count(m, ~"v") == 2);
          }
          _ => die!()
        }
    }

    #[test]
    pub fn test_optflagmulti_long1() {
        let args = ~[~"--verbose"];
        let opts = ~[optflagmulti(~"verbose")];
        let rs = getopts(args, opts);
        match rs {
          Ok(ref m) => {
            assert (opt_count(m, ~"verbose") == 1);
          }
          _ => die!()
        }
    }

    #[test]
    pub fn test_optflagmulti_long2() {
        let args = ~[~"--verbose", ~"--verbose"];
        let opts = ~[optflagmulti(~"verbose")];
        let rs = getopts(args, opts);
        match rs {
          Ok(ref m) => {
            assert (opt_count(m, ~"verbose") == 2);
          }
          _ => die!()
        }
    }

    // Tests for optmulti
    #[test]
    pub fn test_optmulti_long() {
        let args = ~[~"--test=20"];
        let opts = ~[optmulti(~"test")];
        let rs = getopts(args, opts);
        match rs {
          Ok(ref m) => {
            assert (opt_present(m, ~"test"));
            assert (opt_str(m, ~"test") == ~"20");
          }
          _ => die!()
        }
    }

    #[test]
    pub fn test_optmulti_long_missing() {
        let args = ~[~"blah"];
        let opts = ~[optmulti(~"test")];
        let rs = getopts(args, opts);
        match rs {
          Ok(ref m) => assert (!opt_present(m, ~"test")),
          _ => die!()
        }
    }

    #[test]
    pub fn test_optmulti_long_no_arg() {
        let args = ~[~"--test"];
        let opts = ~[optmulti(~"test")];
        let rs = getopts(args, opts);
        match rs {
          Err(copy f) => check_fail_type(f, ArgumentMissing_),
          _ => die!()
        }
    }

    #[test]
    pub fn test_optmulti_long_multi() {
        let args = ~[~"--test=20", ~"--test=30"];
        let opts = ~[optmulti(~"test")];
        let rs = getopts(args, opts);
        match rs {
          Ok(ref m) => {
              assert (opt_present(m, ~"test"));
              assert (opt_str(m, ~"test") == ~"20");
              let pair = opt_strs(m, ~"test");
              assert (pair[0] == ~"20");
              assert (pair[1] == ~"30");
          }
          _ => die!()
        }
    }

    #[test]
    pub fn test_optmulti_short() {
        let args = ~[~"-t", ~"20"];
        let opts = ~[optmulti(~"t")];
        let rs = getopts(args, opts);
        match rs {
          Ok(ref m) => {
            assert (opt_present(m, ~"t"));
            assert (opt_str(m, ~"t") == ~"20");
          }
          _ => die!()
        }
    }

    #[test]
    pub fn test_optmulti_short_missing() {
        let args = ~[~"blah"];
        let opts = ~[optmulti(~"t")];
        let rs = getopts(args, opts);
        match rs {
          Ok(ref m) => assert (!opt_present(m, ~"t")),
          _ => die!()
        }
    }

    #[test]
    pub fn test_optmulti_short_no_arg() {
        let args = ~[~"-t"];
        let opts = ~[optmulti(~"t")];
        let rs = getopts(args, opts);
        match rs {
          Err(copy f) => check_fail_type(f, ArgumentMissing_),
          _ => die!()
        }
    }

    #[test]
    pub fn test_optmulti_short_multi() {
        let args = ~[~"-t", ~"20", ~"-t", ~"30"];
        let opts = ~[optmulti(~"t")];
        let rs = getopts(args, opts);
        match rs {
          Ok(ref m) => {
            assert (opt_present(m, ~"t"));
            assert (opt_str(m, ~"t") == ~"20");
            let pair = opt_strs(m, ~"t");
            assert (pair[0] == ~"20");
            assert (pair[1] == ~"30");
          }
          _ => die!()
        }
    }

    #[test]
    pub fn test_unrecognized_option_long() {
        let args = ~[~"--untest"];
        let opts = ~[optmulti(~"t")];
        let rs = getopts(args, opts);
        match rs {
          Err(copy f) => check_fail_type(f, UnrecognizedOption_),
          _ => die!()
        }
    }

    #[test]
    pub fn test_unrecognized_option_short() {
        let args = ~[~"-t"];
        let opts = ~[optmulti(~"test")];
        let rs = getopts(args, opts);
        match rs {
          Err(copy f) => check_fail_type(f, UnrecognizedOption_),
          _ => die!()
        }
    }

    #[test]
    pub fn test_combined() {
        let args =
            ~[~"prog", ~"free1", ~"-s", ~"20", ~"free2",
              ~"--flag", ~"--long=30", ~"-f", ~"-m", ~"40",
              ~"-m", ~"50", ~"-n", ~"-A B", ~"-n", ~"-60 70"];
        let opts =
            ~[optopt(~"s"), optflag(~"flag"), reqopt(~"long"),
             optflag(~"f"), optmulti(~"m"), optmulti(~"n"),
             optopt(~"notpresent")];
        let rs = getopts(args, opts);
        match rs {
          Ok(ref m) => {
            assert (m.free[0] == ~"prog");
            assert (m.free[1] == ~"free1");
            assert (opt_str(m, ~"s") == ~"20");
            assert (m.free[2] == ~"free2");
            assert (opt_present(m, ~"flag"));
            assert (opt_str(m, ~"long") == ~"30");
            assert (opt_present(m, ~"f"));
            let pair = opt_strs(m, ~"m");
            assert (pair[0] == ~"40");
            assert (pair[1] == ~"50");
            let pair = opt_strs(m, ~"n");
            assert (pair[0] == ~"-A B");
            assert (pair[1] == ~"-60 70");
            assert (!opt_present(m, ~"notpresent"));
          }
          _ => die!()
        }
    }

    #[test]
    pub fn test_multi() {
        let args = ~[~"-e", ~"foo", ~"--encrypt", ~"foo"];
        let opts = ~[optopt(~"e"), optopt(~"encrypt")];
        let matches = &match getopts(args, opts) {
          result::Ok(move m) => m,
          result::Err(_) => die!()
        };
        assert opts_present(matches, ~[~"e"]);
        assert opts_present(matches, ~[~"encrypt"]);
        assert opts_present(matches, ~[~"encrypt", ~"e"]);
        assert opts_present(matches, ~[~"e", ~"encrypt"]);
        assert !opts_present(matches, ~[~"thing"]);
        assert !opts_present(matches, ~[]);

        assert opts_str(matches, ~[~"e"]) == ~"foo";
        assert opts_str(matches, ~[~"encrypt"]) == ~"foo";
        assert opts_str(matches, ~[~"e", ~"encrypt"]) == ~"foo";
        assert opts_str(matches, ~[~"encrypt", ~"e"]) == ~"foo";
    }

    #[test]
    pub fn test_nospace() {
        let args = ~[~"-Lfoo", ~"-M."];
        let opts = ~[optmulti(~"L"), optmulti(~"M")];
        let matches = &match getopts(args, opts) {
          result::Ok(move m) => m,
          result::Err(_) => die!()
        };
        assert opts_present(matches, ~[~"L"]);
        assert opts_str(matches, ~[~"L"]) == ~"foo";
        assert opts_present(matches, ~[~"M"]);
        assert opts_str(matches, ~[~"M"]) == ~".";

    }

    #[test]
    pub fn test_groups_reqopt() {
        let opt = groups::reqopt(~"b", ~"banana", ~"some bananas", ~"VAL");
        assert opt == OptGroup { short_name: ~"b",
                        long_name: ~"banana",
                        hint: ~"VAL",
                        desc: ~"some bananas",
                        hasarg: Yes,
                        occur: Req }
    }

    #[test]
    pub fn test_groups_optopt() {
        let opt = groups::optopt(~"a", ~"apple", ~"some apples", ~"VAL");
        assert opt == OptGroup { short_name: ~"a",
                        long_name: ~"apple",
                        hint: ~"VAL",
                        desc: ~"some apples",
                        hasarg: Yes,
                        occur: Optional }
    }

    #[test]
    pub fn test_groups_optflag() {
        let opt = groups::optflag(~"k", ~"kiwi", ~"some kiwis");
        assert opt == OptGroup { short_name: ~"k",
                        long_name: ~"kiwi",
                        hint: ~"",
                        desc: ~"some kiwis",
                        hasarg: No,
                        occur: Optional }
    }

    #[test]
    pub fn test_groups_optflagopt() {
        let opt = groups::optflagopt(~"p", ~"pineapple",
                                       ~"some pineapples", ~"VAL");
        assert opt == OptGroup { short_name: ~"p",
                        long_name: ~"pineapple",
                        hint: ~"VAL",
                        desc: ~"some pineapples",
                        hasarg: Maybe,
                        occur: Optional }
    }

    #[test]
    pub fn test_groups_optmulti() {
        let opt = groups::optmulti(~"l", ~"lime",
                                     ~"some limes", ~"VAL");
        assert opt == OptGroup { short_name: ~"l",
                        long_name: ~"lime",
                        hint: ~"VAL",
                        desc: ~"some limes",
                        hasarg: Yes,
                        occur: Multi }
    }

    #[test]
    pub fn test_groups_long_to_short() {
        let short = ~[reqopt(~"b"), reqopt(~"banana")];
        let verbose = groups::reqopt(~"b", ~"banana",
                                       ~"some bananas", ~"VAL");

        assert groups::long_to_short(&verbose) == short;
    }

    #[test]
    pub fn test_groups_getopts() {
        let short = ~[
            reqopt(~"b"), reqopt(~"banana"),
            optopt(~"a"), optopt(~"apple"),
            optflag(~"k"), optflagopt(~"kiwi"),
            optflagopt(~"p"),
            optmulti(~"l")
        ];

        let verbose = ~[
            groups::reqopt(~"b", ~"banana", ~"Desc", ~"VAL"),
            groups::optopt(~"a", ~"apple", ~"Desc", ~"VAL"),
            groups::optflag(~"k", ~"kiwi", ~"Desc"),
            groups::optflagopt(~"p", ~"", ~"Desc", ~"VAL"),
            groups::optmulti(~"l", ~"", ~"Desc", ~"VAL"),
        ];

        let sample_args = ~[~"-k", ~"15", ~"--apple", ~"1", ~"k",
                            ~"-p", ~"16", ~"l", ~"35"];

        // FIXME #4681: sort options here?
        assert getopts(sample_args, short)
            == groups::getopts(sample_args, verbose);
    }

    #[test]
    pub fn test_groups_usage() {
        let optgroups = ~[
            groups::reqopt(~"b", ~"banana", ~"Desc", ~"VAL"),
            groups::optopt(~"a", ~"012345678901234567890123456789",
                             ~"Desc", ~"VAL"),
            groups::optflag(~"k", ~"kiwi", ~"Desc"),
            groups::optflagopt(~"p", ~"", ~"Desc", ~"VAL"),
            groups::optmulti(~"l", ~"", ~"Desc", ~"VAL"),
        ];

        let expected =
~"Usage: fruits

Options:
    -b --banana VAL     Desc
    -a --012345678901234567890123456789 VAL
                        Desc
    -k --kiwi           Desc
    -p [VAL]            Desc
    -l VAL              Desc

";

        let generated_usage = groups::usage(~"Usage: fruits", optgroups);

        debug!("expected: <<%s>>", expected);
        debug!("generated: <<%s>>", generated_usage);
        assert generated_usage == expected;
    }

    #[test]
    pub fn test_groups_usage_description_wrapping() {
        // indentation should be 24 spaces
        // lines wrap after 78: or rather descriptions wrap after 54

        let optgroups = ~[
           groups::optflag(~"k", ~"kiwi",
           ~"This is a long description which won't be wrapped..+.."), // 54
           groups::optflag(~"a", ~"apple",
           ~"This is a long description which _will_ be wrapped..+.."), // 55
        ];

        let expected =
~"Usage: fruits

Options:
    -k --kiwi           This is a long description which won't be wrapped..+..
    -a --apple          This is a long description which _will_ be
                        wrapped..+..

";

        let usage = groups::usage(~"Usage: fruits", optgroups);

        debug!("expected: <<%s>>", expected);
        debug!("generated: <<%s>>", usage);
        assert usage == expected
    }
}

// Local Variables:
// mode: rust;
// fill-column: 78;
// indent-tabs-mode: nil
// c-basic-offset: 4
// buffer-file-coding-system: utf-8-unix
// End:
