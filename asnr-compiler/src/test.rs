use asnr_grammar::*;
use asnr_transcoder::{error::{DecodingError, DecodingErrorType}, Decode};


/* * 
 * This DE indicates a change of acceleration.
 *
 * The value shall be set to:
 * - 0 - `accelerate` - if the magnitude of the horizontal velocity vector increases.
 * - 1 - `decelerate` - if the magnitude of the horizontal velocity vector decreases.
 *
 * @category: Kinematic information
 * @revision: Created in V2.1.1
*/
#[derive(Debug, Clone, PartialEq)]
pub enum AccelerationChange {
  accelerate = 0,
	decelerate = 1,
}

impl TryFrom<i128> for AccelerationChange {
    type Error = DecodingError;

    fn try_from(v: i128) -> Result<Self, Self::Error> {
        match v {
            x if x == Self::accelerate as i128 => Ok(Self::accelerate),
		  x if x == Self::decelerate as i128 => Ok(Self::decelerate),
            _ => Err(
              DecodingError::new(
                &format!("Invalid enumerated index decoding AccelerationChange. Received index {}",v), DecodingErrorType::InvalidEnumeratedIndex
              )
            ),
        }
    }
}

impl Decode for AccelerationChange {
    fn decode<'a, D>(decoder: D, input: &'a [u8]) -> nom::IResult<&'a [u8], Self>
    where
        D: asnr_transcoder::Decoder,
        Self: Sized,
    {
        decoder.decode_enumerated(
          AsnEnumerated { members: vec![Enumeral { name: "accelerate".into(), description: None, index: 0 },Enumeral { name: "decelerate".into(), description: None, index: 1 }], extensible: false }, 
          input
        )
    }
}


/* *
 * This DE indicates the acceleration confidence value which represents the estimated absolute accuracy of an acceleration value with a default confidence level of 95 %. 
 * If required, the confidence level can be defined by the corresponding standards applying this DE.
 *
 * The value shall be set to:
 * - `n` (`n > 0` and `n < 101`) if the confidence value is equal to or less than n x 0,1 m/s^2, and greater than (n-1) x 0,1 m/s^2,
 * - `101` if the confidence value is out of range i.e. greater than 10 m/s^2,
 * - `102` if the confidence value is unavailable.
 *
 * The value 0 shall not be used.
 *
 * @note: The fact that an acceleration value is received with confidence value set to `unavailable(102)` can be caused by several reasons, such as:
 * - the sensor cannot deliver the accuracy at the defined confidence level because it is a low-end sensor,
 * - the sensor cannot calculate the accuracy due to lack of variables, or
 * - there has been a vehicle bus (e.g. CAN bus) error.
 * In all 3 cases above, the acceleration value may be valid and used by the application.
 * 
 * @note: If an acceleration value is received and its confidence value is set to `outOfRange(101)`, it means that the value is not valid and therefore cannot be trusted. Such value is not useful for the application.
 *
 * @unit 0,1 m/s^2
 * @category: Kinematic information
 * @revision: Description revised in V2.1.1
 */
#[derive(Debug, Clone, PartialEq)]
pub struct AccelerationConfidence(pub u8);

impl Decode for AccelerationConfidence {
    fn decode<'a, D>(decoder: D, input: &'a [u8]) -> nom::IResult<&'a [u8], Self>
    where
        D: asnr_transcoder::Decoder,
        Self: Sized,
    {
        decoder
            .decode_integer(AsnInteger { constraint: Some(Constraint { min_value: Some(0), max_value: Some(102), extensible: false }), distinguished_values: Some(vec![DistinguishedValue { name: "outOfRange".into(), value: 101 },DistinguishedValue { name: "unavailable".into(), value: 102 }]) }, input)
            .map(|(remaining, res)| (remaining, Self(res)))
    }
}


/* * 
 * This DE represents the magnitude of the acceleration vector in a defined coordinate system.
 *
 * The value shall be set to:
 * - `0` to indicate no acceleration,
 * - `n` (`n > 0` and `n < 160`) to indicate acceleration equal to or less than n x 0,1 m/s^2, and greater than (n-1) x 0,1 m/s^2,
 * - `160` for acceleration values greater than 15,9 m/s^2,
 * - `161` when the data is unavailable.
 *
 * @unit 0,1 m/s^2
 * @category: Kinematic information
 * @revision: Created in V2.1.1
*/
#[derive(Debug, Clone, PartialEq)]
pub struct AccelerationMagnitudeValue(pub u8);

