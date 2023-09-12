use bitvec::{bitvec, prelude::Msb0, vec::BitVec, view::BitView};
use bitvec_nom::BSlice;

use alloc::{string::String, vec::Vec};

use crate::{
    error::{DecodingError, EncodingError},
    Decode, Encode,
};

mod decoder;
mod encoder;

pub struct Uper;

impl Uper {
    pub fn decode<'a, T: Decode<'a, BitIn<'a>>>(
        input: &'a [u8],
    ) -> Result<T, DecodingError<BitIn>> {
        T::decode::<Uper>(BitIn::from(input.view_bits::<Msb0>())).map(|(_, res)| res)
    }

    pub fn encode<'a, T: Encode<u8, BitOut>>(input: T) -> Result<Vec<u8>, EncodingError> {
        T::encode::<Uper>(input, bitvec![u8, Msb0;]).map(|mut bitvec| {
            bitvec.set_uninitialized(false);
            bitvec.into_vec()
        })
    }
}

pub type BitIn<'a> = BSlice<'a, u8, Msb0>;
pub type BitOut = BitVec<u8, Msb0>;

const RUST_KEYWORDS: [&'static str; 38] = [
    "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern",
    "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub",
    "ref", "return", "self", "Self", "static", "struct", "super", "trait", "true", "type",
    "unsafe", "use", "where", "while",
];

pub(crate) fn bit_length(min: i128, max: i128) -> usize {
    let number_of_values = max - min + 1;
    let mut power = 0;
    while number_of_values > 2_i128.pow(power) {
        power += 1;
    }
    power as usize
}

pub fn to_rust_camel_case(input: &String) -> String {
    let mut input = input.replace("-", "_");
    let input = input.drain(..).fold(String::new(), |mut acc, c| {
        if acc.is_empty() && c.is_uppercase() {
            acc.push(c.to_ascii_lowercase());
        } else if acc.ends_with(|last: char| last.is_lowercase() || last == '_') && c.is_uppercase()
        {
            acc.push('_');
            acc.push(c.to_ascii_lowercase());
        } else {
            acc.push(c);
        }
        acc
    });
    if RUST_KEYWORDS.contains(&input.as_str()) {
        String::from("r_") + &input
    } else {
        input
    }
}

pub fn to_rust_title_case(input: &String) -> String {
    let mut input = input.replace("-", "_");
    input.drain(..).fold(String::new(), |mut acc, c| {
        if acc.is_empty() && c.is_lowercase() {
            acc.push(c.to_ascii_uppercase());
        } else if acc.ends_with(|last: char| last == '_') && c.is_uppercase() {
            acc.pop();
            acc.push(c);
        } else if acc.ends_with(|last: char| last == '_') {
            acc.pop();
            acc.push(c.to_ascii_uppercase());
        } else {
            acc.push(c);
        }
        acc
    })
}

#[cfg(test)]
mod tests {
    use asnr_compiler_derive::asn1;

    use crate::uper::Uper;

    #[test]
    fn encodes_as_decodes_integer() {
        asn1!(
            r#"Int-1 ::= INTEGER
            Int-2 ::= INTEGER(42)
            Int-3 ::= INTEGER(-1..65355)
            Int-4 ::= INTEGER(23..MAX)
            Int-5 ::= INTEGER(20,...)
            Int-6 ::= INTEGER(1..24,...)"#,
            Framework::Asnr,
            crate
        );

        assert_eq!(
            42,
            Uper::decode::<Int1>(&Uper::encode(Int1(42)).unwrap())
                .unwrap()
                .0
        );
        assert_eq!(
            42,
            Uper::decode::<Int2>(&Uper::encode(Int2(42)).unwrap())
                .unwrap()
                .0
        );
        assert_eq!(
            42,
            Uper::decode::<Int3>(&Uper::encode(Int3(42)).unwrap())
                .unwrap()
                .0
        );
        assert_eq!(
            42,
            Uper::decode::<Int4>(&Uper::encode(Int4(42)).unwrap())
                .unwrap()
                .0
        );
        assert_eq!(
            87000,
            Uper::decode::<Int5>(&Uper::encode(Int5(87000)).unwrap())
                .unwrap()
                .0
        );
        assert_eq!(
            42,
            Uper::decode::<Int6>(&Uper::encode(Int6(42)).unwrap())
                .unwrap()
                .0
        );
    }

