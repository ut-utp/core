//! Parts for control's *encoding* layer. Mainly the [`Encode`] and [`Decode`]
//! traits (and some friends).
//!
//! TODO!

//! Note: We used associated types for `Encoded` for these next two traits so
//! that there's a one to one mapping from a (message type, [unit type][^1])
//! pair to an encoded type.
//!
//! We realize that it is desirable to have many encodings for a message type
//! (i.e. `ResponseMessage` could be encoded as a String (JSON) or a byte slice
//! COBS, etc.). This is still supported with these traits; you'll just need to
//! make a different unit type (i.e. `Cobs`) to implement the other
//! `ResponseMessage` encodings. We wanted users of this machinery to be able to
//! pick the encoding/decoding by specifying a concrete type (i.e. Enc =
//! `JsonEnc`) and we felt that the 1 (message type, unit type) -> (enc type)
//! restriction was a good way to minimize confusion.
//!
//! Something of the shape `trait Enc<Message, Encoded>` would lift the above
//! restriction.
//!
//! There are other ways to do this that we also considered. One is to have the
//! `Self` type be either the `Encoded` type or the `Message` type. Each of
//! these limits what users of this API can actually do:
//!
//! For `Encoded = Self`:
//! ```rust
//! trait Enc<Message: Debug> { fn encode(message: Message) -> Self }
//! ```
//! Observant readers may notice that the above is basically just `From`.
//!
//! We do away with the unit type and now have a many to one relationship
//! between an encoded type and message types which means users will likely have
//! to specify some types manually (instead of passing in a unit struct), which
//! is not great but is okay (users will have to say, for example,
//! `<JsonString as Enc<ResponseMessage>>`).
//!
//! But more importantly, because of the orphan impl rules, users of this crate
//! won't be able to define encodings/decodings on types they did not implement
//! themselves (i.e. for `JsonString` defined in some other crate (C), I, a user
//! writing crate B that uses this crate (A) cannot write this impl:
//! `impl Enc<ResponseMessage> for JsonString` since I don't own the trait or
//! the concrete type — or any of the types in that line, actually). This is not
//! ideal.
//!
//! For `Message = Self`:
//! ```rust
//! trait Enc<Encoded: Debug> { fn encode(message: Self) -> Encoded }
//! ```
//! This time ^ is basically just `Into`.
//!
//! Again, we do away with the unit type. Now we've got a many to one
//! relationship between a message type and encoded types, which is functionally
//! equivalent to the above, I think.
//!
//! But, again, the orphan impl rules get in our way. Users in downstream crates
//! can't, for example, write impls of `Env` on the message types that are
//! defined in this crate (`ResponseMessage`, `RequestMessage) which
//! significantly limits how useful all of this is.
//!
//! So, ultimately, the below is what we settled on. Marker types (that
//! implementors presumably own) let us get around the orphan impl rules.
//!
//! IIRC, this ends up looking kind of sort of superficially similar to
//! `Serializer`/`Deserializer` from `serde` except `serde` is extremely clever
//! and introduces another layer (the `Serialize` and `Deserialize` traits which
//! are for unserialized types — a.k.a what we call `Message` in the above).
//! This is useful for a couple reasons; both of which you can see below:
//!
//! ```rust,no_run
//! // In this crate (A)
//! pub trait Enc<M: Debug> { type Encd: Debug; fn enc(m: M) -> Self::Encd }
//! pub enum ControlResponse { ... }
//!
//! // In some other crate (B), a user of A
//! struct JsonEnc;
//! impl Enc<A::ControlMessage> for JsonEnc {
//!     type Encd = SomeLibCrate::Json;
//!     ...
//! }
//!
//! // In yet another crate (C) that uses A and B
//! // Say, I want to experiment with another message format...
//! struct TinyControlResponse { ... }
//! // Json is fine as a wire format though, so I'll just use that:
//! // Note: we can actually do this and coherence doesn't get in our way.
//! impl Enc<TinyControlResponse> for JsonEnc {
//!     type Encd = SomeLibCrate::Json;
//!     ...
//! }
//!
//! // But if we tried to do:
//! impl Enc<YetAnotherCrate::SuperCoolResponseFormat> for JsonEnc {
//!     type Encd = SomeLibCrate::Json;
//!     ...
//! }
//! // Oh no! Orphan impl rules!
//!
//! // A bad workaround you can do to get around this is:
//! struct MyJsonEnc;
//! impl Enc<YetAnotherCrate::SuperCoolResponseFormat> for MyJsonEnc {
//!     type Encd = SomeLibCrate::Json;
//!     ...
//! }
//! // This is not great since it means you can no longer just say "use
//! //`JsonEnc`"; instead you've got to find the right unit type for that
//! // particular message with sucks.
//! ```
//! So, some things to notice in the above:
//!   - You totally can make your own message format. For now we're going to
//!     require that it can be mapped to `ResponseMessage` and `RequestMessage`
//!     because they have defined mapping to and from Control function calls and
//!     because we don't have a trait to enforce that arbitrary types provide
//!     this — this perhaps makes this much less interesting, but it's a start.
//!   - We aren't totally immune from orphan impl rules; you still do have to
//!     own either the encoding unit type or the message type. `serde` doesn't
//!     actually get around this either but it has a (also very clever)
//!     workaround where, if you've got a type in another crate that doesn't
//!     impl Serialize/Deserialize, you can make a copy of that type in your
//!     crate that does impl it *but* have the Serialize and Deserialize impls
//!     actually produce the type in the external crate; compile time checks to
//!     ensure that the type was copied correctly + support for getters/setters
//!     (i.e. private fields). I'm slightly befuddled by how `serde` gets around
//!     its `Deserialize` trait returning `Self`... maybe it doesn't except for
//!     things that include remote types within them (i.e. you still need a new
//!     type). More info [here](https://github.com/serde-rs/serde/pull/858).
//!   - If you create a serialization format, you need to implement it for every
//!     message format that you wish to serialize/deserialize. In other words,
//!     you need S * M impls. `serde`, because it decouples Serializers from
//!     types that can be serialized, only requires S + M impls and better yet
//!     it generates those M impls for you with a proc-macro (usually). We're
//!     not looking to make a serialization/deserialization framework so this
//!     isn't a concern for us, but it's interesting to note.
//!   - We don't enforce that all `Enc` impls on `JsonEnc` actually produce
//!     `Json` values. This _could_ be handy, but it's probably a bad idea. Note
//!     that serde's structure _does_ enforce this.
//!
//! So, why not just use `serde`?
//!
//! Well. `serde` has a pretty strict data model that, aiui, requires
//! serialization formats to support things such as maps and all of the Rust
//! primitives/structures (except unions, I guess). This is great since it means
//! that, as a writer of message types, you can (very often) use `serde` with
//! basically no extra work and that there aren't separate types for your actual
//! code and for intermediate serialization/deserialization steps.
//!
//! On the other hand, `serde`'s requirements are a lot stricter than our own
//! which closes the door to wire formats like ProtoBufs which aren't really
//! compatible with `serde`. This seems less than ideal which is why we have the
//! below traits.
//!
//! [^1]: i.e. `JsonEncoding`; a type that this trait is implemented on but is
//!       never actually used for anything.

