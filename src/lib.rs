/*!

# FIPS Geographic Region Code Library

FIPS geographic region codes are used to represent hierarchical geographic regions from the state level down to the
"block" level. They are augmented in some synthetic population datasets with additional ID numbers for households,
workplaces, and schools. This library provides types to represent FIPS geographic region codes (and "code fragments"),
efficient representations, and utilities to convert to and from textual representations. It also provides data
structures and algorithms for searching.

# Terminology and Textual Representation

We use the phrase *census tract* (block, place, etc.) to refer to the full 11-digit encode, while the phrase *census
tract code* (resp. block code, place code, etc.) refers to the 6 digits for the tract designation itself. The digits of
more specific structures are generally the  rightmost decimal digits of the encoding. Thus the census tract code is the
rightmost 6 digits of the GeoId.

# FIPS Code Structure

Source: https://www.census.gov/programs-surveys/geography/guidance/geo-identifiers.html

| **Area Type**                              | **GEOID Structure**            | **Number of Digits** | **Example Geographic Area**                             | **Example GEOID** |
| ------------------------------------------ | ------------------------------ | -------------------- | ------------------------------------------------------- | ----------------- |
| State                                      | STATE                          | 2                    | Texas                                                   | 48                |
| County                                     | STATE+COUNTY                   | 2+3=5                | Harris County, TX                                       | 48201             |
| County Subdivision                         | STATE+COUNTY+COUSUB            | 2+3+5=10             | Pasadena CCD, Harris County, TX                         | 4820192975        |
| Census Tract                               | STATE+COUNTY+TRACT             | 2+3+6=11             | Census Tract 2231 in Harris County, TX                  | 48201223100       |
| Block Group                                | STATE+COUNTY+TRACT+BLOCK GROUP | 2+3+6+1=12           | Block Group 1 in Census Tract 2231 in Harris County, TX | 482012231001      |
| Block*                                     | STATE+COUNTY+TRACT+BLOCK       | 2+3+6+4=15           | Block 1050 in Census Tract 2231 in Harris County, TX    | 482012231001050   |
| Places                                     | STATE+PLACE                    | 2+5=7                | Houston, TX                                             | 4835000           |
| Congressional District (113th Congress)    | STATE+CD                       | 2+2=4                | Connecticut District 2                                  | 902               |
| State Legislative District (Upper Chamber) | STATE+SLDU                     | 2+3=5                | Connecticut State Senate District 33                    | 9033              |
| State Legislative District (Lower Chamber) | STATE+SLDL                     | 2+3=5                | Connecticut State House District 147                    | 9147              |
| ZCTA **                                    | ZCTA                           | 5                    | Suitland, MD ZCTA                                       | 20746             |

\* The block group code is not included in the census block GEOID code
because the first digit of a census block code represents the block group
code. Note – some blocks also contain a one character suffix (A, B, C, ect.)

\** ZIP Code Tabulation Areas (ZCTAs) are generalized areal representations
of United States Postal Service (USPS) ZIP Code service areas.


# Encoding Scheme

The rows in the table above up to and including Block (that is, all but the last
five rows) form a linear order with respect to prefix inclusion ("is prefix of").
This encoding scheme is for these codes. The last four rows are treated separately.

In the following table, we describe the data "fragments" and their storage requirements.

|                                   | **Decimal Digits** | **Actual Max Value** | **Bits** | **Capacity (2^bits - 1)**                                    |
| --------------------------------- | ------------------ | -------------------- | -------- | ------------------------------------------------------------ |
| **Sate**                          | 2                  | 56                   | 6        | 63                                                           |
| **County**                        | 3                  | 840                  | 10       | 1023                                                         |
| **Tract**                         | 6                  | 990101               | 20       | 1048575                                                      |
| **Subtotal**                      |                    |                      | **36**   | **Bits needed for tract code**                               |
|                                   |                    |                      |          |                                                              |
| **Monotonically Increasing Id's** |                    |                      |          |                                                              |
| **homeId**                        | 4                  | 9999                 | 14       | 16383                                                        |
| **publicschoolId**                | 3                  | 999                  | 10       | 1023                                                         |
| **privateschoolId**               | 4                  | 1722                 | 11       | 2047                                                         |
| **workplaceId**                   | 5                  | 14938                | 14       | 16383                                                        |
| **Max:**                          |                    |                      | **14**   |                                                              |
| **Total:**                        |                    |                      | **50**   |                                                              |

To the 50 bits apparently required to store this data we add an additional 4 bits for a category tag to distinguish
between home, public school, private school, and workplace, a field useful for representing ASPR synthetic population
data. Only 2 bits are required to distinguish these 4 categories, so the additional 2 bits are left unused / for future
use.

We encode this data into a `u64` as follows:

 | **Data**               |       **State** | **County** | **Tract** |    **Category Tag** | **Monotonically increasing ID number** | **Reserved / Unused** |
 | :--------------------- | --------------: | ---------: | --------: | ------------------: | -------------------------------------: | --------------------: |
 | **Bits**               |           63…58 |      57…48 |     47…28 |               27…24 |                                  23…10 |                   9…0 |
 | **Ex. Value**          | `AK`, `AZ`, ... |        258 |    223100 | `Home`, `Work`, ... |                                  12345 |                     0 |
 | **Bit Count**          |               6 |         10 |        20 |                   4 |                                     14 |                    10 |
 | **Capacity**           |              64 |       1024 |   1048576 |                  16 |                                  16384 |                  1024 |
 | **Decimal Digits**     |               2 |          3 |         6 |                   - |                                 3 to 5 |                     - |
 | **Max Observed Value** |              56 |        840 |    990101 |                   4 |                                  14938 |                     - |

Observe that:

 - We give the "category tag" 4 bits to allow up to 16 distinct categories. In some applications this field might be unused.
 - The least significant 10 bits is completely unused by this encoding. It may be used for application specific storage.
 - The field for ID number only requires 10 bits for `publicschoolId`, for example. That is, the storage it requires
   depends on the category tag.
 - The category tag is encoded after the tract code but before the ID field so that numerical ordering coincides with
   the hierarchical ordering.
 - Likewise, the unused 10 bits are the least significant bits so that numerical ordering coincides with the
   hierarchical ordering modulo those bits.

# Nonhierarchical FIPS Codes

The encoding of the previous section excludes the nonhierarchical codes of the last five rows from the first table
above:

 - Places
 - Congressional District (113th Congress)
 - State Legislative District (Upper Chamber)
 - State Legislative District (Lower Chamber)
 - ZCTA

We could easily accommodate these codes as well, in a variety of ways, e.g.:
 - assign each of these a category tag and store their corresponding code fragments in the ID field
 - use the 14 buts of the ID field and the unused 10 least significant bits, allowing the category tag to remain
   orthogonal

We leave them unspecified until we have a use case for them.

*/

#![allow(dead_code)]

mod aspr;
mod states;
mod parser;
mod fips_code;

// Convenience constants
const FOUR_BIT_MASK    : u8  = 15;      // 2^4-1
const SIX_BIT_MASK     : u8  = 63;      // 2^6-1
const TEN_BIT_MASK     : u16 = 1023;    // 2^10-1
const FOURTEEN_BIT_MASK: u16 = 16383;   // 2^14-1
const TWENTY_BIT_MASK  : u32 = 1048575; // 2^20-1

// Offsets of the bit fields in the encoded FIPS code
const STATE_OFFSET   : usize = 58;
const COUNTY_OFFSET  : usize = 48;
const TRACT_OFFSET   : usize = 28;
const CATEGORY_OFFSET: usize = 24;
const ID_OFFSET      : usize = 10;
// const DATA_OFFSET: usize = 0;

pub type CountyCode = u16;
pub type TractCode  = u32;
pub type IdCode     = u16;
pub type DataCode   = u16;


