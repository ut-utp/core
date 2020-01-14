//! Parts for the Encoding layer. Mainly the [`Encode`] and [`Decode`] traits
//! (and some friends).
//!
//! TODO!

use core::fmt::Debug;
use core::marker::PhantomData;

// Implementors provide:
//   - T -> Encoded (Infallible)
//   - Encoded -> T (Fallible)
//
// Allowing for a full cycle:
//    [T] -> [Encoded]
//     ^         |
//      \-------/
pub trait Encoding<T: Debug> {
    type Encoded: Debug;
    type Err: Debug;

    // fn encode(message: ControlMessage) -> Result<Self::Encoded, Self::Err>;
    fn encode(message: T) -> Self::Encoded;
    fn decode(encoded: &Self::Encoded) -> Result<T, Self::Err>;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct TransparentEncoding<T: Debug>(PhantomData<T>);

impl<T: Debug> Encoding<T> for TransparentEncoding<T> {
    type Encoded = T;
    type Err = Infallible;

    fn encode(message: T) -> Self::Encoded {
        Ok(message)
    }

    fn decode(message: &Self::Encoded) -> Result<T, Self::Err> {
        Ok(message.clone())
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
// Outer provides:
//   - A -> Outer::Encoded
//   - Outer::Encoded -> A
//
// Inner provides:
//   - B -> Inner::Encoded
//   - Inner::Encoded -> B
//
// Assuming Outer::Encoded = B, we have a full cycle:
//   [A] -> [B / Outer::Encoded] -> [Inner::Encoded]
//    ^                                    |
//    \- <- [Outer::Encoded / B] <--------/
//
// The above contains two encodes and two decodes; this type squashes the above
// into one encode and one decode:
//   [A] -> [Inner::Encoded]
//    ^            |
//     \----------/
pub struct ChainedEncoding<A: Debug, B: Debug, Outer: Encoding<A>, Inner: Encoding<B>>(PhantomData<Outer>, PhantomData<Inner>)
where
    Outer: Encoding<A, Encoded = B>,
    <Inner as Encoding<B>>::Err: Into<<Outer as Encoding<A>>::Err>;


impl<A, B, Outer, Inner> Encoding<A> for ChainedEncoding<A, B, Outer, Inner>
where
    A: Debug,
    B: Debug,
    Outer: Encoding<A, Encoded = B>,
    Inner: Encoding<B>,
    <Inner as Encoding<B>>::Err: Into<<Outer as Encoding<A>>::Err>
{
    type Encoded = <Inner as Encoding<B>>::Encoded;
    type Err = <Outer as Encoding<A>>::Err;

    fn encode(message: A) -> Self::Encoded {
        let b: B = <Outer as Encoding<A>>::encode(message);
        <Inner as Encoding<B>>::encode(b)
    }

    fn decode(message: &Self::Encoded) -> Result<A, Self::Err> {
        let b: B = <Inner as Encoding<B>>::decode(message)?;
        <Outer as Encoding<A>>::decode(b)
    }
}

impl<A, Outer> ChainedEncoding<A, <Outer as Encoding<A>>::Encoded, Outer, TransparentEncoding<<Outer as Encoding<A>>::Encoded>>
where
    A: Debug,
    Outer: Encoding<A>,
    // !: Into<<Outer as Encoding<A>>::Err>,
{
    pub fn new_detached() -> Self {
        Default::default()
    }

    pub fn new(_outer: Outer) -> Self {
        Self::new_detached()
    }
}


impl<A, B, Outer, Inner> ChainedEncoding<A, B, Outer, Inner>
where
    A: Debug,
    B: Debug,
    Outer: Encoding<A, Encoded = B>,
    Inner: Encoding<B>,
    <Inner as Encoding<B>>::Err: Into<<Outer as Encoding<A>>::Err>
{
    pub fn chain_detatched<Z: Debug, NewOuter: Encoding<Z, Encoded = A>>() -> ChainedEncoding<Z, A, NewOuter, Self>
    where
        <Outer as Encoding<A>>::Err: Into<<NewOuter as Encoding<Z>>::Err>
    {
        Default::default()
    }

    pub fn chain<Z: Debug, NewOuter: Encoding<Z, Encoded = A>>(self, _new_outer: NewOuter) -> ChainedEncoding<Z, A, NewOuter, Self>
    where
        <Outer as Encoding<A>>::Err: Into<<NewOuter as Encoding<Z>>::Err>
    {
        Self::chain_detatched()
    }
}

using_std! {
    #[cfg(feature = "json_encoding")]
    pub struct JsonEncoding;

    #[cfg(feature = "json_encoding")]
    impl Encoding for JsonEncoding {
        type Encoded = String;
        type Err = serde_json::error::Error;

        fn encode(message: ControlMessage) -> Result<Self::Encoded, Self::Err> {
            serde_json::to_string(&message)
        }

        fn decode(encoded: &Self::Encoded) -> Result<ControlMessage, Self::Err> {
            serde_json::from_str(encoded)
        }
    }
}