use core::fmt::Debug;
use core::marker::PhantomData;
use core::convert::Infallible;

// Implementors provide:
//   - T -> Encoded (Infallible)
//
// Encoding is assumed to be an infallible process so there is no associated
// error type on this trait.
pub trait Encode<Message: Debug> {
    type Encoded: Debug;

    fn encode(message: Message) -> Self::Encoded;
}

// Implementors provide:
//   - Encoded -> T (Fallible)
//
// Decoding is assumed to be _fallible_ so this trait has an associated error
// type (`Err`).
pub trait Decode<Message: Debug> {
    type Encoded: Debug;
    type Err: Debug;

    fn decode(encoded: &Self::Encoded) -> Result<Message, Self::Err;
}

// In a softer world:
use core::convert::TryFrom;

pub struct CoreConvert;
impl<Encoded, Message> Encode<Message> for CoreConvert
where
    Encoded: Debug,
    Message: Debug + Into<Encoded>,
{
    type Encoded = Encoded;
    fn encode(message: Message) -> Self::Encoded { message.into() }
}

impl<Encoded, Message> Decode<Message> for CoreConvert
where
    Encoded: Debug + Clone,
    Message: Debug + TryFrom<Encoded>,
{
    type Encoded = Encoded;
    type Err = <Message as TryFrom<Encoded>>::Err;
    fn decode(encoded: &Self::Encoded) -> Result<Message, Self::Err> {
        TryFrom::try_from(encoded)
    }
}

