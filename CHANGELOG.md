# Version 0.4
 * __BREAKING CHANGE:__ the `scan!` macro now strictly matches whitespace - this fixes a bunch of common errors, but could lead to breakage if you matched multiple whitespace characters as one in the past.

# Version 0.3.2
 * added support for the `#[serde(untagged)]` attribute on "parse-tree" style enums 

# Version 0.3
 * added basic `scan!` macro for working with custom formats
 * new function - `from_str_skip` - allows skipping custom characters, not just whitespace

# Version 0.2
 * added support for enums with tuple variants

# Version 0.1
 * initial release