impl Decode for AccelerationMagnitudeValue {
    fn decode<'a, D>(decoder: D, input: &'a [u8]) -> nom::IResult<&'a [u8], Self>
    where
        D: asnr_transcoder::Decoder,
        Self: Sized,
    {
        decoder
            .decode_integer(AsnInteger { constraint: Some(Constraint { min_value: Some(0), max_value: Some(161), extensible: false }), distinguished_values: Some(vec![DistinguishedValue { name: "positiveOutOfRange".into(), value: 160 },DistinguishedValue { name: "unavailable".into(), value: 161 }]) }, input)
            .map(|(remaining, res)| (remaining, Self(res)))
    }
}


/* * 
 * This DE represents the value of an acceleration component in a defined coordinate system.
 *
 * The value shall be set to:
 * - `-160` for acceleration values equal to or less than -16 m/s^2,
 * - `n` (`n > -160` and `n <= 0`) to indicate negative acceleration equal to or less than n x 0,1 m/s^2, and greater than (n-1) x 0,1 m/s^2,
 * - `n` (`n > 0` and `n < 160`) to indicate positive acceleration equal to or less than n x 0,1 m/s^2, and greater than (n-1) x 0,1 m/s^2,
 * - `160` for acceleration values greater than 15,9 m/s^2,
 * - `161` when the data is unavailable.
 *
 * @note: the formula for values > -160 and <160 results in rounding up to the next value. Zero acceleration is indicated using n=0.
 * @unit 0,1 m/s^2
 * @category: Kinematic information
 * @revision: Created in V2.1.1
*/
#[derive(Debug, Clone, PartialEq)]
pub struct AccelerationValue(pub i16);

impl Decode for AccelerationValue {
    fn decode<'a, D>(decoder: D, input: &'a [u8]) -> nom::IResult<&'a [u8], Self>
    where
        D: asnr_transcoder::Decoder,
        Self: Sized,
    {
        decoder
            .decode_integer(AsnInteger { constraint: Some(Constraint { min_value: Some(-160), max_value: Some(161), extensible: false }), distinguished_values: Some(vec![DistinguishedValue { name: "negativeOutOfRange".into(), value: -160 },DistinguishedValue { name: "positiveOutOfRange".into(), value: 160 },DistinguishedValue { name: "unavailable".into(), value: 161 }]) }, input)
            .map(|(remaining, res)| (remaining, Self(res)))
    }
}


/* *
 * This DE indicates an access technology.
 *
 * The value shall be set to:
 * - `0`: in case of any access technology class,
 * - `1`: in case of ITS-G5 access technology class,
 * - `2`: in case of LTE-V2X access technology class,
 * - `3`: in case of NR-V2X access technology class.
 * 
 * @category: Communication information
 * @revision: Created in V2.1.1
 */
#[derive(Debug, Clone, PartialEq)]
pub enum AccessTechnologyClass {
  any = 0,
	itsg5Class = 1,
	ltev2xClass = 2,
	nrv2xClass = 3,
}

impl TryFrom<i128> for AccessTechnologyClass {
    type Error = DecodingError;

    fn try_from(v: i128) -> Result<Self, Self::Error> {
        match v {
            x if x == Self::any as i128 => Ok(Self::any),
		  x if x == Self::itsg5Class as i128 => Ok(Self::itsg5Class),
		  x if x == Self::ltev2xClass as i128 => Ok(Self::ltev2xClass),
		  x if x == Self::nrv2xClass as i128 => Ok(Self::nrv2xClass),
            _ => Err(
              DecodingError::new(
                &format!("Invalid enumerated index decoding AccessTechnologyClass. Received index {}",v), DecodingErrorType::InvalidEnumeratedIndex
              )
            ),
        }
    }
}

impl Decode for AccessTechnologyClass {
    fn decode<'a, D>(decoder: D, input: &'a [u8]) -> nom::IResult<&'a [u8], Self>
    where
        D: asnr_transcoder::Decoder,
        Self: Sized,
    {
        decoder.decode_enumerated(
          AsnEnumerated { members: vec![Enumeral { name: "any".into(), description: None, index: 0 },Enumeral { name: "itsg5Class".into(), description: None, index: 1 },Enumeral { name: "ltev2xClass".into(), description: None, index: 2 },Enumeral { name: "nrv2xClass".into(), description: None, index: 3 }], extensible: true }, 
          input
        )
    }
}