// This trait is for symmetric encodes and decodes.
//
// Implementors provide:
//   - T -> Encoded (Infallible) using the [`Encode`] trait.
//   - Encoded -> T (Fallible) using the [`Decode`] trait.
//
// Allowing for a full cycle:
//    [T] -> [Encoded]
//     ^         |
//      \-------/
//
// Never implement this manually.
pub trait Encoding<Message: Debug>: Encode<Message> + Decode<Message>
where
    Self: Encode<Message, Encoded = <Self as Decode<Message>>::Encoded>
{
    // This only exists for convenience.
    // You should never implement this trait manually but if you do, you're not
    // allowed to set this associated type.
    type Encoded = <Self as Encode<Message>>::Encoded;

    // Same as above for this type.
    type Err = <Self as Decode<Message>>::Err;

    fn encode(message: Message) -> <Self as Encode<Message>>::Encoded {
        <Self as Encode<Message>>::encode(message)
    }

    fn decode(encoded: &<Self as Decode<Message>>::Encoded)
            -> Result<Message, <Self as Decode<Message>>::Err> {
        <Self as Decode<Message>>::decode(encoded)
    }
}

impl<Message: Debug, Encoding> EncodingPair<Message> for Encoding
where
    Self: Decode<Message>,
    Self: Encode<Message, Encoded = <Self as Decode<Message>>::Encoded>,
{ }

// Now some type level encoding combinators:

// First, transparent (our "base case").

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
// Essentially a type for which, `Encoded` == `Message`. Provides a no-op encode
// and a no-op decode.
pub struct Transparent<T: Debug>(PhantomData<T>);

impl<Message: Debug> Encode<Message> for Transparent<Message> {
    type Encoded = Message;

    fn encode(message: Message) -> Self::Encoded {
        message
    }
}

impl<Message: Debug + Clone> Decode<Message> for Transparent<Message> {
    type Encoded = Message;
    type Err = Infallible;

    fn decode(message: &Self::Encoded) -> Result<Message, Self::Err> {
        Ok(message.clone())
    }
}

// Next, Pair to put together a decode with its symmetric encode.

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
// This is for symmetric encodes and decodes.
//
// Implementors provide:
//   - T -> Encoded using the [`Encode`] trait.
//   - Encoded -> T using the [`Decode`] trait.
//
// Allowing for a full cycle:
//    [T] -> [Encoded]
//     ^         |
//      \-------/
//
// Note that this should get an impl of `Encoding` for free.
pub struct Pair<Message: Debug, Enc, Dec>(PhantomData<(Enc, Dec)>)
where
    Enc: Encode<Message>,
    Dec: Decode<Message, Encoded = <Enc as Encode<T>>::Encoded>;

impl<Message: Debug, Enc, Dec> Pair<Message, Enc, Dec>
where
    Enc: Encode<Message>,
    Dec: Decode<Message, Encoded = <Enc as Encode<T>>::Encoded>
{
    pub /*const*/ fn with(enc: Enc, dec: Dec) -> Self {
        Default::default()
    }
}