    #[test]
    fn encodes_as_decodes_bit_string() {
        asn1!(
            r#"Bit-string-1 ::= BIT STRING 
            Bit-string-2 ::= BIT STRING (SIZE(4))
            Bit-string-3 ::= BIT STRING (SIZE(1..63))
            Bit-string-4 ::= BIT STRING (SIZE(2,...))
            Bit-string-5 ::= BIT STRING (SIZE(2..24,...))"#,
            Framework::Asnr,
            crate
        );

        assert_eq!(
            vec![true],
            Uper::decode::<BitString1>(&Uper::encode(BitString1(vec![true])).unwrap())
                .unwrap()
                .0
        );
        assert_eq!(
            vec![true, false, true, false],
            Uper::decode::<BitString2>(
                &Uper::encode(BitString2(vec![true, false, true, false])).unwrap()
            )
            .unwrap()
            .0
        );
        assert_eq!(
            vec![true, false],
            Uper::decode::<BitString3>(&Uper::encode(BitString3(vec![true, false])).unwrap())
                .unwrap()
                .0
        );
        assert_eq!(
            vec![true],
            Uper::decode::<BitString4>(&Uper::encode(BitString4(vec![true])).unwrap())
                .unwrap()
                .0
        );
        assert_eq!(
            vec![true],
            Uper::decode::<BitString5>(&Uper::encode(BitString5(vec![true])).unwrap())
                .unwrap()
                .0
        );
        assert_eq!(
            vec![true, false],
            Uper::decode::<BitString5>(&Uper::encode(BitString5(vec![true, false])).unwrap())
                .unwrap()
                .0
        );
    }

    #[test]
    fn encodes_as_decodes_octet_string() {
        asn1!(
            r#"Octet-string-1 ::= OCTET STRING 
            Octet-string-2 ::= OCTET STRING (SIZE(4))
            Octet-string-3 ::= OCTET STRING (SIZE(1..63))
            Octet-string-4 ::= OCTET STRING (SIZE(2,...))
            Octet-string-5 ::= OCTET STRING (SIZE(2..24,...))"#,
            Framework::Asnr,
            crate
        );

        assert_eq!(
            vec![22],
            Uper::decode::<OctetString1>(&Uper::encode(OctetString1(vec![22])).unwrap())
                .unwrap()
                .0
        );
        assert_eq!(
            vec![22, 55, 33, 44],
            Uper::decode::<OctetString2>(
                &Uper::encode(OctetString2(vec![22, 55, 33, 44])).unwrap()
            )
            .unwrap()
            .0
        );
        assert_eq!(
            vec![33, 77],
            Uper::decode::<OctetString3>(&Uper::encode(OctetString3(vec![33, 77])).unwrap())
                .unwrap()
                .0
        );
        assert_eq!(
            vec![22],
            Uper::decode::<OctetString4>(&Uper::encode(OctetString4(vec![22])).unwrap())
                .unwrap()
                .0
        );
        assert_eq!(
            vec![22],
            Uper::decode::<OctetString5>(&Uper::encode(OctetString5(vec![22])).unwrap())
                .unwrap()
                .0
        );
        assert_eq!(
            vec![33, 44],
            Uper::decode::<OctetString5>(&Uper::encode(OctetString5(vec![33, 44])).unwrap())
                .unwrap()
                .0
        );
    }

    #[test]
    fn encodes_as_decodes_ia5string() {
        asn1!(
            r#"String-1 ::= IA5String
        String-2 ::= IA5String(SIZE(4))
        String-3 ::= IA5String(SIZE(1..63))
        String-4 ::= IA5String(SIZE(20,...))
        String-5 ::= IA5String(SIZE(1..24,...))"#,
            Framework::Asnr,
            crate
        );

        assert_eq!(
            "Hello",
            &Uper::decode::<String1>(&Uper::encode(String1("Hello".into())).unwrap())
                .unwrap()
                .0
        );
        assert_eq!(
            "WRLD",
            &Uper::decode::<String2>(&Uper::encode(String2("WRLD".into())).unwrap())
                .unwrap()
                .0
        );
        assert_eq!(
            "Hello, World!",
            &Uper::decode::<String3>(&Uper::encode(String3("Hello, World!".into())).unwrap())
                .unwrap()
                .0
        );
        assert_eq!(
            "Hello, ASN1!",
            &Uper::decode::<String4>(&Uper::encode(String4("Hello, ASN1!".into())).unwrap())
                .unwrap()
                .0
        );
        assert_eq!(
            "Hello, Abstract Syntax Notation 1!",
            &Uper::decode::<String5>(
                &Uper::encode(String5("Hello, Abstract Syntax Notation 1!".into())).unwrap()
            )
            .unwrap()
            .0
        );
    }