/* *
 * This DE represents the value of the sub cause code of the @ref CauseCode `accident`.
 *
 * The value shall be set to:
 * - 0 - `unavailable`                        - in case the information on the sub cause of the accident is unavailable,
 * - 1 - `multiVehicleAccident`               - in case more than two vehicles are involved in accident,
 * - 2 - `heavyAccident`                      - in case the airbag of the vehicle involved in the accident is triggered, 
 *                                              the accident requires important rescue and/or recovery work,
 * - 3 - `accidentInvolvingLorry`             - in case the accident involves a lorry,
 * - 4 - `accidentInvolvingBus`               - in case the accident involves a bus,
 * - 5 - `accidentInvolvingHazardousMaterials`- in case the accident involves hazardous material,
 * - 6 - `accidentOnOppositeLane`             - in case the accident happens on opposite lanes,
 * - 7 - `unsecuredAccident`                  - in case the accident is not secured,
 * - 8 - `assistanceRequested`                - in case rescue and assistance are requested,
 * - 9-255                                    - reserved for future usage. 
 *
 * @category: Traffic information
 * @revision: V1.3.1
 */
#[derive(Debug, Clone, PartialEq)]
pub struct AccidentSubCauseCode(pub u8);

impl Decode for AccidentSubCauseCode {
    fn decode<'a, D>(decoder: D, input: &'a [u8]) -> nom::IResult<&'a [u8], Self>
    where
        D: asnr_transcoder::Decoder,
        Self: Sized,
    {
        decoder
            .decode_integer(AsnInteger { constraint: Some(Constraint { min_value: Some(0), max_value: Some(255), extensible: false }), distinguished_values: Some(vec![DistinguishedValue { name: "unavailable".into(), value: 0 },DistinguishedValue { name: "multiVehicleAccident".into(), value: 1 },DistinguishedValue { name: "heavyAccident".into(), value: 2 },DistinguishedValue { name: "accidentInvolvingLorry".into(), value: 3 },DistinguishedValue { name: "accidentInvolvingBus".into(), value: 4 },DistinguishedValue { name: "accidentInvolvingHazardousMaterials".into(), value: 5 },DistinguishedValue { name: "accidentOnOppositeLane".into(), value: 6 },DistinguishedValue { name: "unsecuredAccident".into(), value: 7 },DistinguishedValue { name: "assistanceRequested".into(), value: 8 }]) }, input)
            .map(|(remaining, res)| (remaining, Self(res)))
    }
}


/* *
 * This DE represents the value of the sub cause code of the @ref CauseCode `adverseWeatherCondition-Adhesion`. 
 * 
 * The value shall be set to:
 * - 0 - `unavailable`     - in case information on the cause of the low road adhesion is unavailable,
 * - 1 - `heavyFrostOnRoad`- in case the low road adhesion is due to heavy frost on the road,
 * - 2 - `fuelOnRoad`      - in case the low road adhesion is due to fuel on the road,
 * - 3 - `mudOnRoad`       - in case the low road adhesion is due to mud on the road,
 * - 4 - `snowOnRoad`      - in case the low road adhesion is due to snow on the road,
 * - 5 - `iceOnRoad`       - in case the low road adhesion is due to ice on the road,
 * - 6 - `blackIceOnRoad`  - in case the low road adhesion is due to black ice on the road,
 * - 7 - `oilOnRoad`       - in case the low road adhesion is due to oil on the road,
 * - 8 - `looseChippings`  - in case the low road adhesion is due to loose gravel or stone fragments detached from a road surface or from a hazard,
 * - 9 - `instantBlackIce` - in case the low road adhesion is due to instant black ice on the road surface,
 * - 10 - `roadsSalted`    - when the low road adhesion is due to salted road,
 * - 11-255                - are reserved for future usage.
 *
 * @category: Traffic information
 * @revision: V1.3.1
 */
#[derive(Debug, Clone, PartialEq)]
pub struct AdverseWeatherCondition_AdhesionSubCauseCode(pub u8);

