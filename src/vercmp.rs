/*
 *  vercmp.rs - Compare package version numbers using pacman's version
 *      comparison logic
 *
 *  Copyright (c) 2006-2018 Pacman Development Team <pacman-dev@archlinux.org>
 *  Copyright (c) 2002-2005 by Judd Vinet <jvinet@zeroflux.org>
 *
 *  Ported by Jay Robson 2025
 *
 *  This program is free software; you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation; either version 2 of the License, or
 *  (at your option) any later version.
 *
 *  This program is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License
 *  along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

use std::cmp::Ordering;

/// Split EVR into epoch, version, and release components.
/// 
/// # Parameters
/// - `evr`: [epoch:]version[-release] string
/// 
/// # Returns
/// - A tuple containing:
///   - `&str`: reference to epoch
///   - `&str`: reference to version
///   - `Option<&str>`: reference to release
pub fn parse_evr(evr: &str) -> (&str, &str, Option<&str>) {

	// points to epoch terminator
	let epoch_end = evr.find(|c: char| !c.is_ascii_digit()).unwrap_or(evr.len());
	let (mut epoch, mut version) = evr.split_at(epoch_end);

	if version.starts_with(':') {
		version = &version[1..];
	} else {
		// different from RPM- always assume 0 epoch
		epoch = "0";
		version = evr;
	}
	if let Some((version, release)) = version.split_once('-') {
		(epoch, version, Some(release))
	} else {
		(epoch, version, None)
	}
}


/// Compare alpha and numeric segments of two versions.
/// 
/// # Parameters
/// - `a`: The first version string to compare.
/// - `b`: The second version string to compare.
/// 
/// # Returns
/// - `Ordering::Greater`: if `a` is newer than `b`
/// - `Ordering::Less`: if `b` is newer than `a`
/// - `Ordering::Equal`: if `a` and `b` are the same version
pub fn rpm_ver_cmp(mut one: &str, mut two: &str) -> Ordering {

	// easy comparison to see if versions are identical
	if one == two {
		return Ordering::Equal;
	}

	let is_alnum = |ch: char| ch.is_alphanumeric();
	let is_not_alnum = |ch: char| !ch.is_alphanumeric();
	let is_not_digit = |ch: char| !ch.is_ascii_digit();
	
	// loop through each version segment of str1 and str2 and compare them
	while one.len() > 0 && two.len() > 0 {

		// If we ran to the end of either, we are finished with the loop
		let (Some(mid_a), Some(mid_b)) = (one.find(is_alnum), two.find(is_alnum)) else {
			break;
		};

		// If the separator lengths were different, we are also finished
		match usize::cmp(&mid_a, &mid_b) {
			Ordering::Equal => (),
			v => return v,
		}

		one = &one[mid_a..];
		two = &two[mid_b..];

		// grab first completely alpha or completely numeric segment
		let (mid_a, mid_b, is_num) = if one.chars().next().unwrap().is_ascii_digit() {
			(one.find(is_not_digit).unwrap_or(one.len()), two.find(is_not_digit).unwrap_or(two.len()), true)
		} else {
			(one.find(is_not_alnum).unwrap_or(one.len()), two.find(is_not_alnum).unwrap_or(two.len()), false)
		};

		let (start_a, end_a) = one.split_at(mid_a);
		let (start_b, end_b) = two.split_at(mid_b);
		one = start_a;
		two = start_b;
		
		// take care of the case where the two version segments are of different types
		if two.is_empty() {
			return match is_num { true => Ordering::Greater, false => Ordering::Less };
		}

		if is_num {
			// throw away any leading zeros - it's a number, right?
			one = one.trim_start_matches('0');
			two = two.trim_start_matches('0');

			// whichever number has more digits wins
			match usize::cmp(&one.len(), &two.len()) {
				Ordering::Equal => (),
				v => return v,
			}
		}

		// cmp will return which one is greater - if one is greater
		match one.cmp(two) {
			Ordering::Equal => (),
			v => return v,
		}
		
		// move on to the next
		one = end_a;
		two = end_b;
	}

	// this catches the case where all numeric and alpha segments have
	// compared identically but the segment separating characters were
	// different
	if one.is_empty() && two.is_empty() {
		return Ordering::Equal;
	}

	// the final showdown. we never want a remaining alpha string to
	// beat an empty string. the logic is a bit weird, but:
	// - if two is not an alpha, two is newer.
	// - if one is an alpha, two is newer.
	// - otherwise one is newer.
	if two.chars().next().map_or(false, is_not_alnum) || one.chars().next().map_or(false, is_alnum) {
		Ordering::Less
	} else {
		Ordering::Greater
	}
}

/// Compare two version strings and determine which one is 'newer'.
/// 
/// # Parameters
/// - `a`: The first version string to compare.
/// - `b`: The second version string to compare.
/// 
/// # Returns
/// - `Ordering::Greater`: if `a` is newer than `b`
/// - `Ordering::Less`: if `b` is newer than `a`
/// - `Ordering::Equal`: if `a` and `b` are the same version
/// 
/// Different epoch values for version strings will override any further
/// comparison. If no epoch is provided, `0` is assumed.
/// 
/// Keep in mind that the release component (pkgrel) is only compared
/// if it is available on both versions handed to this function. For
/// example, comparing `1.5-1` and `1.5` will yield `Ordering::Equal`;
/// comparing `1.5-1` and `1.5-2` will yield `Ordering::Less` as expected.
/// This is mainly for supporting versioned dependencies that do not
/// include the pkgrel.
pub fn alpm_pkg_ver_cmp(a: &str, b: &str) -> Ordering {

	// ensure our strings are not empty
	match (a.is_empty(), b.is_empty()) {
		(true, true) => return Ordering::Equal,
		(true, false) => return Ordering::Less,
		(false, true) => return Ordering::Greater,
		_ => (),
	}

	// another quick shortcut- if full version specs are equal
	if a == b {
		return Ordering::Equal;
	}

	// Parse both versions into [epoch:]version[-release] triplets. We probably
	// don't need epoch and release to support all the same magic, but it is
	// easier to just run it all through the same code.
	let (epoch1, ver1, rel1) = parse_evr(a);
	let (epoch2, ver2, rel2) = parse_evr(b);

	match rpm_ver_cmp(epoch1, epoch2) {
		Ordering::Equal => (),
		v => return v,
	}

	match rpm_ver_cmp(ver1, ver2) {
		Ordering::Equal => (),
		v => return v,
	}

	if let (Some(rel1), Some(rel2)) = (rel1, rel2) {
		rpm_ver_cmp(rel1, rel2)
	} else {
		Ordering::Equal
	}
}