impl<Message: Debug, Enc, Dec> Encode<Message> Pair<Message, Enc, Dec>
where
    Enc: Encode<Message>,
    Dec: Decode<Message, Encoded = <Enc as Encoded<T>>::Encoded>
{
    type Encoded = Message;

    fn encode(message: Message) -> Self::Encoded {
        <Enc as Encode<Message, Encoded = Self::Encoded>>::encode(message)
    }
}

impl<Message: Debug, Enc, Dec> Decode<Message> Pair<Message, Enc, Dec>
where
    Enc: Encode<Message>,
    Dec: Decode<Message, Encoded = <Enc as Encoded<T>>::Encoded>
{
    type Encoded = Message;
    type Err = <Dec as Decode<Message>>::Err;

    fn decode(encoded: &Self::Encoded) -> Result<Message, Self::Err> {
        <Dec as Decode<B, Encoded = Self::Encoded, Err = Self::Err>>::decode(encoded)
    }
}

// Now, chained (where we "recurse").

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
// Outer provides:
//   - A -> Outer::Encoded
//
// Inner provides:
//   - B -> Inner::Encoded
//
// Assuming Outer::Encoded = B, we get:
//   [A] -> [B / Outer::Encoded] -> [Inner::Encoded]
//
// Which we can squish into one encode:
//   [A] -> [Inner::Encoded]
pub struct ChainedEncode<A: Debug, B: Debug, Outer, Inner>(PhantomData<(Outer, Inner)>)
where
    Outer: Encode<A, Encoded = B>,
    Inner: Encode<B>;

impl<A, Outer> ChainedEncode<A, <Outer as Encode<A>>::Encoded, Outer, TransparentEncoding<<Outer as Encode<A>>::Encoded>>
where
    A: Debug,
    Outer: Encode<A>,
{
    pub /*const*/ fn new_detached() -> Self { Default::default() }
    pub /*const*/ fn new(_outer: Outer) -> Self { Self::new_detached() }
}

impl<A: Debug, B: Debug, Outer, Inner> ChainedEncode<A, B, Outer, Inner>
where
    Outer: Encode<A, Encoded = B>,
    Inner: Encode<B>,
{
    pub /*const*/ fn chain_detached<Z: Debug, NewOuter: Encode<Z, Encoded = A>>() -> ChainedEncode<Z, A, NewOuter, Self> {
        Default::default()
    }

    pub /*const*/ fn chain<Z: Debug, NewOuter: Encode<Z, Encoded = A>>(self, _new_outer: NewOuter) -> ChainedEncode<Z, A, NewOuter, Self> {
        Self::chain_detached()
    }
}

