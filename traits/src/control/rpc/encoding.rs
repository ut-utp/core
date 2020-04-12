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
//! # use core::fmt::Debug;
//! trait Enc<Message: Debug> { fn encode(message: Message) -> Self; }
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
//! # use core::fmt::Debug;
//! trait Enc<Encoded: Debug> { fn encode(message: Self) -> Encoded; }
//! ```
//! This time ^ is basically just `Into`.
//!
//! Again, we do away with the unit type. Now we've got a many to one
//! relationship between a message type and encoded types, which is functionally
//! equivalent to the above, I think.
//!
//! But, again, the orphan impl rules get in our way. Users in downstream crates
//! can't, for example, write impls of `Enc` on the message types that are
//! defined in this crate (`ResponseMessage`, `RequestMessage) which
//! significantly limits how useful all of this is.
//!
//! So, ultimately, the below is what we settled on. Marker types (that
//! implementors presumably own) let us get around the orphan impl rules.
//!
//! Also note that marker types should implement Default, Copy, and Clone. And
//! also probably the other types in `core` that are derivable.
//!
//! IIRC, this ends up looking kind of sort of superficially similar to
//! `Serializer`/`Deserializer` from `serde` except `serde` is extremely clever
//! and introduces another layer (the `Serialize` and `Deserialize` traits which
//! are for unserialized types — a.k.a what we call `Message` in the above).
//! This is useful for a couple reasons; both of which you can see below:
//!
//! ```rust,ignore
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

// TODO: fix all the stuff with `Default`

// Implementors provide:
//   - T -> Encoded (Infallible)
//
// Encoding is assumed to be an infallible process so there is no associated
// error type on this trait.
pub trait Encode<Message: Debug> {
    type Encoded: Debug;

    fn encode(&mut self, message: Message) -> Self::Encoded;
}

// Implementors provide:
//   - Encoded -> T (Fallible)
//
// Decoding is assumed to be _fallible_ so this trait has an associated error
// type (`Err`).
pub trait Decode<Message: Debug> {
    type Encoded: Debug;
    type Err: Debug;

    fn decode(&mut self, encoded: &Self::Encoded) -> Result<Message, Self::Err>;
}

// In a softer world:
use core::convert::TryFrom;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct CoreConvert<T>(PhantomData<T>);

impl<T> Default for CoreConvert<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<Encoded, Message> Encode<Message> for CoreConvert<Encoded>
where
    Encoded: Debug,
    Message: Debug + Into<Encoded>,
{
    type Encoded = Encoded;
    fn encode(&mut self, message: Message) -> Self::Encoded { message.into() }
}