impl Decode for AdverseWeatherCondition_AdhesionSubCauseCode {
    fn decode<'a, D>(decoder: D, input: &'a [u8]) -> nom::IResult<&'a [u8], Self>
    where
        D: asnr_transcoder::Decoder,
        Self: Sized,
    {
        decoder
            .decode_integer(AsnInteger { constraint: Some(Constraint { min_value: Some(0), max_value: Some(255), extensible: false }), distinguished_values: Some(vec![DistinguishedValue { name: "unavailable".into(), value: 0 },DistinguishedValue { name: "heavyFrostOnRoad".into(), value: 1 },DistinguishedValue { name: "fuelOnRoad".into(), value: 2 },DistinguishedValue { name: "mudOnRoad".into(), value: 3 },DistinguishedValue { name: "snowOnRoad".into(), value: 4 },DistinguishedValue { name: "iceOnRoad".into(), value: 5 },DistinguishedValue { name: "blackIceOnRoad".into(), value: 6 },DistinguishedValue { name: "oilOnRoad".into(), value: 7 },DistinguishedValue { name: "looseChippings".into(), value: 8 },DistinguishedValue { name: "instantBlackIce".into(), value: 9 },DistinguishedValue { name: "roadsSalted".into(), value: 10 }]) }, input)
            .map(|(remaining, res)| (remaining, Self(res)))
    }
}


/* *
 * This DE represents the value of the sub cause codes of the @ref CauseCode `adverseWeatherCondition-ExtremeWeatherCondition`.
 *
 * The value shall be set to:
 * - 0 - `unavailable` - in case information on the type of extreme weather condition is unavailable,
 * - 1 - `strongWinds` - in case the type of extreme weather condition is strong wind,
 * - 2 - `damagingHail`- in case the type of extreme weather condition is damaging hail,
 * - 3 - `hurricane`   - in case the type of extreme weather condition is hurricane,
 * - 4 - `thunderstorm`- in case the type of extreme weather condition is thunderstorm,
 * - 5 - `tornado`     - in case the type of extreme weather condition is tornado,
 * - 6 - `blizzard`    - in case the type of extreme weather condition is blizzard.
 * - 7-255             - are reserved for future usage.
 *
 * @category: Traffic information
 * @revision: V1.3.1
 */
#[derive(Debug, Clone, PartialEq)]
pub struct AdverseWeatherCondition_ExtremeWeatherConditionSubCauseCode(pub u8);

impl Decode for AdverseWeatherCondition_ExtremeWeatherConditionSubCauseCode {
    fn decode<'a, D>(decoder: D, input: &'a [u8]) -> nom::IResult<&'a [u8], Self>
    where
        D: asnr_transcoder::Decoder,
        Self: Sized,
    {
        decoder
            .decode_integer(AsnInteger { constraint: Some(Constraint { min_value: Some(0), max_value: Some(255), extensible: false }), distinguished_values: Some(vec![DistinguishedValue { name: "unavailable".into(), value: 0 },DistinguishedValue { name: "strongWinds".into(), value: 1 },DistinguishedValue { name: "damagingHail".into(), value: 2 },DistinguishedValue { name: "hurricane".into(), value: 3 },DistinguishedValue { name: "thunderstorm".into(), value: 4 },DistinguishedValue { name: "tornado".into(), value: 5 },DistinguishedValue { name: "blizzard".into(), value: 6 }]) }, input)
            .map(|(remaining, res)| (remaining, Self(res)))
    }
}


/* *
 * This DE represents the value of the sub cause codes of the @ref CauseCode `adverseWeatherCondition-Precipitation`. 
 *
 * The value shall be set to:
 * - 0 - `unavailable`   - in case information on the type of precipitation is unavailable,
 * - 1 - `heavyRain`     - in case the type of precipitation is heavy rain,
 * - 2 - `heavySnowfall` - in case the type of precipitation is heavy snow fall,
 * - 3 - `softHail`      - in case the type of precipitation is soft hail.
 * - 4-255               - are reserved for future usage
 *
 * @category: Traffic information
 * @revision: V1.3.1
 */
#[derive(Debug, Clone, PartialEq)]
pub struct AdverseWeatherCondition_PrecipitationSubCauseCode(pub u8);

