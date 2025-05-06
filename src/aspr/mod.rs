/*!

This module provides routines to make it easier to work with the ASPR synthetic population dataset. It provides basic 
parsing functionality for parsing the codes found in the dataset.  The `archive` submodule, enabled with the 
"aspr_archive" feature, additionally allows for reading ASPR data from CSV files in the dataset, including files within 
zipped archives.

This dataset encodes `homeId`, `schoolId`, and `workplaceId` using a FIPS geographic region code prefix. In particular, 
each row is a single entry for each person with:

1. **Age** as an integer by single year
2. **Home ID** as a 15-character string:
    - 11-digit tract + 4-digit within-tract sequential id
3. **School ID** as a 14-character string:
    - Public: 11-digit tract + 3-digit within-tract sequential id
    - Private: 5-digit county + “xprvx” + 4-digit within-county sequential id
4. **Work ID** as a 16-character string:
    - 11-digit tract + 5-digit within-tract sequential id

*/

use std::fmt::Display;
use crate::fips_code::FIPSCode;

// Re-exported publicly in `parser.rs`.
pub mod parser;
#[cfg(feature = "aspr_archive")]
pub mod archive;
pub mod errors;

/// A record representing a person in the ASPR synthetic population dataset
#[derive(Copy, Clone, Eq, PartialEq, Hash, Default, Debug)]
pub struct ASPRPersonRecord {
  pub age      : u8,
  pub home_id  : Option<FIPSCode>,
  pub school_id: Option<FIPSCode>,
  pub work_id  : Option<FIPSCode>,
}

impl Display for ASPRPersonRecord {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "Age: {}", self.age)?;

    if let Some(home) = &self.home_id {
      write!(f, ", Home: ({})", home)?;
    }
    if let Some(school) = &self.school_id {
      write!(f, ", School: ({})", school)?;
    }
    if let Some(work) = &self.work_id {
      write!(f, ", Work: ({})", work)?;
    }

    Ok(())
  }
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
  CensusTract,
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

impl Display for SettingCategory {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      SettingCategory::Unspecified   => write!(f, "Unspecified"),
      SettingCategory::Home          => write!(f, "Home"),
      SettingCategory::Workplace     => write!(f, "Workplace"),
      SettingCategory::PublicSchool  => write!(f, "Public School"),
      SettingCategory::PrivateSchool => write!(f, "Private School"),
      SettingCategory::CensusTract   => write!(f, "Census Tract"),
    }
  }
}