impl<Encoded, Message> Decode<Message> for CoreConvert<Encoded>
where
    Encoded: Debug + Clone,
    Message: Debug + TryFrom<Encoded>,
    <Message as TryFrom<Encoded>>::Error: Debug,
{
    type Encoded = Encoded;
    type Err = <Message as TryFrom<Encoded>>::Error;

    fn decode(&mut self, encoded: &Self::Encoded) -> Result<Message, Self::Err> {
        TryFrom::try_from(encoded.clone())
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
    Self: Encode<Message, Encoded = <Self as Decode<Message>>::Encoded>,
{
    // This only exists for convenience.
    // You should never implement this trait manually but if you do, you're not
    // allowed to set this associated type to anything but the below:
    // type Encoded = <Self as Encode<Message>>::Encoded;
    // type Encoded: Debug;

    // If we had GATs:
    // type Encoded: Debug where Self: Encode<Message, Encoded = <Self as Encoding<Message>>::Encoded>;

    // Same as above for this type.
    // type Err = <Self as Decode<Message>>::Err;
    // type Err: Debug;

    // These, when named encode/decode, cause inference problems and require
    // the user to use FQS!
    // fn symmetric_encode(message: Message) -> <Self as Encode<Message>>::Encoded {
    //     <Self as Encode<Message>>::encode(message)
    // }

    // fn symmetric_decode(encoded: &<Self as Decode<Message>>::Encoded)
    //         -> Result<Message, <Self as Decode<Message>>::Err> {
    //     <Self as Decode<Message>>::decode(encoded)
    // }
}

// impl<T, Message: Debug> Encoding<Message> for Pair<Message>
impl<T, Message: Debug> Encoding<Message> for T
where
    T: Decode<Message>,
    T: Encode<Message, Encoded = <Self as Decode<Message>>::Encoded>,
{
    // type Encoded = <Self as Encode<Message>>::Encoded;
    // type Err = <Self as Decode<Message>>::Err;
}

// Now some type level encoding combinators:

// First, transparent (our "base case").

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
// Essentially a type for which, `Encoded` == `Message`. Provides a no-op encode
// and a no-op decode.
pub struct Transparent<T: Debug>(PhantomData<T>);

impl<T: Debug> Default for Transparent<T> {
    fn default() -> Self {
        Transparent(PhantomData)
    }
}

impl<Message: Debug> Encode<Message> for Transparent<Message> {
    type Encoded = Message;

    fn encode(&mut self, message: Message) -> Self::Encoded {
        message
    }
}

impl<Message: Debug + Clone> Decode<Message> for Transparent<Message> {
    type Encoded = Message;
    type Err = Infallible;

    fn decode(&mut self, message: &Self::Encoded) -> Result<Message, Self::Err> {
        Ok(message.clone())
    }
}

// Next, Pair to put together a decode with its symmetric encode.

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
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
pub struct Pair<Message: Debug, Enc, Dec>
where
    Enc: Encode<Message>,
    Dec: Decode<Message, Encoded = <Enc as Encode<Message>>::Encoded>
{
    enc: Enc,
    dec: Dec,
    _m: PhantomData<Message>,
}

impl<Message: Debug, Enc, Dec> Default for Pair<Message, Enc, Dec>
where
    Enc: Default + Encode<Message>,
    Dec: Default + Decode<Message, Encoded = <Enc as Encode<Message>>::Encoded>
{
    fn default() -> Self {
        Self::with(Default::default(), Default::default())
    }
}

impl<Message: Debug, Enc, Dec> Pair<Message, Enc, Dec>
where
    Enc: Encode<Message>,
    Dec: Decode<Message, Encoded = <Enc as Encode<Message>>::Encoded>
{
    pub const fn with(enc: Enc, dec: Dec) -> Self {
        Self { enc, dec, _m: PhantomData }
    }
}


impl<Message: Debug, Enc, Dec> Encode<Message> for Pair<Message, Enc, Dec>
where
    Enc: Encode<Message>,
    Dec: Decode<Message, Encoded = <Enc as Encode<Message>>::Encoded>
{
    type Encoded = <Enc as Encode<Message>>::Encoded;

    fn encode(&mut self, message: Message) -> Self::Encoded {
        self.enc.encode(message)
        // <Enc as Encode<Message>>::encode(message)
    }
}

impl<Message: Debug, Enc, Dec> Decode<Message> for Pair<Message, Enc, Dec>
where
    Enc: Encode<Message>,
    Dec: Decode<Message, Encoded = <Enc as Encode<Message>>::Encoded>
{
    type Encoded = <Enc as Encode<Message>>::Encoded;
    type Err = <Dec as Decode<Message>>::Err;

    fn decode(&mut self, encoded: &Self::Encoded) -> Result<Message, Self::Err> {
        self.dec.decode(encoded)
        // <Dec as Decode<Message>>::decode(encoded)
    }
}

// Now, chained (where we "recurse").

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
pub struct ChainedEncode<A: Debug, B: Debug, Outer, Inner>
where
    Outer: Encode<A, Encoded = B>,
    Inner: Encode<B>
{
    outer: Outer,
    inner: Inner,
    _p: PhantomData<(A, B)>,
}

impl<A: Debug, B: Debug, Outer, Inner> Default for ChainedEncode<A, B, Outer, Inner>
where
    Outer: Default + Encode<A, Encoded = B>,
    Inner: Default + Encode<B>,
{
    fn default() -> Self {
        Self::with(Default::default(), Default::default())
    }
}

impl<A: Debug, B: Debug, Outer, Inner> ChainedEncode<A, B, Outer, Inner>
where
    Outer: Encode<A, Encoded = B>,
    Inner: Encode<B>,
{
    pub const fn with(outer: Outer, inner: Inner) -> Self {
        Self {
            outer,
            inner,
            _p: PhantomData,
        }
    }
}

impl<A, Outer> ChainedEncode<A, <Outer as Encode<A>>::Encoded, Outer, Transparent<<Outer as Encode<A>>::Encoded>>
where
    A: Debug,
    Outer: Default + Encode<A>,
{
    pub fn new_detached() -> Self { Default::default() }
}

impl<A, Outer> ChainedEncode<A, <Outer as Encode<A>>::Encoded, Outer, Transparent<<Outer as Encode<A>>::Encoded>>
where
    A: Debug,
    Outer: Encode<A>,
{
    pub const fn new(outer: Outer) -> Self {
        // Self::with(outer, Transparent::default())
        // Self {
        //     outer,
        //     inner: Transparent::default(),
        //     _p: PhantomData,
        // }
        // Not the above two so that we can be const:


        // Not using `Transparent::default()` that we can be const:
        Self::with(outer, Transparent(PhantomData))
    }
}

// impl<A: Debug, B: Debug, Outer, Inner> ChainedEncode<A, B, Outer, Inner>
// where
//     Outer: Default + Encode<A, Encoded = B>,
//     Inner: Default + Encode<B>,
// {
//     pub /*const*/ fn chain_back_detached<Z: Debug, NewOuter: Encode<Z, Encoded = A>>() -> ChainedEncode<Z, A, NewOuter, Self>
//     where
//         NewOuter: Default
//     {
//         Default::default()
//     }

//     pub fn chain_front_detatched<Z: Debug, NewInner: Encode<<Inner as Encode<B>>::Encoded, Encoded = Z>>() -> ChainedEncode<A, <Inner as Encode<B>>::Encoded, Self, NewInner>
//     where
//         NewInner: Default
//     {
//         Default::default()
//     }

//     // an alias for chain_front_detatched
//     pub fn chain_detatched<Z: Debug, NewInner: Encode<<Inner as Encode<B>>::Encoded, Encoded = Z>>() -> ChainedEncode<A, <Inner as Encode<B>>::Encoded, Self, NewInner>
//     where
//         NewInner: Default
//     {
//         Self::chain_front_detatched()
//     }
// }

impl<A: Debug, B: Debug, Outer, Inner> ChainedEncode<A, B, Outer, Inner>
where
    Outer: Default + Encode<A, Encoded = B>,
    Inner: Default + Encode<B>,
{
    pub fn chain_back_detached<Z: Debug, NewOuter: Encode<Z, Encoded = A>>() -> ChainedEncode<Z, A, NewOuter, Self>
    where
        NewOuter: Default,
    {
        Default::default()
    }

    pub fn chain_front_detatched<Z: Debug, NewInner: Encode<<Inner as Encode<B>>::Encoded, Encoded = Z>>() -> ChainedEncode<A, <Inner as Encode<B>>::Encoded, Self, NewInner>
    where
        NewInner: Default,
    {
        Default::default()
    }

    // an alias for chain_front_detatched
    pub fn chain_detatched<Z: Debug, NewInner: Encode<<Inner as Encode<B>>::Encoded, Encoded = Z>>() -> ChainedEncode<A, <Inner as Encode<B>>::Encoded, Self, NewInner>
    where
        NewInner: Default,
    {
        Self::chain_front_detatched()
    }
}

impl<A: Debug, B: Debug, Outer, Inner> ChainedEncode<A, B, Outer, Inner>
where
    Outer: Encode<A, Encoded = B>,
    Inner: Encode<B>,
{
    pub const fn chain_back<Z: Debug, NewOuter: Encode<Z, Encoded = A>>(self, new_outer: NewOuter) -> ChainedEncode<Z, A, NewOuter, Self> {
        ChainedEncode::with(new_outer, self)
        // ChainedEncode {
        //     outer: new_outer,
        //     inner: self,
        //     _p: PhantomData,
        // }
    }

    pub const fn chain_front<Z: Debug, NewInner: Encode<<Inner as Encode<B>>::Encoded, Encoded = Z>>(self, new_inner: NewInner) -> ChainedEncode<A, <Inner as Encode<B>>::Encoded, Self, NewInner> {
        ChainedEncode::with(self, new_inner)
        // ChainedEncode {
        //     outer: self,
        //     inner: new_inner,
        //     _p: PhantomData,
        // }
    }

    // an alias for chain_front
    pub const fn chain<Z: Debug, NewInner: Encode<<Inner as Encode<B>>::Encoded, Encoded = Z>>(self, new_inner: NewInner) -> ChainedEncode<A, <Inner as Encode<B>>::Encoded, Self, NewInner> {
        self.chain_front(new_inner)
    }
}

impl<A: Debug, B: Debug, Outer, Inner> Encode<A> for ChainedEncode<A, B, Outer, Inner>
where
    Outer: Encode<A, Encoded = B>,
    Inner: Encode<B>,
{
    type Encoded = <Inner as Encode<B>>::Encoded;

    fn encode(&mut self, message: A) -> Self::Encoded {
        let b: B = <Outer as Encode<A>>::encode(&mut self.outer, message);
        <Inner as Encode<B>>::encode(&mut self.inner, b)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
pub struct ChainedDecode<A: Debug, B: Debug, Outer, Inner>
where
    Inner: Decode<B>,
    Outer: Decode<A, Encoded = B>,
    <Inner as Decode<B>>::Err: Into<<Outer as Decode<A>>::Err>
{
    outer: Outer,
    inner: Inner,
    _p: PhantomData<(A, B)>,
}

impl<A: Debug, B: Debug, Outer, Inner> Default for ChainedDecode<A, B, Outer, Inner>
where
    Inner: Default + Decode<B>,
    Outer: Default + Decode<A, Encoded = B>,
    <Inner as Decode<B>>::Err: Into<<Outer as Decode<A>>::Err>,
{
    fn default() -> Self {
        Self::with(Default::default(), Default::default())
    }
}

impl<A: Debug, B: Debug, Outer, Inner> ChainedDecode<A, B, Outer, Inner>
where
    Inner: Decode<B>,
    Outer: Decode<A, Encoded = B>,
    <Inner as Decode<B>>::Err: Into<<Outer as Decode<A>>::Err>,
{
    pub const fn with(outer: Outer, inner: Inner) -> Self {
        Self {
            outer,
            inner,
            _p: PhantomData
        }
    }
}

impl<A, Outer> ChainedDecode<A, <Outer as Decode<A>>::Encoded, Outer, Transparent<<Outer as Decode<A>>::Encoded>>
where
    A: Debug,
    Outer: Default + Decode<A>,
    <Outer as Decode<A>>::Encoded: Clone, // Needed to use a transparent encoding!
    <Transparent<<Outer as Decode<A>>::Encoded> as Decode<<Outer as Decode<A>>::Encoded>>::Err: Into<<Outer as Decode<A>>::Err>,
    // Infallible: Into<<Outer as Decode<A>>::Err>,
    // !: Into<<Outer as Decode<A>>::Err>,
{
    pub fn new_detached() -> Self { Default::default() }
}

impl<A, Outer> ChainedDecode<A, <Outer as Decode<A>>::Encoded, Outer, Transparent<<Outer as Decode<A>>::Encoded>>
where
    A: Debug,
    Outer: Decode<A>,
    <Outer as Decode<A>>::Encoded: Clone, // Needed to use a transparent encoding!
    <Transparent<<Outer as Decode<A>>::Encoded> as Decode<<Outer as Decode<A>>::Encoded>>::Err: Into<<Outer as Decode<A>>::Err>,
    // Infallible: Into<<Outer as Decode<A>>::Err>,
    // !: Into<<Outer as Decode<A>>::Err>,
{
    pub const fn new(outer: Outer) -> Self {
        // Not using `Transparent::default()` that we can be const:
        Self::with(outer, Transparent(PhantomData))
    }
}

impl<A: Debug, B: Debug, Outer, Inner> ChainedDecode<A, B, Outer, Inner>
where
    Inner: Default + Decode<B>,
    Outer: Default + Decode<A, Encoded = B>,
    <Inner as Decode<B>>::Err: Into<<Outer as Decode<A>>::Err>, // These two should be
    <Outer as Decode<A>>::Err: From<<Inner as Decode<B>>::Err>, // eq, but alas.
{
    pub fn chain_back_detached<Z: Debug, NewOuter: Decode<Z, Encoded = A>>() -> ChainedDecode<Z, A, NewOuter, Self>
    where
        <Outer as Decode<A>>::Err: Into<<NewOuter as Decode<Z>>::Err>,
        NewOuter: Default,
    {
        Default::default()
    }

    pub fn chain_front_detatched<Z: Debug, NewInner>() -> ChainedDecode<A, <Inner as Decode<B>>::Encoded, Self, NewInner>
    where
        NewInner: Decode<<Inner as Decode<B>>::Encoded, Encoded = Z>,
        <Outer as Decode<A>>::Err: From<<NewInner as Decode<<Inner as Decode<B>>::Encoded>>::Err>,
        NewInner: Default,
    {
        Default::default()
    }

    // an alias for chain_front_detatched
    pub fn chain_detatched<Z: Debug, NewInner>() -> ChainedDecode<A, <Inner as Decode<B>>::Encoded, Self, NewInner>
    where
        NewInner: Decode<<Inner as Decode<B>>::Encoded, Encoded = Z>,
        <Outer as Decode<A>>::Err: From<<NewInner as Decode<<Inner as Decode<B>>::Encoded>>::Err>,
        NewInner: Default,
    {
        Self::chain_front_detatched()
    }
}


impl<A: Debug, B: Debug, Outer, Inner> ChainedDecode<A, B, Outer, Inner>
where
    Inner: Decode<B>,
    Outer: Decode<A, Encoded = B>,
    <Inner as Decode<B>>::Err: Into<<Outer as Decode<A>>::Err>, // These two should be
    <Outer as Decode<A>>::Err: From<<Inner as Decode<B>>::Err>, // eq, but alas.
{
    pub const fn chain_back<Z: Debug, NewOuter: Decode<Z, Encoded = A>>(self, new_outer: NewOuter) -> ChainedDecode<Z, A, NewOuter, Self>
    where
        <Outer as Decode<A>>::Err: Into<<NewOuter as Decode<Z>>::Err>
    {
        ChainedDecode::with(new_outer, self)
    }

    pub const fn chain_front<Z: Debug, NewInner>(self, new_inner: NewInner) -> ChainedDecode<A, <Inner as Decode<B>>::Encoded, Self, NewInner>
    where
        NewInner: Decode<<Inner as Decode<B>>::Encoded, Encoded = Z>,
        <Outer as Decode<A>>::Err: From<<NewInner as Decode<<Inner as Decode<B>>::Encoded>>::Err>,
    {
        ChainedDecode::with(self, new_inner)
    }

    // an alias for chain_front
    pub const fn chain<Z: Debug, NewInner>(self, new_inner: NewInner) -> ChainedDecode<A, <Inner as Decode<B>>::Encoded, Self, NewInner>
    where
        NewInner: Decode<<Inner as Decode<B>>::Encoded, Encoded = Z>,
        <Outer as Decode<A>>::Err: From<<NewInner as Decode<<Inner as Decode<B>>::Encoded>>::Err>,
    {
        self.chain_front(new_inner)
    }
}

impl<A: Debug, B: Debug, Outer, Inner> Decode<A> for ChainedDecode<A, B, Outer, Inner>
where
    Inner: Decode<B>,
    Outer: Decode<A, Encoded = B>,
    <Inner as Decode<B>>::Err: Into<<Outer as Decode<A>>::Err>, // These two should be
    <Outer as Decode<A>>::Err: From<<Inner as Decode<B>>::Err>, // eq, but alas.
{
    type Encoded = <Inner as Decode<B>>::Encoded;
    type Err = <Outer as Decode<A>>::Err;

    fn decode(&mut self, encoded: &Self::Encoded) -> Result<A, Self::Err> {
        let b: B = <Inner as Decode<B>>::decode(&mut self.inner, encoded)?;
        <Outer as Decode<A>>::decode(&mut self.outer, &b)
    }
}


#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
pub struct ChainedEncoding<A: Debug, B: Debug, Outer, Inner>(PhantomData<(A, Outer)>, PhantomData<(B, Inner)>)
where
    Inner: Encode<B>,
    Inner: Decode<B, Encoded = <Inner as Encode<B>>::Encoded>,
    Outer: Decode<A, Encoded = B> + Encode<A, Encoded = B>,
    <Inner as Decode<B>>::Err: Into<<Outer as Decode<A>>::Err>;

impl<A, B, Outer, Inner> Default for ChainedEncoding<A, B, Outer, Inner>
where
    A: Debug,
    B: Debug,
    Inner: Encode<B>,
    Inner: Decode<B, Encoded = <Inner as Encode<B>>::Encoded>,
    Outer: Decode<A, Encoded = B> + Encode<A, Encoded = B>,
    <Inner as Decode<B>>::Err: Into<<Outer as Decode<A>>::Err>,
{
    fn default() -> Self {
        Self(PhantomData, PhantomData)
    }
}

// TODO!!!!!!!!!!!!!!!!!!!!!!!!!!!!! this should be a coherence error
// impl<A, B, Outer, Inner> Encoding<A> for ChainedEncoding<A, B, Outer, Inner>
// where
//     A: Debug,
//     B: Debug,
//     Outer: Encoding<A, Encoded = B>,
//     Inner: Encoding<B>,
//     <Inner as Encoding<B>>::Err: Into<<Outer as Encoding<A>>::Err>
// {
//     type Encoded = <Inner as Encoding<B>>::Encoded;
//     type Err = <Outer as Encoding<A>>::Err;
// }

impl<A, B, Outer, Inner> Encode<A> for ChainedEncoding<A, B, Outer, Inner>
where
    A: Debug,
    B: Debug,
    Inner: Encode<B>,
    Inner: Decode<B, Encoded = <Inner as Encode<B>>::Encoded>,
    Outer: Decode<A, Encoded = B> + Encode<A, Encoded = B>,
    <Inner as Decode<B>>::Err: Into<<Outer as Decode<A>>::Err>,
{
    type Encoded = <Inner as Encode<B>>::Encoded;

    fn encode(message: A) -> <Inner as Encode<B>>::Encoded {
        let b: B = <Outer as Encode<A>>::encode(message);
        <Inner as Encode<B>>::encode(b)
    }
}

impl<A, B, Outer, Inner> Decode<A> for ChainedEncoding<A, B, Outer, Inner>
where
    A: Debug,
    B: Debug,
    Inner: Encode<B>,
    Inner: Decode<B, Encoded = <Inner as Encode<B>>::Encoded>,
    Outer: Decode<A, Encoded = B> + Encode<A, Encoded = B>,
    <Inner as Decode<B>>::Err: Into<<Outer as Decode<A>>::Err>,
    <Outer as Decode<A>>::Err: From<<Inner as Decode<B>>::Err>, // redundant!
{
    type Encoded = <Inner as Encode<B>>::Encoded;
    type Err = <Outer as Decode<A>>::Err;

    fn decode(message: &<Inner as Decode<B>>::Encoded) -> Result<A, <Outer as Decode<A>>::Err> {
        let b: B = <Inner as Decode<B>>::decode(message)?;
        <Outer as Decode<A>>::decode(&b)
    }
}

impl<A, Outer> ChainedEncoding<A, <Outer as Encode<A>>::Encoded, Outer, Transparent<<Outer as Encode<A>>::Encoded>>
where
    A: Debug,
    Outer: Encoding<A>,
    <Outer as Encode<A>>::Encoded: Clone, // Required by Transparent!
    <Outer as Decode<A>>::Err: From<Infallible>,
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
    Inner: Encode<B>,
    Inner: Decode<B, Encoded = <Inner as Encode<B>>::Encoded>,
    Outer: Decode<A, Encoded = B> + Encode<A, Encoded = B>,
    <Inner as Decode<B>>::Err: Into<<Outer as Decode<A>>::Err>,
{
    pub fn chain_back_detached<Z, NewOuter>() -> ChainedEncoding<Z, A, NewOuter, Self>
    where
        Z: Debug,
        NewOuter: Encode<Z, Encoded = A>,
        NewOuter: Decode<Z, Encoded = A>,
        <Outer as Decode<A>>::Err: Into<<NewOuter as Decode<Z>>::Err>,
        <Outer as Decode<A>>::Err: From<<Inner as Decode<B>>::Err>,
    {
        Default::default()
    }

    pub fn chain_back<Z, NewOuter>(self, _new_outer: NewOuter) -> ChainedEncoding<Z, A, NewOuter, Self>
    where
        Z: Debug,
        NewOuter: Encode<Z, Encoded = A>,
        NewOuter: Decode<Z, Encoded = A>,
        <Outer as Decode<A>>::Err: Into<<NewOuter as Decode<Z>>::Err>,
        <Outer as Decode<A>>::Err: From<<Inner as Decode<B>>::Err>,
    {
        Self::chain_back_detached()
    }

    pub fn chain_front_detatched<Z, NewInner>() -> ChainedEncoding<A, <Inner as Encode<B>>::Encoded, Self, NewInner>
    where
        Z: Debug,
        NewInner: Encode<<Inner as Encode<B>>::Encoded, Encoded = Z>,
        NewInner: Decode<<Inner as Encode<B>>::Encoded, Encoded = Z>,
        <Outer as Decode<A>>::Err: From<<Inner as Decode<B>>::Err>,
        <Outer as Decode<A>>::Err: From<<NewInner as Decode<<Inner as Encode<B>>::Encoded>>::Err>,
    {
        Default::default()
    }

    pub fn chain_front<Z, NewInner>(self, _new_inner: NewInner) -> ChainedEncoding<A, <Inner as Encode<B>>::Encoded, Self, NewInner>
    where
        Z: Debug,
        NewInner: Encode<<Inner as Encode<B>>::Encoded, Encoded = Z>,
        NewInner: Decode<<Inner as Encode<B>>::Encoded, Encoded = Z>,
        <Outer as Decode<A>>::Err: From<<Inner as Decode<B>>::Err>,
        <Outer as Decode<A>>::Err: From<<NewInner as Decode<<Inner as Encode<B>>::Encoded>>::Err>,
    {
        Self::chain_front_detatched()
    }

    // an alias for chain_front_detatched
    pub fn chain_detatched<Z, NewInner>() -> ChainedEncoding<A, <Inner as Encode<B>>::Encoded, Self, NewInner>
    where
        Z: Debug,
        NewInner: Encode<<Inner as Encode<B>>::Encoded, Encoded = Z>,
        NewInner: Decode<<Inner as Encode<B>>::Encoded, Encoded = Z>,
        <Outer as Decode<A>>::Err: From<<Inner as Decode<B>>::Err>,
        <Outer as Decode<A>>::Err: From<<NewInner as Decode<<Inner as Encode<B>>::Encoded>>::Err>,
    {
        Self::chain_front_detatched()
    }

    // an alias for chain_front
    pub fn chain<Z, NewInner>(self, _new_inner: NewInner) -> ChainedEncoding<A, <Inner as Encode<B>>::Encoded, Self, NewInner>
    where
        Z: Debug,
        NewInner: Encode<<Inner as Encode<B>>::Encoded, Encoded = Z>,
        NewInner: Decode<<Inner as Encode<B>>::Encoded, Encoded = Z>,
        <Outer as Decode<A>>::Err: From<<Inner as Decode<B>>::Err>,
        <Outer as Decode<A>>::Err: From<<NewInner as Decode<<Inner as Encode<B>>::Encoded>>::Err>,
    {
        Self::chain_detatched()
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

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;

    macro_rules! impl_enc_dec {
        ($unit:ident: $src:ty => $dest:ty) => {
            #[derive(Debug, Copy, Clone, Default)]
            struct $unit;

            impl Encode<$src> for $unit {
                type Encoded = $dest;
                fn encode(m: $src) -> $dest { m as $dest }
            }

            impl Decode<$src> for $unit {
                type Encoded = $dest;
                type Err = core::num::TryFromIntError;

                fn decode(e: &$dest) -> Result<$src, Self::Err> {
                    core::convert::TryInto::try_into(*e)
                }
            }
        };
    }

    impl_enc_dec!(U8ToU16: u8 => u16);
    impl_enc_dec!(U16ToU32: u16 => u32);
    impl_enc_dec!(U32ToU64: u32 => u64);
    impl_enc_dec!(U64ToU128: u64 => u128);

    #[test]
    fn transparent() {
        fn check<T: Debug + Clone + Eq>(inp: T) {
            assert_eq!(inp, Transparent::<T>::encode(inp.clone()));
            assert_eq!(inp, Transparent::<T>::decode(&inp).unwrap());
        }

        #[derive(Copy, Clone, Debug, PartialEq, Eq)]
        struct Foo { a: u8, b: u16, g: (u8, u128, u16) }

        check(8u8);
        check(8u8);
        check(Foo { a: 78, b: 900, g: (1, 2, 3) });
    }

    #[test]
    // This really does nothing at run time; if this function compiles, the
    // blanket impl works.
    fn encoding_blanket_impl() {
        fn ser<M: Debug, E: Encoding<M>>(m: M) -> <E as Encode<M>>::Encoded { E::encode(m) }
        fn de<M: Debug, E: Encoding<M>>(e: <E as Encode<M>>::Encoded) -> Result<M, <E as Decode<M>>::Err> { E::decode(&e) }


        let a: u16 = ser::<_, U8ToU16>(255u8);
        let a: u32 = ser::<_, U16ToU32>(a);
        let a: u64 = ser::<_, U32ToU64>(a);
        let a: u128 = ser::<_, U64ToU128>(a);

        let a: u64 = de::<_, U64ToU128>(a).unwrap();
        let a: u32 = de::<_, U32ToU64>(a).unwrap();
        let a: u16 = de::<_, U16ToU32>(a).unwrap();
        let a: u8 = de::<_, U8ToU16>(a).unwrap();

        assert_eq!(255u8, a);
    }

    #[test]
    // This really does nothing at run time; if this function compiles, the
    // `ChainedEncoding` types and functions work.
    fn chained_encoding() {
        fn ser<M: Debug, E: Encoding<M>>(_e: E, m: M) -> <E as Encode<M>>::Encoded { E::encode(m) }
        fn de<M: Debug, E: Encoding<M>>(_e: E, e: <E as Encode<M>>::Encoded) -> Result<M, <E as Decode<M>>::Err> { E::decode(&e) }

        let chain = ChainedEncoding::new(U8ToU16)
            .chain(U16ToU32)
            .chain(U32ToU64)
            .chain(U64ToU128);

        assert_eq!(255u128, ser(chain, 255u8));
        assert_eq!(Ok(255u8), de(chain, 255u128));

        assert_eq!(Ok(255u8), de(chain, ser(chain, 255u8)));
    }

    #[test]
    // This really does nothing at run time; if this function compiles, the
    // `ChainedEncode` types and functions work.
    fn chained_back_encode() {
        fn ser<M: Debug, E: Encode<M>>(_e: E, m: M) -> E::Encoded { E::encode(m) }

        let chain = ChainedEncode::new(U64ToU128)
            .chain_back(U32ToU64)
            .chain_back(U16ToU32)
            .chain_back(U8ToU16);

        assert_eq!(255u128, ser(chain, 255u8));
    }

    #[test]
    // This really does nothing at run time; if this function compiles, the
    // `ChainedDecode` types and functions work.
    fn chained_back_decode() {
        fn de<M: Debug, E: Decode<M>>(_e: E, e: E::Encoded) -> Result<M, E::Err> { E::decode(&e) }

        let chain = ChainedDecode::new(U64ToU128)
            .chain_back(U32ToU64)
            .chain_back(U16ToU32)
            .chain_back(U8ToU16);

        assert_eq!(Ok(255u8), de(chain, 255u128));
    }

    #[test]
    // This really does nothing at run time; if this function compiles, the
    // `ChainedEncode` types and functions work.
    fn chained_encode() {
        fn ser<M: Debug, E: Encode<M>>(_e: E, m: M) -> E::Encoded { E::encode(m) }

        let chain = ChainedEncode::new(U8ToU16)
            .chain(U16ToU32)
            .chain(U32ToU64)
            .chain(U64ToU128);

        assert_eq!(255u128, ser(chain, 255u8));
    }

    #[test]
    // This really does nothing at run time; if this function compiles, the
    // `ChainedDecode` types and functions work.
    fn chained_decode() {
        fn de<M: Debug, E: Decode<M>>(_e: E, e: E::Encoded) -> Result<M, E::Err> { E::decode(&e) }

        let chain = ChainedDecode::new(U8ToU16)
            .chain(U16ToU32)
            .chain(U32ToU64)
            .chain(U64ToU128);

        assert_eq!(Ok(255u8), de(chain, 255u128));
    }

    #[test]
    // This really does nothing at run time; if this function compiles, the
    // `Pair` types and functions work.
    fn pair() {
        // This is basically the same test as `chained_encoding` except we
        // assemble the encode pipeline and the decode pipeline ourselves.
        fn ser<M: Debug, E: Encoding<M>>(_e: E, m: M) -> <E as Encode<M>>::Encoded { E::encode(m) }
        fn de<M: Debug, E: Encoding<M>>(_e: E, e: <E as Encode<M>>::Encoded) -> Result<M, <E as Decode<M>>::Err> { E::decode(&e) }

        fn check<M: Debug + Copy + PartialEq, E: Copy + Encoding<M>>(chain: E, m: M) where <E as Decode<M>>::Err: PartialEq {
            assert_eq!(Ok(m), de(chain, ser(chain, m)));
        }

        let enc_chain = ChainedEncode::new(U8ToU16)
            .chain(U16ToU32)
            .chain(U32ToU64)
            .chain(U64ToU128);

        let dec_chain = ChainedDecode::new(U8ToU16)
            .chain(U16ToU32)
            .chain(U32ToU64)
            .chain(U64ToU128);

        let chain = Pair::with(enc_chain, dec_chain);
        check(chain, 255u8);

        // Of note is that we don't have to be perfectly symmetric here; only
        // the inputs and outputs have to be symmetric. For example:
        impl_enc_dec!(U8ToU128: u8 => u128);
        check(Pair::with(U8ToU128, U8ToU128), 254u8);
        check(U8ToU128, 253u8);

        check(Pair::with(enc_chain, U8ToU128), 252u8);
        check(Pair::with(U8ToU128, dec_chain), 251u8);
    }
}