impl Decode for AdverseWeatherCondition_PrecipitationSubCauseCode {
    fn decode<'a, D>(decoder: D, input: &'a [u8]) -> nom::IResult<&'a [u8], Self>
    where
        D: asnr_transcoder::Decoder,
        Self: Sized,
    {
        decoder
            .decode_integer(AsnInteger { constraint: Some(Constraint { min_value: Some(0), max_value: Some(255), extensible: false }), distinguished_values: Some(vec![DistinguishedValue { name: "unavailable".into(), value: 0 },DistinguishedValue { name: "heavyRain".into(), value: 1 },DistinguishedValue { name: "heavySnowfall".into(), value: 2 },DistinguishedValue { name: "softHail".into(), value: 3 }]) }, input)
            .map(|(remaining, res)| (remaining, Self(res)))
    }
}


/* *
 * This DE represents the value of the sub cause codes of the @ref CauseCode `adverseWeatherCondition-Visibility`.
 *
 * The value shall be set to:
 * - 0 - `unavailable`    - in case information on the cause of low visibility is unavailable,
 * - 1 - `fog`            - in case the cause of low visibility is fog,
 * - 2 - `smoke`          - in case the cause of low visibility is smoke,
 * - 3 - `heavySnowfall`  - in case the cause of low visibility is heavy snow fall,
 * - 4 - `heavyRain`      - in case the cause of low visibility is heavy rain,
 * - 5 - `heavyHail`      - in case the cause of low visibility is heavy hail,
 * - 6 - `lowSunGlare`    - in case the cause of low visibility is sun glare,
 * - 7 - `sandstorms`     - in case the cause of low visibility is sand storm,
 * - 8 - `swarmsOfInsects`- in case the cause of low visibility is swarm of insects.
 * - 9-255                - are reserved for future usage
 *
 * @category: Traffic information
 * @revision: V1.3.1
 */
#[derive(Debug, Clone, PartialEq)]
pub struct AdverseWeatherCondition_VisibilitySubCauseCode(pub u8);

impl Decode for AdverseWeatherCondition_VisibilitySubCauseCode {
    fn decode<'a, D>(decoder: D, input: &'a [u8]) -> nom::IResult<&'a [u8], Self>
    where
        D: asnr_transcoder::Decoder,
        Self: Sized,
    {
        decoder
            .decode_integer(AsnInteger { constraint: Some(Constraint { min_value: Some(0), max_value: Some(255), extensible: false }), distinguished_values: Some(vec![DistinguishedValue { name: "unavailable".into(), value: 0 },DistinguishedValue { name: "fog".into(), value: 1 },DistinguishedValue { name: "smoke".into(), value: 2 },DistinguishedValue { name: "heavySnowfall".into(), value: 3 },DistinguishedValue { name: "heavyRain".into(), value: 4 },DistinguishedValue { name: "heavyHail".into(), value: 5 },DistinguishedValue { name: "lowSunGlare".into(), value: 6 },DistinguishedValue { name: "sandstorms".into(), value: 7 },DistinguishedValue { name: "swarmsOfInsects".into(), value: 8 }]) }, input)
            .map(|(remaining, res)| (remaining, Self(res)))
    }
}


/* *
 * This DE represents the air humidity in tenths of percent.
 *
 * The value shall be set to:
 * - `n` (`n > 0` and `n < 1001`) indicates that the applicable value is equal to or less than n x 0,1 percent and greater than (n-1) x 0,1 percent.
 * - `1001` indicates that the air humidity is unavailable.
 *
 * @category: Basic information
 * @unit: 0,1 % 
 * @revision: created in V2.1.1
 */
#[derive(Debug, Clone, PartialEq)]
pub struct AirHumidity(pub u16);

impl Decode for AirHumidity {
    fn decode<'a, D>(decoder: D, input: &'a [u8]) -> nom::IResult<&'a [u8], Self>
    where
        D: asnr_transcoder::Decoder,
        Self: Sized,
    {
        decoder
            .decode_integer(AsnInteger { constraint: Some(Constraint { min_value: Some(1), max_value: Some(1001), extensible: false }), distinguished_values: Some(vec![DistinguishedValue { name: "oneHundredPercent".into(), value: 1000 },DistinguishedValue { name: "unavailable".into(), value: 1001 }]) }, input)
            .map(|(remaining, res)| (remaining, Self(res)))
    }
}


