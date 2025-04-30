use std::{
    cmp::Ordering,
    num::NonZero,
    fmt::{Display, Formatter}
};
use crate::{
    aspr::SettingCategory,
    CountyCode,
    DataCode,
    IdCode,
    TractCode,
    CATEGORY_OFFSET,
    COUNTY_OFFSET,
    FOURTEEN_BIT_MASK,
    FOUR_BIT_MASK,
    ID_OFFSET,
    SIX_BIT_MASK,
    STATE_OFFSET,
    TEN_BIT_MASK,
    TRACT_OFFSET,
    TWENTY_BIT_MASK,
    states::USState,
};


/// Encodes a hierarchical FIPS geographic region code in 64 bits. Excludes the nonhierarchical codes places,
/// congressional or state legislative districts, and ZIP code tabulation areas.
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct FIPSCode(NonZero<u64>);

impl FIPSCode {
    // region Constructors
    pub fn with_state(state: USState) -> Self {
        Self::new(state, 0,0, SettingCategory::default(),0,0)
    }
    pub fn with_county(state: USState, county: CountyCode) -> Self {
        Self::new(state, county,0, SettingCategory::default(),0,0)
    }
    pub fn with_tract(state: USState, county: CountyCode, tract: TractCode) -> Self {
        Self::new(state, county, tract, SettingCategory::default(),0,0)
    }
    pub fn with_category(state: USState, county: CountyCode, tract: TractCode, category: SettingCategory) -> Self {
        Self::new(state, county, tract, category,0,0)
    }

    pub fn new(
        state   : USState,
        county  : CountyCode,
        tract   : TractCode,
        category: SettingCategory,
        id      : IdCode,
        data    : DataCode
    ) -> Self {
        let encoded: u64 =
            Self::encode_state(state.encode())
            | Self::encode_county(county)
            | Self::encode_tract(tract)
            | Self::encode_category(category.encode())
            | Self::encode_id(id)
            | Self::encode_data(data);
        // At the very least, `USState.encode()` will return a non-zero value, so this unwrapping is safe.
        let encoded = NonZero::new(encoded).unwrap();
        Self(encoded)
    }
    // endregion Constructors

    // region Accessors

    /// Returns the FIPS STATE as a `USState` enum variant.
    #[inline(always)]
    pub fn state(&self) -> USState {
        // We are guaranteed to have a valid state code if this `FIPSCode` was constructed safely
        unsafe{ USState::decode(self.state_code()).unwrap_unchecked() }
    }

    /// Returns the FIPS STATE code as a `u8`
    #[inline(always)]
    pub fn state_code(&self) -> u8 {
        // The state code occupies the 6 most significant bits, bits 58..63
        (self.0.get() >> STATE_OFFSET) as u8
    }

    /// Returns the numeric FIPS COUNTY code
    #[inline(always)]
    pub fn county_code(&self) -> u16 {
        // The county code occupies the 10 bits from bits 48..57
        ((self.0.get() >> COUNTY_OFFSET) as u16) & TEN_BIT_MASK
    }

    /// Returns the numeric FIPS CENSUS TRACT code
    #[inline(always)]
    pub fn census_tract_code(&self) -> u32 {
        // The census tract code occupies the 20 bits from bits 28..47
        ((self.0.get() >> TRACT_OFFSET) as u32) & TWENTY_BIT_MASK
    }

    /// Returns the setting category code as a `u18`
    #[inline(always)]
    pub fn category_code(&self) -> u8 {
        // The category code occupies the 4 bits from bits 24..27
        ((self.0.get() >> CATEGORY_OFFSET) as u8) & FOUR_BIT_MASK
    }

    /// Returns the setting category as a `SettingCategory`
    #[inline(always)]
    pub fn category(&self) -> SettingCategory {
        // We are guaranteed to have a valid SettingCategory if this `FIPSCode` was constructed safely
        unsafe{ SettingCategory::decode(self.category_code()).unwrap_unchecked() }
    }

    /// Returns the monotonically increasing ID number as a `u16`
    #[inline(always)]
    pub fn id(&self) -> u16 {
        // The ID number occupies the 14 bits from bits 10..23
        ((self.0.get() >> ID_OFFSET) as u16) & FOURTEEN_BIT_MASK
    }

    /// Returns the unused data region occupying the 10 LSB
    #[inline(always)]
    pub fn data(&self) -> u16 {
        self.0.get() as u16 & TEN_BIT_MASK
    }
    // endregion Accessors

    /// Sets the unused data region occupying the 10 LSB
    #[inline(always)]
    pub fn set_data(&mut self, data: u16) {
        assert!(data <= TEN_BIT_MASK);
        let inverse_mask = !(TEN_BIT_MASK as u64);
        self.0 = unsafe{
            NonZero::new(
                (self.0.get() & inverse_mask) | ((data & TEN_BIT_MASK) as u64)
            ).unwrap_unchecked()
        };
    }


