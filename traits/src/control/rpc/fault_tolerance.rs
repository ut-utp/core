//! Some ramblings.
//!
//! #### Normal:
//!
//! ```scala
//! host:
//!     => request(_)
//!     -> encode (i.e. ser)
//!     -> transport: send
//!
//!     ++ await(transport: recv)
//!
//! device:
//!     <= transport: recv
//!     <- decode (i.e. de)
//!     <- request(_)
//!
//!     ...
//!
//!     -> response(_)
//!     -> encode (i.e. ser)
//!     -> transport: send
//!
//! host:
//!     +< transport: recv
//!     <- decode (i.e. de)
//!     <- response(_)
//! ```
//!
//! #### Borked:
//!
//! ```scala
//! host:
//!     => request(_)
//!     -> encode [infallible]
//! %re -> send
//!
//!     ++ await::race({recv | decode}:ok | {wait(10s)}:err) .ok else(%re)
//!
//! device:
//!     <= recv
//!     <- decode
//!        *** failure! ***
//!
//!     <dropped> // This is the scenario in which we could have communicated failure to the host
//!
//! host:
//!     +< timeout!
//!        *** failure! ***
//!
//!     <retry>
//!
//!     +< send
//!
//! device:
//!     <= recv
//!     <- decode
//!     <- request(_)
//!
//!     ...
//!
//!     -> response(_)
//!     -> encode
//!     -> send
//!
//! host:
//!     +< recv
//!     +< decode
//!        *** failure! ***
//!
//!     <retry, again> // Not every request is idempotent like this...
//!
//!     +< send
//!
//! device:
//!     <= transport: recv
//!     <- decode (i.e. de)
//!     <- request(_)
//!
//!     ...
//!
//!     -> response(_)
//!     -> encode (i.e. ser)
//!     -> transport: send
//!
//! host:
//!     +< recv
//!     +< decode
//!     <- response(_)
//! ```
//!

//! Alternative option is to communicate failures but then there's potentially
//! weird asymmetry (the host wouldn't communicate recv failures to the device
//! since we expect that the device will have moved on) and since you have to
//! accommodate situations where the entire message just gets eaten, you still
//! need to have a timeout and deal with just getting nothing back.
//!
//! A potential benefit would be that if you can communicate failure the host
//! doesn't need to wait for the timeout; it could try again immediately.

//! Another thing worth noting is that the method of handling failures
//! illustrated above (when used with a length prefix message format and a
//! checksum) can deal with failure modes such as bit flips and dropped
//! *messages* but not dropped bits or bytes (or dropped *or* flipped bits in the
//! length prefix; flipped bits in the checksum will just result in the message
//! being sent again, *dropped* bits in the checksum are as disruptive as dropped
//! bits anywhere else).
//!
//! Put differently, checksum + a timeout and retry system will protect against
//! bit flips when used with a length prefix system but will *not* protect
//! against _dropped_ bits. Bit flips in the length prefix are also not protected
//! against (but the length prefix can be included in the checksum to remedy
//! this).
//!
//! Dropped bits essentially cause the transportation equivalent of a _frameshift
//! mutation_; the receiver will need to realign itself to a packet/message
//! boundary. This is very hard to do if the receiver doesn't have any way to
//! identify the start or end of a message without context (normally the
//! receiver would track the number of bytes received to know when one message
//! ends and the next begins but if bits/bytes can be dropped then all bets are
//! off; only what can be inferred with *no context* is useful).
//!
//! On the other hand sentinel based encoded schemes such as [COBS][COBS]
//! (Consistent Overhead Byte Stuffing) protect against this failure mode by
//! doing exactly what's described above: providing a way to see message
//! boundaries without context (the sentinel).
//!
//! I think there's a tradeoff between the size of the sentinel and how protected
//! you are against dropped data; large sentinels (multi-byte, for example)
//! 'steal' fewer values from the pool of values that your actual data can be
//! encoded in but increase the risk of the receiver getting data that
//! masquerades as the sentinel.
//!
//! It's also worth noting that with transport layers that have some error
//! correction built-in (like UART does, typically -- 1 parity bit), you don't
//! actually have to have your encoding protect against dropped _bits_; the level
//! of granularity is a byte, which is very helpful.
//!
//! [COBS]: (https://en.wikipedia.org/wiki/Consistent_Overhead_Byte_Stuffing)