/* *
 * This DE indicates the altitude confidence value which represents the estimated absolute accuracy of an altitude value of a geographical point with a default confidence level of 95 %.
 * If required, the confidence level can be defined by the corresponding standards applying this DE.
 *
 * The value shall be set to: 
 *   - 0  - `alt-000-01`   - if the confidence value is equal to or less than 0,01 metre,
 *   - 1  - `alt-000-02`   - if the confidence value is equal to or less than 0,02 metre and greater than 0,01 metre,
 *   - 2  - `alt-000-05`   - if the confidence value is equal to or less than 0,05 metre and greater than 0,02 metre,            
 *   - 3  - `alt-000-10`   - if the confidence value is equal to or less than 0,1 metre and greater than 0,05 metre,            
 *   - 4  - `alt-000-20`   - if the confidence value is equal to or less than 0,2 metre and greater than 0,1 metre,            
 *   - 5  - `alt-000-50`   - if the confidence value is equal to or less than 0,5 metre and greater than 0,2 metre,             
 *   - 6  - `alt-001-00`   - if the confidence value is equal to or less than 1 metre and greater than 0,5 metre,             
 *   - 7  - `alt-002-00`   - if the confidence value is equal to or less than 2 metres and greater than 1 metre,             
 *   - 8  - `alt-005-00`   - if the confidence value is equal to or less than 5 metres and greater than 2 metres,              
 *   - 9  - `alt-010-00`   - if the confidence value is equal to or less than 10 metres and greater than 5 metres,             
 *   - 10 - `alt-020-00`   - if the confidence value is equal to or less than 20 metres and greater than 10 metres,            
 *   - 11 - `alt-050-00`   - if the confidence value is equal to or less than 50 metres and greater than 20 metres,            
 *   - 12 - `alt-100-00`   - if the confidence value is equal to or less than 100 metres and greater than 50 metres,           
 *   - 13 - `alt-200-00`   - if the confidence value is equal to or less than 200 metres and greater than 100 metres,           
 *   - 14 - `outOfRange`   - if the confidence value is out of range, i.e. greater than 200 metres,
 *   - 15 - `unavailable`  - if the confidence value is unavailable.       
 *
 * @note: The fact that an altitude value is received with confidence value set to `unavailable(15)` can be caused
 * by several reasons, such as:
 * - the sensor cannot deliver the accuracy at the defined confidence level because it is a low-end sensor,
 * - the sensor cannot calculate the accuracy due to lack of variables, or
 * - there has been a vehicle bus (e.g. CAN bus) error.
 * In all 3 cases above, the altitude value may be valid and used by the application.
 * 
 * @note: If an altitude value is received and its confidence value is set to `outOfRange(14)`, it means that the  
 * altitude value is not valid and therefore cannot be trusted. Such value is not useful for the application.             
 *
 * @category: GeoReference information
 * @revision: Description revised in V2.1.1
 */
#[derive(Debug, Clone, PartialEq)]
pub enum AltitudeConfidence {
  alt_000_01 = 0,
	alt_000_02 = 1,
	alt_000_05 = 2,
	alt_000_10 = 3,
	alt_000_20 = 4,
	alt_000_50 = 5,
	alt_001_00 = 6,
	alt_002_00 = 7,
	alt_005_00 = 8,
	alt_010_00 = 9,
	alt_020_00 = 10,
	alt_050_00 = 11,
	alt_100_00 = 12,
	alt_200_00 = 13,
	outOfRange = 14,
	unavailable = 15,
}

impl TryFrom<i128> for AltitudeConfidence {
    type Error = DecodingError;