    /// Compares the given values without respect to the data region (the Least Significant Bits)
    #[inline(always)]
    pub fn compare_non_data(&self, other: Self) -> Ordering{
        let inverse_mask = !(TEN_BIT_MASK as u64);
        let this         = self.0.get()  & inverse_mask;
        let other        = other.0.get() & inverse_mask;

        this.cmp(&other)
    }

    // region Encoding
    // It is convenient to factor out the encode operations into their own functions.
    // These functions take numeric values and return encoded `u64` values. To encode
    // enum variants, call the `encode` function on the enum variant.

    #[inline(always)]
    fn encode_state(state: u8) -> u64 {
        // Validate
        assert!(USState::valid_code(state));
        // Only 6 bits are available for the state code.
        assert!(state <= SIX_BIT_MASK);
        (state as u64) << STATE_OFFSET

    }

    #[inline(always)]
    fn encode_county(county: u16) -> u64 {
        // Validate
        assert!(county <= TEN_BIT_MASK);
        (county as u64) << COUNTY_OFFSET
    }

    #[inline(always)]
    fn encode_tract(tract: u32) -> u64 {
        // Validate
        assert!(tract <= TWENTY_BIT_MASK);
        (tract as u64) << TRACT_OFFSET
    }

    #[inline(always)]
    fn encode_category(setting_category: u8) -> u64 {
        // Validate
        assert!(setting_category <= FOUR_BIT_MASK);
        (setting_category as u64) << CATEGORY_OFFSET
    }

    #[inline(always)]
    fn encode_id(id: u16) -> u64 {
        // Validate
        assert!(id <= FOURTEEN_BIT_MASK);
        (id as u64) << ID_OFFSET
    }

    #[inline(always)]
    fn encode_data(data: u16) -> u64 {
        // Validate
        assert!(data <= TEN_BIT_MASK);
        data as u64
    }
    // endregion Encoding
}

impl Display for FIPSCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", ExpandedFIPSCode::from_fips_code(*self))
    }
}

pub struct ExpandedFIPSCode {
    pub state   : USState,
    pub county  : CountyCode,
    pub tract   : TractCode,
    pub category: SettingCategory,
    pub id      : IdCode,
    pub data    : DataCode,
}

impl ExpandedFIPSCode {
    pub fn from_fips_code(fips_code: FIPSCode) -> Self {
        Self {
            state   : fips_code.state(),
            county  : fips_code.county_code(),
            tract   : fips_code.census_tract_code(),
            category: fips_code.category(),
            id      : fips_code.id(),
            data    : fips_code.data()
        }
    }

    pub fn to_fips_code(&self) -> FIPSCode {
        FIPSCode::new(
            self.state,
            self.county,
            self.tract,
            self.category,
            self.id,
            self.data
        )
    }
}

impl Display for ExpandedFIPSCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "state: {}", self.state)?;
        
        if self.county != 0 {
            write!(f, ", county: {}", self.county)?;
        }
        if self.tract != 0 {
            write!(f, ", tract: {}", self.tract)?;
        }
        if self.category != SettingCategory::Unspecified {
            write!(f, ", setting: {}", self.category)?;
        }
        if self.id != 0 {
            write!(f, ", id: {}", self.id)?;
        }
        if self.data != 0 {
            write!(f, ", data field: {}", self.data)?;
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
    fn fields_round_trip() {
        let fips_code = FIPSCode::new(
            USState::TX,
            123,
            990101,
            SettingCategory::Home,
            14938,
            0x3ff
        );
        assert_eq!(fips_code.state(), USState::TX);
        assert_eq!(fips_code.county_code(), 123);
        assert_eq!(fips_code.census_tract_code(), 990101);
        assert_eq!(fips_code.category(), SettingCategory::Home);
        assert_eq!(fips_code.id(), 14938);
        assert_eq!(fips_code.data(), 0x3ff);
    }

    #[test]
    fn nonstate_round_trip() {
        let fips_code = FIPSCode::with_state(USState::VirginIslandsOfTheUS);
        assert_eq!(fips_code.state(), USState::VirginIslandsOfTheUS);
        assert_eq!(fips_code.state_code(), 52);

        let fips_code = FIPSCode::with_state(USState::HawaiianCoast);
        assert_eq!(fips_code.state(), USState::HawaiianCoast);
        assert_eq!(fips_code.state_code(), 59);
    }

    #[test]
    fn expanded_round_trip() {
        let fips_code = FIPSCode::new(
            USState::TX,
            123,
            990101,
            SettingCategory::Home,
            14938,
            0x01ff
        );
        let expanded = ExpandedFIPSCode::from_fips_code(fips_code);
        assert_eq!(expanded.to_fips_code(), fips_code);
    }

    #[test]
    fn test_compare_non_data() {
        let fips_code_a = FIPSCode::new(
            USState::TX,
            123,
            990101,
            SettingCategory::Home,
            14938,
            0x01ff
        );
        let fips_code_b = FIPSCode::new(
            USState::TX,
            123,
            990101,
            SettingCategory::Home,
            14938,
            0x00ff
        );

        assert_eq!(fips_code_a.compare_non_data(fips_code_b), Ordering::Equal);
        assert_eq!(fips_code_a.cmp(&fips_code_b), Ordering::Greater);
    }

}