impl<A: Debug, B: Debug, Outer, Inner> Encode for ChainedEncode<A, B, Outer, Inner>
where
    Outer: Encode<A, Encoded = B>,
    Inner: Encode<B>,
{
    type Encoded = <Inner as Encode<B>>::Encoded;

    fn encode(message: A) -> Self::Encoded {
        let b = <Outer as Encode<A, Encoding = B>>::encode(message);
        <Inner as Encode<B, Encoding = Self::Encoding>>::encode(b)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
// Inner provides:
//   - Inner::Encoded -> B
//
// Outer provides:
//   - Outer::Encoded -> A
//
// Assuming Outer::Encoded = B, we get:
//   [Inner::Encoded] -> [B / Outer::Encoded] -> [A]
//
// Which we can squish into one decode:
//   [Inner::Encoded] -> [A]
pub struct ChainedDecode<A: Debug, B: Debug, Outer, Inner>(PhantomData<(Outer, Inner)>)
where
    Inner: Decode<B>,
    Outer: Decode<A, Encoded = B>,
    <Inner as Decode<B>>::Err: Into<<Outer as Encoding<A>>::Err;

impl<A, Outer> ChainedDecode<A, <Outer as Decode<A>>::Encoded, Outer, TransparentEncoding<<Outer as Decode<A>>::Encoded>>
where
    A: Debug,
    Outer: Decode<A>,
    <TransparentEncoding<<Outer as Decode<A>>::Encoded> as Decode<<Outer as Decode<A>>::Encoded>>::Err: Into<<Outer as Decode<A>>::Err>,
    // Infallible: Into<<Outer as Decode<A>>::Err>,
    // !: Into<<Outer as Decode<A>>::Err>,
{
    pub /*const*/ fn new_detached() -> Self { Default::default() }
    pub /*const*/ fn new(_outer: Outer) -> Self { Self::new_detached() }
}

impl<A: Debug, B: Debug, Outer, Inner> ChainedDecode<A, B, Outer, Inner>
where
    Inner: Decode<B>,
    Outer: Decode<A, Encoded = B>,
    <Inner as Decode<B>>::Err: Into<<Outer as Decode<A>>::Err>,
{
    pub /*const*/ fn chain_detached<Z: Debug, NewOuter: Decode<Z, Encoded = A>>() -> ChainedDecode<Z, A, NewOuter, Self>
    where
        <Outer as Decode<A>>::Err: Into<<NewOuter as Decode<Z>>::Err>
    {
        Default::default()
    }

    pub /*const*/ fn chain<Z: Debug, NewOuter: Decode<Z, Encoded = A>>(self, _new_outer: NewOuter) -> ChainedDecode<Z, A, NewOuter, Self>
    where
        <Outer as Decode<A>>::Err: Into<<NewOuter as Decode<Z>>::Err>
    {
        Self::chain_detached()
    }
}

impl<A: Debug, B: Debug, Outer, Inner> Encode for ChainedDecode<A, B, Outer, Inner>
where
    Inner: Decode<B>,
    Outer: Decode<A, Encoded = B>,
    <Inner as Decode<B>>::Err: Into<<Outer as Encoding<A>>::Err>,
{
    type Encoded = A;
    type Err = <Outer as Encoding<A>>::Err;

    fn decode(encoded: &Self::Encoded) -> Result<A, Self::Err> {
        let b = <Inner as Decode<B, Encoded = Self::Encoded, Err = Self::Err>>::decode(encoded)?;
        <Outer as Decode<A, Encoded = B>>::decode(b)
    }
}


#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
// This is for _chained_ symmetric encodes and decodes.
//
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
    pub fn chain_detached<Z: Debug, NewOuter: Encoding<Z, Encoded = A>>() -> ChainedEncoding<Z, A, NewOuter, Self>
    where
        <Outer as Encoding<A>>::Err: Into<<NewOuter as Encoding<Z>>::Err>
    {
        Default::default()
    }

    pub fn chain<Z: Debug, NewOuter: Encoding<Z, Encoded = A>>(self, _new_outer: NewOuter) -> ChainedEncoding<Z, A, NewOuter, Self>
    where
        <Outer as Encoding<A>>::Err: Into<<NewOuter as Encoding<Z>>::Err>
    {
        Self::chain_detached()
    }
}

using_std! {
    #[cfg(feature = "json_encoding_layer")]
    pub struct JsonEncoding;

    #[cfg(feature = "json_encoding_layer")]
    impl<Message: Debug> Encode<Message> for JsonEncoding {
        type Encoded = String;

        fn encode(message: Message) -> Self::Encoded {
            serde_json::to_string(&message).unwrap()
        }
    }

    #[cfg(feature = "json_encoding_layer")]
    impl<Message: Debug> Decode<Message> for JsonEncoding {
        type Err = serde_json::error::Error;

        fn decode(encoded: &Self::Encoded) -> Result<ControlMessage, Self::Err> {
            serde_json::from_str(encoded)
        }
    }
}
