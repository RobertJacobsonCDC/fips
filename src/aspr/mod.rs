/*!
This module is enabled with the "aspr" feature and provides routines to make it easier to work with the ASPR synthetic
population dataset. It provides basic parsing functionality for parsing the codes found in the dataset.  The `archive`
submodule, enabled with the "aspr_archive" feature, additionally allows for reading ASPR data from zipped archives.

This dataset encodes `homeId`, `schoolId`, and `workplaceId` using a FIPS geographic region code prefix. In particular,
each row is a single entry for each person with:

  i. Age as an integer by single year

  ii. Home ID as a 15-character string: 11-digit tract + 4-digit within-tract sequential id

  iii. School ID as a 14-character string:

    1. Public: 11-digit tract + 3-digit within-tract sequential id
    2. Private: 5-digit county + “xprvx” + 4-digit within-county sequential id

  iv. Work ID as a 16-character string: 11-digit tract + 5-digit within-tract sequential id

*/

use crate::FIPSCode;

// Re-exported publicly in `parser.rs`.
pub(crate) mod parser;
#[cfg(feature = "aspr_archive")]
pub mod archive;
pub mod errors;


pub struct ASPRPersonRecord {
  pub age      : u8,
  pub home_id  : Option<FIPSCode>,
  pub school_id: Option<FIPSCode>,
  pub work_id  : Option<FIPSCode>,
}

/// A `SettingCategory` is not a FIPS code but is implicit in the ASPR synthetic population dataset
#[derive(Copy, Clone, Eq, PartialEq, Hash, Default, Debug)]
#[repr(u8)]
pub enum SettingCategory {
  // We expect applications that do not use `SettingCategory` to have this field zeroed out.
  #[default]
  Unspecified = 0,
  Home,
  Workplace,
  PublicSchool,
  PrivateSchool,
}

impl SettingCategory {
  /// Decode a numeric value to a `SettingCategory`
  #[inline(always)]
  pub fn decode(value: u8) -> Option<Self> {
    if value <= 4 {
      Some(unsafe { std::mem::transmute(value) })
    } else {
      None
    }
  }

  /// Encode a `SettingCategory` as a `u8`
  #[inline(always)]
  pub fn encode(self) -> u8 {
    self as u8
  }
}