    #[test]
    fn encodes_as_decodes_sequence_of() {
        asn1!(
            r#"
        Sequence-of-1 ::= SEQUENCE OF Member
        Sequence-of-2 ::= SEQUENCE (SIZE(4)) OF Member
        Sequence-of-3 ::= SEQUENCE (SIZE(1..63)) OF Member
        Sequence-of-4 ::= SEQUENCE (SIZE(2,...)) OF Member
        Sequence-of-5 ::= SEQUENCE (SIZE(1..24,...)) OF Member
        Member ::= BOOLEAN"#,
            Framework::Asnr,
            crate
        );

        assert_eq!(
            vec![Member(true)],
            Uper::decode::<SequenceOf1>(&Uper::encode(SequenceOf1(vec![Member(true)])).unwrap())
                .unwrap()
                .0
        );
        assert_eq!(
            vec![Member(true), Member(true), Member(false), Member(true)],
            Uper::decode::<SequenceOf2>(
                &Uper::encode(SequenceOf2(vec![
                    Member(true),
                    Member(true),
                    Member(false),
                    Member(true)
                ]))
                .unwrap()
            )
            .unwrap()
            .0
        );
        assert_eq!(
            vec![Member(true)],
            Uper::decode::<SequenceOf3>(&Uper::encode(SequenceOf3(vec![Member(true)])).unwrap())
                .unwrap()
                .0
        );
        assert_eq!(
            vec![Member(true)],
            Uper::decode::<SequenceOf4>(&Uper::encode(SequenceOf4(vec![Member(true)])).unwrap())
                .unwrap()
                .0
        );
        assert_eq!(
            Vec::<Member>::new(),
            Uper::decode::<SequenceOf5>(&Uper::encode(SequenceOf5(vec![])).unwrap())
                .unwrap()
                .0
        );
    }