    fn try_from(v: i128) -> Result<Self, Self::Error> {
        match v {
            x if x == Self::alt_000_01 as i128 => Ok(Self::alt_000_01),
		  x if x == Self::alt_000_02 as i128 => Ok(Self::alt_000_02),
		  x if x == Self::alt_000_05 as i128 => Ok(Self::alt_000_05),
		  x if x == Self::alt_000_10 as i128 => Ok(Self::alt_000_10),
		  x if x == Self::alt_000_20 as i128 => Ok(Self::alt_000_20),
		  x if x == Self::alt_000_50 as i128 => Ok(Self::alt_000_50),
		  x if x == Self::alt_001_00 as i128 => Ok(Self::alt_001_00),
		  x if x == Self::alt_002_00 as i128 => Ok(Self::alt_002_00),
		  x if x == Self::alt_005_00 as i128 => Ok(Self::alt_005_00),
		  x if x == Self::alt_010_00 as i128 => Ok(Self::alt_010_00),
		  x if x == Self::alt_020_00 as i128 => Ok(Self::alt_020_00),
		  x if x == Self::alt_050_00 as i128 => Ok(Self::alt_050_00),
		  x if x == Self::alt_100_00 as i128 => Ok(Self::alt_100_00),
		  x if x == Self::alt_200_00 as i128 => Ok(Self::alt_200_00),
		  x if x == Self::outOfRange as i128 => Ok(Self::outOfRange),
		  x if x == Self::unavailable as i128 => Ok(Self::unavailable),
            _ => Err(
              DecodingError::new(
                &format!("Invalid enumerated index decoding AltitudeConfidence. Received index {}",v), DecodingErrorType::InvalidEnumeratedIndex
              )
            ),
        }
    }
}

impl Decode for AltitudeConfidence {
    fn decode<'a, D>(decoder: D, input: &'a [u8]) -> nom::IResult<&'a [u8], Self>
    where
        D: asnr_transcoder::Decoder,
        Self: Sized,
    {
        decoder.decode_enumerated(
          AsnEnumerated { members: vec![Enumeral { name: "alt-000-01".into(), description: None, index: 0 },Enumeral { name: "alt-000-02".into(), description: None, index: 1 },Enumeral { name: "alt-000-05".into(), description: None, index: 2 },Enumeral { name: "alt-000-10".into(), description: None, index: 3 },Enumeral { name: "alt-000-20".into(), description: None, index: 4 },Enumeral { name: "alt-000-50".into(), description: None, index: 5 },Enumeral { name: "alt-001-00".into(), description: None, index: 6 },Enumeral { name: "alt-002-00".into(), description: None, index: 7 },Enumeral { name: "alt-005-00".into(), description: None, index: 8 },Enumeral { name: "alt-010-00".into(), description: None, index: 9 },Enumeral { name: "alt-020-00".into(), description: None, index: 10 },Enumeral { name: "alt-050-00".into(), description: None, index: 11 },Enumeral { name: "alt-100-00".into(), description: None, index: 12 },Enumeral { name: "alt-200-00".into(), description: None, index: 13 },Enumeral { name: "outOfRange".into(), description: None, index: 14 },Enumeral { name: "unavailable".into(), description: None, index: 15 }], extensible: false }, 
          input
        )
    }
}


/* *
 * This DE represents the altitude value in a WGS84 coordinate system.
 * The specific WGS84 coordinate system is specified by the corresponding standards applying this DE.
 *
 * The value shall be set to: 
 * - `-100 000` if the altitude is equal to or less than -1 000 m,
 * - `n` (`n > -100 000` and `n < 800 000`) if the altitude is equal to or less than n  x 0,01 metre and greater than (n-1) x 0,01 metre,
 * - `800 000` if the altitude  greater than 7 999,99 m,
 * - `800 001` if the information is not available.
 *
 * @note: the range of this DE does not use the full binary encoding range, but all reasonable values are covered. In order to cover all possible altitude ranges a larger encoding would be necessary.
 * @unit: 0,01 metre
 * @category: GeoReference information
 * @revision: Description revised in V2.1.1 (definition of 800 000 has slightly changed) 
 */
#[derive(Debug, Clone, PartialEq)]
pub struct AltitudeValue(pub i32);

impl Decode for AltitudeValue {
    fn decode<'a, D>(decoder: D, input: &'a [u8]) -> nom::IResult<&'a [u8], Self>
    where
        D: asnr_transcoder::Decoder,
        Self: Sized,
    {
        decoder
            .decode_integer(AsnInteger { constraint: Some(Constraint { min_value: Some(-100000), max_value: Some(800001), extensible: false }), distinguished_values: Some(vec![DistinguishedValue { name: "negativeOutOfRange".into(), value: -100000 },DistinguishedValue { name: "postiveOutOfRange".into(), value: 800000 },DistinguishedValue { name: "unavailable".into(), value: 800001 }]) }, input)
            .map(|(remaining, res)| (remaining, Self(res)))
    }
}