    #[test]
    fn encodes_as_decodes_sequence() {
        asn1!(
            r#"
        Seq-1 ::= SEQUENCE {
            member1 Member-1
        }

        Seq-2 ::= SEQUENCE {
            member1 Member-1,
            ...,
            ext1 Member-2
        }

        Seq-3 ::= SEQUENCE {
            member1 Member-1 DEFAULT TRUE,
            member2 Member-2 DEFAULT 0,
            member3 Member-3 OPTIONAL,
            member4 Member-4 OPTIONAL
        }

        Seq-4 ::= SEQUENCE {
            member1 Member-1 DEFAULT TRUE,
            member2 Member-2 DEFAULT 0,
            member3 Member-3 OPTIONAL,
            ...
            ext1 Member-4 OPTIONAL
        }
        
            Member-1 ::= BOOLEAN
            Member-2 ::= INTEGER(0..2)
            Member-3 ::= SEQUENCE OF Member-1
            Member-4 ::= BIT STRING (SIZE(1))"#,
            Framework::Asnr,
            crate
        );

        assert_eq!(
            Seq1 {
                member1: Member1(true)
            },
            Uper::decode::<Seq1>(
                &Uper::encode(Seq1 {
                    member1: Member1(true)
                })
                .unwrap()
            )
            .unwrap()
        );

        assert_eq!(
            Seq2 {
                member1: Member1(true),
                ext1: None,
            },
            Uper::decode::<Seq2>(
                &Uper::encode(Seq2 {
                    member1: Member1(true),
                    ext1: None,
                })
                .unwrap()
            )
            .unwrap()
        );

        assert_eq!(
            Seq2 {
                member1: Member1(true),
                ext1: Some(Member2(1)),
            },
            Uper::decode::<Seq2>(
                &Uper::encode(Seq2 {
                    member1: Member1(true),
                    ext1: Some(Member2(1)),
                })
                .unwrap()
            )
            .unwrap()
        );

        assert_eq!(
            Seq3 {
                member1: None,
                member2: Some(Member2(1)),
                member3: None,
                member4: Some(Member4(vec![false]))
            },
            Uper::decode::<Seq3>(
                &Uper::encode(Seq3 {
                    member1: None,
                    member2: Some(Member2(1)),
                    member3: None,
                    member4: Some(Member4(vec![false]))
                })
                .unwrap()
            )
            .unwrap()
        );

        assert_eq!(
            Seq3 {
                member1: Some(Member1(false)),
                member2: Some(Member2(1)),
                member3: Some(Member3(vec![Member1(true)])),
                member4: Some(Member4(vec![false]))
            },
            Uper::decode::<Seq3>(
                &Uper::encode(Seq3 {
                    member1: Some(Member1(false)),
                    member2: Some(Member2(1)),
                    member3: Some(Member3(vec![Member1(true)])),
                    member4: Some(Member4(vec![false]))
                })
                .unwrap()
            )
            .unwrap()
        );

        assert_eq!(
            Seq3 {
                member1: None,
                member2: None,
                member3: None,
                member4: None,
            },
            Uper::decode::<Seq3>(
                &Uper::encode(Seq3 {
                    member1: None,
                    member2: None,
                    member3: None,
                    member4: None,
                })
                .unwrap()
            )
            .unwrap()
        );

        assert_eq!(
            Seq4 {
                member1: None,
                member2: Some(Member2(1)),
                member3: None,
                ext1: Some(Member4(vec![false]))
            },
            Uper::decode::<Seq4>(
                &Uper::encode(Seq4 {
                    member1: None,
                    member2: Some(Member2(1)),
                    member3: None,
                    ext1: Some(Member4(vec![false]))
                })
                .unwrap()
            )
            .unwrap()
        );

        assert_eq!(
            Seq4 {
                member1: Some(Member1(false)),
                member2: Some(Member2(1)),
                member3: Some(Member3(vec![Member1(true)])),
                ext1: Some(Member4(vec![false]))
            },
            Uper::decode::<Seq4>(
                &Uper::encode(Seq4 {
                    member1: Some(Member1(false)),
                    member2: Some(Member2(1)),
                    member3: Some(Member3(vec![Member1(true)])),
                    ext1: Some(Member4(vec![false]))
                })
                .unwrap()
            )
            .unwrap()
        );

        assert_eq!(
            Seq4 {
                member1: Some(Member1(false)),
                member2: Some(Member2(1)),
                member3: Some(Member3(vec![Member1(true)])),
                ext1: None
            },
            Uper::decode::<Seq4>(
                &Uper::encode(Seq4 {
                    member1: Some(Member1(false)),
                    member2: Some(Member2(1)),
                    member3: Some(Member3(vec![Member1(true)])),
                    ext1: None
                })
                .unwrap()
            )
            .unwrap()
        );

        assert_eq!(
            Seq4 {
                member1: None,
                member2: None,
                member3: None,
                ext1: None,
            },
            Uper::decode::<Seq4>(
                &Uper::encode(Seq4 {
                    member1: None,
                    member2: None,
                    member3: None,
                    ext1: None,
                })
                .unwrap()
            )
            .unwrap()
        );
    }

    #[test]
    fn en_decodes_readme_example() {
        asn1!(
            r#"ExampleSequence ::= SEQUENCE {
            member-1 IA5String (SIZE (1..24)),
            member-2 INTEGER (0..15),
            ...,
            extension BOOLEAN OPTIONAL
          }"#,
            Framework::Asnr,
            crate
        );

        assert_eq!(
            ExampleSequence {
                member_1: InnerExampleSequenceMember1("Hello, World!".into()),
                member_2: InnerExampleSequenceMember2(8),
                extension: None
            },
            Uper::decode::<ExampleSequence>(
                &Uper::encode(ExampleSequence {
                    member_1: InnerExampleSequenceMember1("Hello, World!".into()),
                    member_2: InnerExampleSequenceMember2(8),
                    extension: None
                })
                .unwrap()
            )
            .unwrap()
        );
    }

    #[test]
    fn encodes_as_decodes_extended_sequence() {
        asn1!(
            r#"TestSequenceAsnr ::= SEQUENCE { 
            hello OCTET STRING (SIZE(0..8)),
            ...,
            world INTEGER(0..8) DEFAULT 8
          }"#,
            Framework::Asnr,
            crate
        );

        assert_eq!(
            TestSequenceAsnr {
                hello: InnerTestSequenceAsnrHello(vec![1, 2, 3, 4]),
                world: Some(InnerTestSequenceAsnrWorld(4))
            },
            Uper::decode::<TestSequenceAsnr>(
                &Uper::encode(TestSequenceAsnr {
                    hello: InnerTestSequenceAsnrHello(vec![1, 2, 3, 4]),
                    world: Some(InnerTestSequenceAsnrWorld(4))
                })
                .unwrap()
            )
            .unwrap()
        );
    }
}
