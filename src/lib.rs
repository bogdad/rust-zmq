//! Module: zmq

#![cfg_attr(feature = "unstable", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]
#![allow(trivial_numeric_casts)]
#![cfg_attr(feature = "clippy", allow(needless_lifetimes))]

#[macro_use]
extern crate log;

extern crate libc;
extern crate zmq_sys;

use libc::{c_int, c_long, size_t, int64_t, uint64_t};
use std::ffi;
use std::fmt;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::os::raw::c_void;
use std::result;
use std::string::FromUtf8Error;
use std::{mem, ptr, str, slice};

pub use SocketType::*;

pub type Result<T> = result::Result<T, Error>;

/// Socket types
#[allow(non_camel_case_types)]
#[derive(Clone, Debug, PartialEq)]
pub enum SocketType {
    PAIR   = 0,
    PUB    = 1,
    SUB    = 2,
    REQ    = 3,
    REP    = 4,
    DEALER = 5,
    ROUTER = 6,
    PULL   = 7,
    PUSH   = 8,
    XPUB   = 9,
    XSUB   = 10,
}

impl Copy for SocketType {}

pub static DONTWAIT : i32 = 1;
pub static SNDMORE : i32 = 2;

#[allow(non_camel_case_types)]
#[derive(Clone)]
pub enum Constants {
    ZMQ_AFFINITY                 = 4,
    ZMQ_IDENTITY                 = 5,
    ZMQ_SUBSCRIBE                = 6,
    ZMQ_UNSUBSCRIBE              = 7,
    ZMQ_RATE                     = 8,
    ZMQ_RECOVERY_IVL             = 9,
    ZMQ_SNDBUF                   = 11,
    ZMQ_RCVBUF                   = 12,
    ZMQ_RCVMORE                  = 13,
    ZMQ_FD                       = 14,
    ZMQ_EVENTS                   = 15,
    ZMQ_TYPE                     = 16,
    ZMQ_LINGER                   = 17,
    ZMQ_RECONNECT_IVL            = 18,
    ZMQ_BACKLOG                  = 19,
    ZMQ_RECONNECT_IVL_MAX        = 21,
    ZMQ_MAXMSGSIZE               = 22,
    ZMQ_SNDHWM                   = 23,
    ZMQ_RCVHWM                   = 24,
    ZMQ_MULTICAST_HOPS           = 25,
    ZMQ_RCVTIMEO                 = 27,
    ZMQ_SNDTIMEO                 = 28,
    ZMQ_LAST_ENDPOINT            = 32,
    ZMQ_ROUTER_MANDATORY         = 33,
    ZMQ_TCP_KEEPALIVE            = 34,
    ZMQ_TCP_KEEPALIVE_CNT        = 35,
    ZMQ_TCP_KEEPALIVE_IDLE       = 36,
    ZMQ_TCP_KEEPALIVE_INTVL      = 37,
    ZMQ_IMMEDIATE                = 39,
    ZMQ_XPUB_VERBOSE             = 40,
    ZMQ_ROUTER_RAW               = 41,
    ZMQ_IPV6                     = 42,
    ZMQ_MECHANISM                = 43,
    ZMQ_PLAIN_SERVER             = 44,
    ZMQ_PLAIN_USERNAME           = 45,
    ZMQ_PLAIN_PASSWORD           = 46,
    ZMQ_CURVE_SERVER             = 47,
    ZMQ_CURVE_PUBLICKEY          = 48,
    ZMQ_CURVE_SECRETKEY          = 49,
    ZMQ_CURVE_SERVERKEY          = 50,
    ZMQ_PROBE_ROUTER             = 51,
    ZMQ_REQ_CORRELATE            = 52,
    ZMQ_REQ_RELAXED              = 53,
    ZMQ_CONFLATE                 = 54,
    ZMQ_ZAP_DOMAIN               = 55,
    ZMQ_ROUTER_HANDOVER          = 56,
    ZMQ_TOS                      = 57,
    ZMQ_CONNECT_RID              = 61,
    ZMQ_GSSAPI_SERVER            = 62,
    ZMQ_GSSAPI_PRINCIPAL         = 63,
    ZMQ_GSSAPI_SERVICE_PRINCIPAL = 64,
    ZMQ_GSSAPI_PLAINTEXT         = 65,
    ZMQ_HANDSHAKE_IVL            = 66,
    ZMQ_SOCKS_PROXY              = 68,
    ZMQ_XPUB_NODROP              = 69,

    ZMQ_MSG_MORE                 = 1,
    ZMQ_MSG_SHARED               = 128,
    ZMQ_MSG_MASK                 = 129,
}

impl Copy for Constants {}

impl Constants {
    pub fn to_raw(&self) -> i32 {
        *self as i32
    }

    pub fn from_raw(raw: i32) -> Option<Constants> {
        // fails if `raw` is not a valid value
        match raw {
            4   => Some(Constants::ZMQ_AFFINITY),
            5   => Some(Constants::ZMQ_IDENTITY),
            6   => Some(Constants::ZMQ_SUBSCRIBE),
            7   => Some(Constants::ZMQ_UNSUBSCRIBE),
            8   => Some(Constants::ZMQ_RATE),
            9   => Some(Constants::ZMQ_RECOVERY_IVL),
            11  => Some(Constants::ZMQ_SNDBUF),
            12  => Some(Constants::ZMQ_RCVBUF),
            13  => Some(Constants::ZMQ_RCVMORE),
            14  => Some(Constants::ZMQ_FD),
            15  => Some(Constants::ZMQ_EVENTS),
            16  => Some(Constants::ZMQ_TYPE),
            17  => Some(Constants::ZMQ_LINGER),
            18  => Some(Constants::ZMQ_RECONNECT_IVL),
            19  => Some(Constants::ZMQ_BACKLOG),
            21  => Some(Constants::ZMQ_RECONNECT_IVL_MAX),
            22  => Some(Constants::ZMQ_MAXMSGSIZE),
            23  => Some(Constants::ZMQ_SNDHWM),
            24  => Some(Constants::ZMQ_RCVHWM),
            25  => Some(Constants::ZMQ_MULTICAST_HOPS),
            27  => Some(Constants::ZMQ_RCVTIMEO),
            28  => Some(Constants::ZMQ_SNDTIMEO),
            32  => Some(Constants::ZMQ_LAST_ENDPOINT),
            33  => Some(Constants::ZMQ_ROUTER_MANDATORY),
            34  => Some(Constants::ZMQ_TCP_KEEPALIVE),
            35  => Some(Constants::ZMQ_TCP_KEEPALIVE_CNT),
            36  => Some(Constants::ZMQ_TCP_KEEPALIVE_IDLE),
            37  => Some(Constants::ZMQ_TCP_KEEPALIVE_INTVL),
            39  => Some(Constants::ZMQ_IMMEDIATE),
            40  => Some(Constants::ZMQ_XPUB_VERBOSE),
            41  => Some(Constants::ZMQ_ROUTER_RAW),
            42  => Some(Constants::ZMQ_IPV6),
            43  => Some(Constants::ZMQ_MECHANISM),
            44  => Some(Constants::ZMQ_PLAIN_SERVER),
            45  => Some(Constants::ZMQ_PLAIN_USERNAME),
            46  => Some(Constants::ZMQ_PLAIN_PASSWORD),
            47  => Some(Constants::ZMQ_CURVE_SERVER),
            48  => Some(Constants::ZMQ_CURVE_PUBLICKEY),
            49  => Some(Constants::ZMQ_CURVE_SECRETKEY),
            50  => Some(Constants::ZMQ_CURVE_SERVERKEY),
            51  => Some(Constants::ZMQ_PROBE_ROUTER),
            52  => Some(Constants::ZMQ_REQ_CORRELATE),
            53  => Some(Constants::ZMQ_REQ_RELAXED),
            54  => Some(Constants::ZMQ_CONFLATE),
            55  => Some(Constants::ZMQ_ZAP_DOMAIN),
            56  => Some(Constants::ZMQ_ROUTER_HANDOVER),
            57  => Some(Constants::ZMQ_TOS),
            61  => Some(Constants::ZMQ_CONNECT_RID),
            62  => Some(Constants::ZMQ_GSSAPI_SERVER),
            63  => Some(Constants::ZMQ_GSSAPI_PRINCIPAL),
            64  => Some(Constants::ZMQ_GSSAPI_SERVICE_PRINCIPAL),
            65  => Some(Constants::ZMQ_GSSAPI_PLAINTEXT),
            66  => Some(Constants::ZMQ_HANDSHAKE_IVL),
            68  => Some(Constants::ZMQ_SOCKS_PROXY),
            69  => Some(Constants::ZMQ_XPUB_NODROP),

            1    => Some(Constants::ZMQ_MSG_MORE),
            128  => Some(Constants::ZMQ_MSG_SHARED),
            129  => Some(Constants::ZMQ_MSG_MASK),

            _   => None,
        }
    }
}

/// Security Mechanism
#[allow(non_camel_case_types)]
#[derive(Clone, Debug, PartialEq)]
pub enum Mechanism {
    ZMQ_NULL   = 0,
    ZMQ_PLAIN  = 1,
    ZMQ_CURVE  = 2,
    ZMQ_GSSAPI = 3,
}

impl Copy for Mechanism {}

const ZMQ_HAUSNUMERO: isize = 156384712;

#[derive(Clone, Eq, PartialEq)]
pub enum Error {
    EACCES          = libc::EACCES as isize,
    EADDRINUSE      = libc::EADDRINUSE as isize,
    EAGAIN          = libc::EAGAIN as isize,
    EBUSY           = libc::EBUSY as isize,
    ECONNREFUSED    = libc::ECONNREFUSED as isize,
    EFAULT          = libc::EFAULT as isize,
    EINTR           = libc::EINTR as isize,
    EHOSTUNREACH    = libc::EHOSTUNREACH as isize,
    EINPROGRESS     = libc::EINPROGRESS as isize,
    EINVAL          = libc::EINVAL as isize,
    EMFILE          = libc::EMFILE as isize,
    EMSGSIZE        = libc::EMSGSIZE as isize,
    ENAMETOOLONG    = libc::ENAMETOOLONG as isize,
    ENODEV          = libc::ENODEV as isize,
    ENOENT          = libc::ENOENT as isize,
    ENOMEM          = libc::ENOMEM as isize,
    ENOTCONN        = libc::ENOTCONN as isize,
    ENOTSOCK        = libc::ENOTSOCK as isize,
    EPROTO          = libc::EPROTO as isize,
    EPROTONOSUPPORT = libc::EPROTONOSUPPORT as isize,
    ENOTSUP         = (ZMQ_HAUSNUMERO + 1) as isize,
    ENOBUFS         = (ZMQ_HAUSNUMERO + 3) as isize,
    ENETDOWN        = (ZMQ_HAUSNUMERO + 4) as isize,
    EADDRNOTAVAIL   = (ZMQ_HAUSNUMERO + 6) as isize,

    // native zmq error codes
    EFSM            = (ZMQ_HAUSNUMERO + 51) as isize,
    ENOCOMPATPROTO  = (ZMQ_HAUSNUMERO + 52) as isize,
    ETERM           = (ZMQ_HAUSNUMERO + 53) as isize,
    EMTHREAD        = (ZMQ_HAUSNUMERO + 54) as isize,
}

impl Copy for Error {}

impl Error {
    pub fn to_raw(&self) -> i32 {
        *self as i32
    }

    pub fn from_raw(raw: i32) -> Error {
        #![cfg_attr(feature = "clippy", allow(match_same_arms))]
        match raw {
            libc::EACCES             => Error::EACCES,
            libc::EADDRINUSE         => Error::EADDRINUSE,
            libc::EAGAIN             => Error::EAGAIN,
            libc::EBUSY              => Error::EBUSY,
            libc::ECONNREFUSED       => Error::ECONNREFUSED,
            libc::EFAULT             => Error::EFAULT,
            libc::EHOSTUNREACH       => Error::EHOSTUNREACH,
            libc::EINPROGRESS        => Error::EINPROGRESS,
            libc::EINVAL             => Error::EINVAL,
            libc::EMFILE             => Error::EMFILE,
            libc::EMSGSIZE           => Error::EMSGSIZE,
            libc::ENAMETOOLONG       => Error::ENAMETOOLONG,
            libc::ENODEV             => Error::ENODEV,
            libc::ENOENT             => Error::ENOENT,
            libc::ENOMEM             => Error::ENOMEM,
            libc::ENOTCONN           => Error::ENOTCONN,
            libc::ENOTSOCK           => Error::ENOTSOCK,
            libc::EPROTO             => Error::EPROTO,
            libc::EPROTONOSUPPORT    => Error::EPROTONOSUPPORT,
            156384713                => Error::ENOTSUP,
            156384714                => Error::EPROTONOSUPPORT,
            156384715                => Error::ENOBUFS,
            156384716                => Error::ENETDOWN,
            156384717                => Error::EADDRINUSE,
            156384718                => Error::EADDRNOTAVAIL,
            156384719                => Error::ECONNREFUSED,
            156384720                => Error::EINPROGRESS,
            156384721                => Error::ENOTSOCK,
            156384763                => Error::EFSM,
            156384764                => Error::ENOCOMPATPROTO,
            156384765                => Error::ETERM,
            156384766                => Error::EMTHREAD,

            x => {
                unsafe {
                    let s = zmq_sys::zmq_strerror(x);
                    panic!("unknown error [{}]: {}",
                        x,
                        str::from_utf8(ffi::CStr::from_ptr(s).to_bytes()).unwrap()
                    )
                }
            }
        }
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        unsafe {
            let s = zmq_sys::zmq_strerror(*self as c_int);
            let v: &'static [u8] =
                mem::transmute(ffi::CStr::from_ptr(s).to_bytes());
            str::from_utf8(v).unwrap()
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unsafe {
            let s = zmq_sys::zmq_strerror(*self as c_int);
            let v: &'static [u8] =
                mem::transmute(ffi::CStr::from_ptr(s).to_bytes());
            write!(f, "{}", str::from_utf8(v).unwrap())
        }
    }
}

// Return the current zeromq version.
pub fn version() -> (i32, i32, i32) {
    let mut major = 0;
    let mut minor = 0;
    let mut patch = 0;

    unsafe {
        zmq_sys::zmq_version(&mut major, &mut minor, &mut patch);
    }

    (major as i32, minor as i32, patch as i32)
}

/// zmq context, used to create sockets. Is thread safe, and can be safely
/// shared, but dropping it while sockets are still open will cause them to
/// close (see `zmq_ctx_destroy`(3)).
///
/// For this reason, one should use an Arc to share it, rather than any unsafe
/// trickery you might think up that would call the destructor.
pub struct Context {
    ctx: *mut c_void,
}

unsafe impl Send for Context {}
unsafe impl Sync for Context {}

impl Context {
    pub fn new() -> Context {
        Context {
            ctx: unsafe { zmq_sys::zmq_ctx_new() }
        }
    }

    pub fn socket(&mut self, socket_type: SocketType) -> Result<Socket> {
        let sock = unsafe { zmq_sys::zmq_socket(self.ctx, socket_type as c_int) };

        if sock.is_null() {
            return Err(errno_to_error());
        }

        Ok(Socket {
            sock: sock,
            owned: true,
        })
    }

    /// Try to destroy the context. This is different than the destructor; the
    /// destructor will loop when zmq_ctx_destroy returns EINTR
    pub fn destroy(&mut self) -> Result<()> {
        if unsafe { zmq_sys::zmq_ctx_destroy(self.ctx) } == -1i32 {
            Err(errno_to_error())
        } else {
            Ok(())
        }
    }
}

impl Default for Context {
    fn default() -> Self {
        Context::new()
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        debug!("context dropped");
        let mut e = self.destroy();
        while e == Err(Error::EINTR) {
            e = self.destroy();
        }
    }
}

pub struct Socket {
    sock: *mut c_void,
    owned: bool,
}

unsafe impl Send for Socket {}

impl Drop for Socket {
    fn drop(&mut self) {
        if self.owned {
            if unsafe { zmq_sys::zmq_close(self.sock) } == -1i32 {
                panic!(errno_to_error());
            } else {
                debug!("socket dropped");
            }
        }
    }
}

impl Socket {
    /// Consume the Socket and return the raw socket pointer.
    ///
    /// Failure to close the raw socket manually or call `from_raw`
    /// will lead to a memory leak.
    pub fn into_raw(mut self) -> *mut c_void {
        self.owned = false;
        self.sock
    }

    /// Create a Socket from a raw socket pointer. The Socket assumes
    /// ownership of the pointer and will close the socket when it is
    /// dropped.
    pub unsafe fn from_raw(sock: *mut c_void) -> Socket {
        Socket {
            sock: sock,
            owned: true,
        }
    }

    /// Returns the inner pointer to this Socket.
    /// **WARNING**
    /// It is your responsibility to make sure that the underlying
    /// memory is not freed too early.
    pub fn as_mut_ptr(&mut self) -> *mut c_void {
        self.sock
    }

    /// Accept connections on a socket.
    pub fn bind(&mut self, endpoint: &str) -> Result<()> {
        let c_str = ffi::CString::new(endpoint.as_bytes()).unwrap();

        let rc = unsafe {
            zmq_sys::zmq_bind(self.sock, c_str.as_ptr())
        };

        if rc == -1i32 { Err(errno_to_error()) } else { Ok(()) }
    }

    /// Stop accepting connections on a socket.
    pub fn unbind(&mut self, endpoint: &str) -> Result<()> {
        let c_str = ffi::CString::new(endpoint.as_bytes()).unwrap();

        let rc = unsafe {
            zmq_sys::zmq_unbind(self.sock, c_str.as_ptr())
        };

        if rc == -1i32 { Err(errno_to_error()) } else { Ok(()) }
    }

    /// Connect a socket.
    pub fn connect(&mut self, endpoint: &str) -> Result<()> {
        let c_str = ffi::CString::new(endpoint.as_bytes()).unwrap();

        let rc = unsafe {
            zmq_sys::zmq_connect(self.sock, c_str.as_ptr())
        };

        if rc == -1i32 { Err(errno_to_error()) } else { Ok(()) }
    }

    /// Send a `&[u8]` message.
    pub fn send(&mut self, data: &[u8], flags: i32) -> Result<()> {
        let msg = try!(Message::from_slice(data));
        self.send_msg(msg, flags)
    }

    /// Send a `Message` message.
    pub fn send_msg(&mut self, mut msg: Message, flags: i32) -> Result<()> {
        let rc = unsafe {
            zmq_sys::zmq_msg_send(&mut msg.msg, self.sock, flags as c_int)
        };

        if rc == -1i32 {
            Err(errno_to_error())
        } else {
            Ok(())
        }
    }

    pub fn send_str(&mut self, data: &str, flags: i32) -> Result<()> {
        self.send(data.as_bytes(), flags)
    }

    pub fn send_multipart(&mut self, parts: &[&[u8]], flags: i32) -> Result<()> {
        if parts.len() == 0 {
            return Ok(());
        }
        let (last_part, first_parts) = parts.split_last().unwrap();

        for part in first_parts.iter() {
            try!(self.send(part, flags | SNDMORE));
        }
        try!(self.send(last_part, flags));

        Ok(())
    }

    /// Receive a message into a `Message`. The length passed to zmq_msg_recv
    /// is the length of the buffer.
    pub fn recv(&mut self, msg: &mut Message, flags: i32) -> Result<()> {
        let rc = unsafe {
            zmq_sys::zmq_msg_recv(&mut msg.msg, self.sock, flags as c_int)
        };

        if rc == -1i32 {
            Err(errno_to_error())
        } else {
            Ok(())
        }
    }

    // Receive bytes into a slice. The length passed to zmq_recv is the length of the slice.
    pub fn recv_into(&mut self, bytes: &mut [u8], flags: i32) -> Result<()> {
        let rc = unsafe {
            let bytes_ptr = bytes.as_mut_ptr() as *mut c_void;
            zmq_sys::zmq_recv(self.sock, bytes_ptr, bytes.len(), flags as c_int)
        };

        if rc == -1i32 {
            Err(errno_to_error())
        } else {
            Ok(())
        }
    }

    pub fn recv_msg(&mut self, flags: i32) -> Result<Message> {
        let mut msg = try!(Message::new());
        match self.recv(&mut msg, flags) {
            Ok(()) => Ok(msg),
            Err(e) => Err(e),
        }
    }

    pub fn recv_bytes(&mut self, flags: i32) -> Result<Vec<u8>> {
        match self.recv_msg(flags) {
            Ok(msg) => Ok(msg.to_vec()),
            Err(e) => Err(e),
        }
    }

    /// Read a `String` from the socket.
    pub fn recv_string(&mut self, flags: i32) -> Result<result::Result<String, Vec<u8>>> {
        match self.recv_bytes(flags) {
            Ok(msg) => Ok(Ok(String::from_utf8(msg).unwrap_or("".to_string()))),
            Err(e) => Err(e),
        }
    }

    pub fn recv_multipart(&mut self, flags: i32) -> Result<Vec<Vec<u8>>> {
        let mut parts: Vec<Vec<u8>> = vec![];
        loop {
            let part = try!(self.recv_bytes(flags));
            parts.push(part);

            let more_parts = try!(self.get_rcvmore());
            if !more_parts {
                break;
            }
        }
        Ok(parts)
    }

    pub fn is_ipv6(&self) -> Result<bool> {
        Ok(try!(getsockopt_i32(self.sock, Constants::ZMQ_IPV6.to_raw())) == 1)
    }

    pub fn is_immediate(&self) -> Result<bool> {
        Ok(try!(getsockopt_i32(self.sock, Constants::ZMQ_IMMEDIATE.to_raw())) == 1)
    }

    pub fn is_plain_server(&self) -> Result<bool> {
        Ok(try!(getsockopt_i32(self.sock, Constants::ZMQ_PLAIN_SERVER.to_raw())) == 1)
    }

    #[cfg(ZMQ_HAS_CURVE = "1")]
    pub fn is_curve_server(&self) -> Result<bool> {
        Ok(try!(getsockopt_i32(self.sock, Constants::ZMQ_CURVE_SERVER.to_raw())) == 1)
    }

    #[cfg(ZMQ_HAS_GSSAPI = "1")]
    pub fn is_gssapi_server(&self) -> Result<bool> {
        Ok(try!(getsockopt_i32(self.sock, Constants::ZMQ_GSSAPI_SERVER.to_raw())) == 1)
    }

    #[cfg(ZMQ_HAS_GSSAPI = "1")]
    pub fn is_gssapi_plaintext(&self) -> Result<bool> {
        Ok(try!(getsockopt_i32(self.sock, Constants::ZMQ_GSSAPI_PLAINTEXT.to_raw())) == 1)
    }

    pub fn is_conflate(&self) -> Result<bool> {
        Ok(try!(getsockopt_i32(self.sock, Constants::ZMQ_CONFLATE.to_raw())) == 1)
    }

    pub fn get_socket_type(&self) -> Result<SocketType> {
        getsockopt_i32(self.sock, Constants::ZMQ_TYPE.to_raw()).map(|ty| {
            match ty {
                0 => SocketType::PAIR,
                1 => SocketType::PUB,
                2 => SocketType::SUB,
                3 => SocketType::REQ,
                4 => SocketType::REP,
                5 => SocketType::DEALER,
                6 => SocketType::ROUTER,
                7 => SocketType::PULL,
                8 => SocketType::PUSH,
                9 => SocketType::XPUB,
                10 => SocketType::XSUB,
                _ => panic!("socket type is out of range!")
            }
        })
    }

    pub fn get_rcvmore(&self) -> Result<bool> {
        getsockopt_i64(self.sock, Constants::ZMQ_RCVMORE.to_raw())
            .map(|o| o == 1i64 )
    }

    pub fn get_maxmsgsize(&self) -> Result<i64> {
        getsockopt_i64(self.sock, Constants::ZMQ_MAXMSGSIZE.to_raw())
    }


    pub fn get_sndhwm(&self) -> Result<i32> {
        getsockopt_i32(self.sock, Constants::ZMQ_SNDHWM.to_raw())
    }

    pub fn get_rcvhwm(&self) -> Result<i32> {
        getsockopt_i32(self.sock, Constants::ZMQ_RCVHWM.to_raw())
    }

    pub fn get_affinity(&self) -> Result<u64> {
        getsockopt_u64(self.sock, Constants::ZMQ_AFFINITY.to_raw())
    }

    pub fn get_identity(&self) -> Result<result::Result<String, Vec<u8>>> {
        // 255 = identity max length
        getsockopt_string(self.sock, Constants::ZMQ_IDENTITY.to_raw(), 255, false)
    }

    pub fn get_rate(&self) -> Result<i32> {
        getsockopt_i32(self.sock, Constants::ZMQ_RATE.to_raw())
    }

    pub fn get_recovery_ivl(&self) -> Result<i32> {
        getsockopt_i32(self.sock, Constants::ZMQ_RECOVERY_IVL.to_raw())
    }

    pub fn get_sndbuf(&self) -> Result<i32> {
        getsockopt_i32(self.sock, Constants::ZMQ_SNDBUF.to_raw())
    }

    pub fn get_rcvbuf(&self) -> Result<i32> {
        getsockopt_i32(self.sock, Constants::ZMQ_RCVBUF.to_raw())
    }

    pub fn get_tos(&self) -> Result<i32> {
        getsockopt_i32(self.sock, Constants::ZMQ_TOS.to_raw())
    }

    pub fn get_linger(&self) -> Result<i32> {
        getsockopt_i32(self.sock, Constants::ZMQ_LINGER.to_raw())
    }

    pub fn get_reconnect_ivl(&self) -> Result<i32> {
        getsockopt_i32(self.sock, Constants::ZMQ_RECONNECT_IVL.to_raw())
    }

    pub fn get_reconnect_ivl_max(&self) -> Result<i32> {
        getsockopt_i32(self.sock, Constants::ZMQ_RECONNECT_IVL_MAX.to_raw())
    }

    pub fn get_backlog(&self) -> Result<i32> {
        getsockopt_i32(self.sock, Constants::ZMQ_BACKLOG.to_raw())
    }

    pub fn get_fd(&self) -> Result<i64> {
        getsockopt_i64(self.sock, Constants::ZMQ_FD.to_raw())
    }

    pub fn get_events(&self) -> Result<i32> {
        getsockopt_i32(self.sock, Constants::ZMQ_EVENTS.to_raw())
    }

    pub fn get_multicast_hops(&self) -> Result<i32> {
        getsockopt_i32(self.sock, Constants::ZMQ_MULTICAST_HOPS.to_raw())
    }

    pub fn get_rcvtimeo(&self) -> Result<i32> {
        getsockopt_i32(self.sock, Constants::ZMQ_RCVTIMEO.to_raw())
    }

    pub fn get_sndtimeo(&self) -> Result<i32> {
        getsockopt_i32(self.sock, Constants::ZMQ_SNDTIMEO.to_raw())
    }

    pub fn get_socks_proxy(&self) -> Result<result::Result<String, Vec<u8>>> {
        // 255 = longest allowable domain name is 253 so this should
        // be a reasonable size.
        getsockopt_string(self.sock, Constants::ZMQ_SOCKS_PROXY.to_raw(), 255, true)
    }

    pub fn get_tcp_keepalive(&self) -> Result<i32> {
        getsockopt_i32(self.sock, Constants::ZMQ_TCP_KEEPALIVE.to_raw())
    }

    pub fn get_tcp_keepalive_cnt(&self) -> Result<i32> {
        getsockopt_i32(self.sock, Constants::ZMQ_TCP_KEEPALIVE_CNT.to_raw())
    }

    pub fn get_tcp_keepalive_idle(&self) -> Result<i32> {
        getsockopt_i32(self.sock, Constants::ZMQ_TCP_KEEPALIVE_IDLE.to_raw())
    }

    pub fn get_tcp_keepalive_intvl(&self) -> Result<i32> {
        getsockopt_i32(self.sock, Constants::ZMQ_TCP_KEEPALIVE_INTVL.to_raw())
    }

    pub fn get_mechanism(&self) -> Result<Mechanism> {
        getsockopt_i32(self.sock, Constants::ZMQ_MECHANISM.to_raw()).map(|mech| {
            match mech {
                0 => Mechanism::ZMQ_NULL,
                1 => Mechanism::ZMQ_PLAIN,
                2 => Mechanism::ZMQ_CURVE,
                3 => Mechanism::ZMQ_GSSAPI,
                _ => panic!("Mechanism is out of range!")
            }
        })
    }

    pub fn get_plain_username(&self) -> Result<result::Result<String, Vec<u8>>> {
        // 255 = arbitrary size
        getsockopt_string(self.sock, Constants::ZMQ_PLAIN_USERNAME.to_raw(), 255, true)
    }

    pub fn get_plain_password(&self) -> Result<result::Result<String, Vec<u8>>> {
        // 256 = arbitrary size based on std crypto key size
        getsockopt_string(self.sock, Constants::ZMQ_PLAIN_PASSWORD.to_raw(), 256, true)
    }

    pub fn get_zap_domain(&self) -> Result<result::Result<String, Vec<u8>>> {
        // 255 = arbitrary size
        getsockopt_string(self.sock, Constants::ZMQ_ZAP_DOMAIN.to_raw(), 255, true)
    }

    #[cfg(ZMQ_HAS_CURVE = "1")]
    pub fn get_curve_publickey(&self) -> Result<result::Result<String, Vec<u8>>> {
        // 41 = Z85 encoded keysize + 1 for null byte
        getsockopt_string(self.sock, Constants::ZMQ_CURVE_PUBLICKEY.to_raw(), 41, true)
    }

    #[cfg(ZMQ_HAS_CURVE = "1")]
    pub fn get_curve_secretkey(&self) -> Result<result::Result<String, Vec<u8>>> {
        // 41 = Z85 encoded keysize + 1 for null byte
        getsockopt_string(self.sock, Constants::ZMQ_CURVE_SECRETKEY.to_raw(), 41, true)
    }

    #[cfg(ZMQ_HAS_CURVE = "1")]
    pub fn get_curve_serverkey(&self) -> Result<result::Result<String, Vec<u8>>> {
        // 41 = Z85 encoded keysize + 1 for null byte
        getsockopt_string(self.sock, Constants::ZMQ_CURVE_SERVERKEY.to_raw(), 41, true)
    }

    #[cfg(ZMQ_HAS_GSSAPI = "1")]
    pub fn get_gssapi_principal(&self) -> Result<result::Result<String, Vec<u8>>> {
        // 260 = best guess of max length based on docs.
        getsockopt_string(self.sock, Constants::ZMQ_GSSAPI_PRINCIPAL.to_raw(), 260, true)
    }

    #[cfg(ZMQ_HAS_GSSAPI = "1")]
    pub fn get_gssapi_service_principal(&self) -> Result<result::Result<String, Vec<u8>>> {
        // 260 = best guess of max length based on docs.
        getsockopt_string(self.sock, Constants::ZMQ_GSSAPI_SERVICE_PRINCIPAL.to_raw(), 260, true)
    }

    pub fn get_handshake_ivl(&self) -> Result<i32> {
        getsockopt_i32(self.sock, Constants::ZMQ_HANDSHAKE_IVL.to_raw())
    }

    pub fn set_maxmsgsize(&self, value: i64) -> Result<()> {
        setsockopt_i64(self.sock, Constants::ZMQ_MAXMSGSIZE.to_raw(), value)
    }

    pub fn set_probe_router(&self, value: bool) -> Result<()> {
        let value = if value { 1i32 } else { 0i32 };
        setsockopt_i32(self.sock, Constants::ZMQ_PROBE_ROUTER.to_raw(), value)
    }

    pub fn set_router_mandatory(&self, value: bool) -> Result<()> {
        let value = if value { 1i32 } else { 0i32 };
        setsockopt_i32(self.sock, Constants::ZMQ_ROUTER_MANDATORY.to_raw(), value)
    }

    pub fn set_sndhwm(&self, value: i32) -> Result<()> {
        setsockopt_i32(self.sock, Constants::ZMQ_SNDHWM.to_raw(), value)
    }

    pub fn set_rcvhwm(&self, value: i32) -> Result<()> {
        setsockopt_i32(self.sock, Constants::ZMQ_RCVHWM.to_raw(), value)
    }

    pub fn set_affinity(&self, value: u64) -> Result<()> {
        setsockopt_u64(self.sock, Constants::ZMQ_AFFINITY.to_raw(), value)
    }

    pub fn set_identity(&self, value: &[u8]) -> Result<()> {
        setsockopt_bytes(self.sock, Constants::ZMQ_IDENTITY.to_raw(), value)
    }

    pub fn set_subscribe(&self, value: &[u8]) -> Result<()> {
        setsockopt_bytes(self.sock, Constants::ZMQ_SUBSCRIBE.to_raw(), value)
    }

    pub fn set_unsubscribe(&self, value: &[u8]) -> Result<()> {
        setsockopt_bytes(self.sock, Constants::ZMQ_UNSUBSCRIBE.to_raw(), value)
    }

    pub fn set_rate(&self, value: i32) -> Result<()> {
        setsockopt_i32(self.sock, Constants::ZMQ_RATE.to_raw(), value)
    }

    pub fn set_recovery_ivl(&self, value: i32) -> Result<()> {
        setsockopt_i32(self.sock, Constants::ZMQ_RECOVERY_IVL.to_raw(), value)
    }

    pub fn set_sndbuf(&self, value: i32) -> Result<()> {
        setsockopt_i32(self.sock, Constants::ZMQ_SNDBUF.to_raw(), value)
    }

    pub fn set_rcvbuf(&self, value: i32) -> Result<()> {
        setsockopt_i32(self.sock, Constants::ZMQ_RCVBUF.to_raw(), value)
    }

    pub fn set_tos(&self, value: i32) -> Result<()> {
        setsockopt_i32(self.sock, Constants::ZMQ_TOS.to_raw(), value)
    }

    pub fn set_linger(&self, value: i32) -> Result<()> {
        setsockopt_i32(self.sock, Constants::ZMQ_LINGER.to_raw(), value)
    }

    pub fn set_reconnect_ivl(&self, value: i32) -> Result<()> {
        setsockopt_i32(self.sock, Constants::ZMQ_RECONNECT_IVL.to_raw(), value)
    }

    pub fn set_reconnect_ivl_max(&self, value: i32) -> Result<()> {
        setsockopt_i32(
            self.sock, Constants::ZMQ_RECONNECT_IVL_MAX.to_raw(), value)
    }

    pub fn set_backlog(&self, value: i32) -> Result<()> {
        setsockopt_i32(self.sock, Constants::ZMQ_BACKLOG.to_raw(), value)
    }

    pub fn set_multicast_hops(&self, value: i32) -> Result<()> {
        setsockopt_i32(self.sock, Constants::ZMQ_MULTICAST_HOPS.to_raw(), value)
    }

    pub fn set_rcvtimeo(&self, value: i32) -> Result<()> {
        setsockopt_i32(self.sock, Constants::ZMQ_RCVTIMEO.to_raw(), value)
    }

    pub fn set_sndtimeo(&self, value: i32) -> Result<()> {
        setsockopt_i32(self.sock, Constants::ZMQ_SNDTIMEO.to_raw(), value)
    }

    pub fn set_ipv6(&self, value: bool) -> Result<()> {
        setsockopt_i32(self.sock, Constants::ZMQ_IPV6.to_raw(), if value { 1 } else { 0 })
    }

    pub fn set_socks_proxy(&self, value: Option<&str>) -> Result<()> {
        if let Some(proxy) = value {
            setsockopt_bytes(self.sock, Constants::ZMQ_SOCKS_PROXY.to_raw(), proxy.as_bytes())
        } else {
            setsockopt_null(self.sock, Constants::ZMQ_SOCKS_PROXY.to_raw())
        }
    }

    pub fn set_tcp_keepalive(&self, value: i32) -> Result<()> {
        setsockopt_i32(self.sock, Constants::ZMQ_TCP_KEEPALIVE.to_raw(), value)
    }

    pub fn set_tcp_keepalive_cnt(&self, value: i32) -> Result<()> {
        setsockopt_i32(self.sock, Constants::ZMQ_TCP_KEEPALIVE_CNT.to_raw(), value)
    }

    pub fn set_tcp_keepalive_idle(&self, value: i32) -> Result<()> {
        setsockopt_i32(self.sock, Constants::ZMQ_TCP_KEEPALIVE_IDLE.to_raw(), value)
    }

    pub fn set_tcp_keepalive_intvl(&self, value: i32) -> Result<()> {
        setsockopt_i32(self.sock, Constants::ZMQ_TCP_KEEPALIVE_INTVL.to_raw(), value)
    }

    pub fn set_immediate(&self, value: bool) -> Result<()> {
        setsockopt_i32(self.sock, Constants::ZMQ_IMMEDIATE.to_raw(), if value { 1 } else { 0 })
    }

    pub fn set_plain_server(&self, value: bool) -> Result<()> {
        setsockopt_i32(self.sock, Constants::ZMQ_PLAIN_SERVER.to_raw(), if value { 1 } else { 0 })
    }

    pub fn set_plain_username(&self, value: Option<&str>) -> Result<()> {
        if let Some(user) = value {
            setsockopt_bytes(self.sock, Constants::ZMQ_PLAIN_USERNAME.to_raw(), user.as_bytes())
        } else {
            setsockopt_null(self.sock, Constants::ZMQ_PLAIN_USERNAME.to_raw())
        }
    }

    pub fn set_plain_password(&self, value: Option<&str>) -> Result<()> {
        if let Some(user) = value {
            setsockopt_bytes(self.sock, Constants::ZMQ_PLAIN_PASSWORD.to_raw(), user.as_bytes())
        } else {
            setsockopt_null(self.sock, Constants::ZMQ_PLAIN_PASSWORD.to_raw())
        }
    }

    pub fn set_zap_domain(&self, value: &str) -> Result<()> {
        let cval = ffi::CString::new(value).unwrap();
        setsockopt_bytes(self.sock, Constants::ZMQ_ZAP_DOMAIN.to_raw(), cval.as_bytes())
    }

    #[cfg(ZMQ_HAS_CURVE = "1")]
    pub fn set_curve_server(&self, value: bool) -> Result<()> {
        setsockopt_i32(self.sock, Constants::ZMQ_CURVE_SERVER.to_raw(), if value { 1 } else { 0 })
    }

    #[cfg(ZMQ_HAS_CURVE = "1")]
    pub fn set_curve_publickey(&self, value: &str) -> Result<()> {
        setsockopt_bytes(self.sock, Constants::ZMQ_CURVE_PUBLICKEY.to_raw(), value.as_bytes())
    }

    #[cfg(ZMQ_HAS_CURVE = "1")]
    pub fn set_curve_secretkey(&self, value: &str) -> Result<()> {
        setsockopt_bytes(self.sock, Constants::ZMQ_CURVE_SECRETKEY.to_raw(), value.as_bytes())
    }

    #[cfg(ZMQ_HAS_CURVE = "1")]
    pub fn set_curve_serverkey(&self, value: &str) -> Result<()> {
        setsockopt_bytes(self.sock, Constants::ZMQ_CURVE_SERVERKEY.to_raw(), value.as_bytes())
    }

    pub fn set_conflate(&self, value: bool) -> Result<()> {
        setsockopt_i32(self.sock, Constants::ZMQ_CONFLATE.to_raw(), if value { 1 } else { 0 })
    }

    #[cfg(ZMQ_HAS_GSSAPI = "1")]
    pub fn set_gssapi_server(&self, value: bool) -> Result<()> {
        setsockopt_i32(self.sock, Constants::ZMQ_GSSAPI_SERVER.to_raw(), if value { 1 } else { 0 })
    }

    #[cfg(ZMQ_HAS_GSSAPI = "1")]
    pub fn set_gssapi_principal(&self, value: &str) -> Result<()> {
        setsockopt_bytes(self.sock, Constants::ZMQ_GSSAPI_PRINCIPAL.to_raw(), value.as_bytes())
    }

    #[cfg(ZMQ_HAS_GSSAPI = "1")]
    pub fn set_gssapi_service_principal(&self, value: &str) -> Result<()> {
        setsockopt_bytes(self.sock, Constants::ZMQ_GSSAPI_SERVICE_PRINCIPAL.to_raw(), value.as_bytes())
    }

    #[cfg(ZMQ_HAS_GSSAPI = "1")]
    pub fn set_gssapi_plaintext(&self, value: bool) -> Result<()> {
        setsockopt_i32(self.sock, Constants::ZMQ_GSSAPI_PLAINTEXT.to_raw(), if value { 1 } else { 0 })
    }

    pub fn set_handshake_ivl(&self, value: i32) -> Result<()> {
        setsockopt_i32(self.sock, Constants::ZMQ_HANDSHAKE_IVL.to_raw(), value)
    }

    pub fn as_poll_item<'a>(&'a self, events: i16) -> PollItem<'a> {
        PollItem {
            socket: self.sock,
            fd: 0,
            events: events,
            revents: 0,
            marker: PhantomData
        }
    }

    pub fn poll(&self, events: i16, timeout_ms: i64) -> Result<i32> {
        poll(&mut [self.as_poll_item(events)], timeout_ms)
    }
}

const MSG_SIZE: usize = 64;

pub struct Message {
    msg: zmq_sys::zmq_msg_t,
}

impl Drop for Message {
    fn drop(&mut self) {
        unsafe {
            let rc = zmq_sys::zmq_msg_close(&mut self.msg);
            assert_eq!(rc, 0);
        }
    }
}

impl Message {
    /// Create an empty `Message`.
    pub fn new() -> Result<Message> {
        unsafe {
            let mut msg = zmq_sys::zmq_msg_t { unnamed_field1: [0; MSG_SIZE] };
            let rc = zmq_sys::zmq_msg_init(&mut msg);

            if rc == -1i32 { return Err(errno_to_error()); }

            Ok(Message { msg: msg })
        }
    }

    /// Create a `Message` preallocated with `len` uninitialized bytes.
    pub unsafe fn with_capacity_unallocated(len: usize) -> Result<Message> {
        let mut msg = zmq_sys::zmq_msg_t { unnamed_field1: [0; MSG_SIZE] };
        let rc = zmq_sys::zmq_msg_init_size(&mut msg, len as size_t);

        if rc == -1i32 { return Err(errno_to_error()); }

        Ok(Message { msg: msg })
    }

    /// Create a `Message` with space for `len` bytes that are initialized to 0.
    pub fn with_capacity(len: usize) -> Result<Message> {
        unsafe {
            let mut msg = try!(Message::with_capacity_unallocated(len));
            ptr::write_bytes(msg.as_mut_ptr(), 0, len);
            Ok(msg)
        }
    }

    /// Create a `Message` from a `&[u8]`. This will copy `data` into the message.
    pub fn from_slice(data: &[u8]) -> Result<Message> {
        unsafe {
            let mut msg = try!(Message::with_capacity_unallocated(data.len()));
            ptr::copy_nonoverlapping(data.as_ptr(), msg.as_mut_ptr(), data.len());
            Ok(msg)
        }
    }

    pub fn as_str<'a>(&'a self) -> Option<&'a str> {
        str::from_utf8(self).ok()
    }

    pub fn gets<'a>(&'a mut self, property: &str) -> Option<&'a str> {
        let c_str = ffi::CString::new(property.as_bytes()).unwrap();

        let value = unsafe {
            zmq_sys::zmq_msg_gets(&mut self.msg, c_str.as_ptr())
        };

        if value.is_null() {
            None
        } else {
            Some(unsafe { str::from_utf8(ffi::CStr::from_ptr(value).to_bytes()).unwrap() })
        }
    }
}

impl Deref for Message {
    type Target = [u8];

    fn deref<'a>(&'a self) -> &'a [u8] {
        // This is safe because we're constraining the slice to the lifetime of
        // this message.
        unsafe {
            let ptr = self.msg.unnamed_field1.as_ptr() as *mut _;
            let data = zmq_sys::zmq_msg_data(ptr);
            let len = zmq_sys::zmq_msg_size(ptr) as usize;
            slice::from_raw_parts(mem::transmute(data), len)
        }
    }
}

impl DerefMut for Message {
    fn deref_mut<'a>(&'a mut self) -> &'a mut [u8] {
        // This is safe because we're constraining the slice to the lifetime of
        // this message.
        unsafe {
            let data = zmq_sys::zmq_msg_data(&mut self.msg);
            let len = zmq_sys::zmq_msg_size(&mut self.msg) as usize;
            slice::from_raw_parts_mut(mem::transmute(data), len)
        }
    }
}

pub static POLLIN : i16 = 1i16;
pub static POLLOUT : i16 = 2i16;
pub static POLLERR : i16 = 4i16;

#[repr(C)]
pub struct PollItem<'a> {
    socket: *mut c_void,
    fd: c_int,
    events: i16,
    revents: i16,
    marker: PhantomData<&'a Socket>
}

impl<'a> PollItem<'a> {
    pub fn from_fd(fd: c_int) -> PollItem<'a> {
        PollItem {
            socket: ptr::null_mut(),
            fd: fd,
            events: 0,
            revents: 0,
            marker: PhantomData
        }
    }

    pub fn get_revents(&self) -> i16 {
        self.revents
    }
}

pub fn poll(items: &mut [PollItem], timeout: i64) -> Result<i32,> {
    unsafe {
        let rc = zmq_sys::zmq_poll(
            items.as_mut_ptr() as *mut zmq_sys::zmq_pollitem_t,
            items.len() as c_int,
            timeout as c_long);

        if rc == -1i32 {
            Err(errno_to_error())
        } else {
            Ok(rc as i32)
        }
    }
}

pub fn proxy(frontend: &mut Socket,
             backend: &mut Socket) -> Result<()> {
    unsafe {
        let rc = zmq_sys::zmq_proxy(frontend.sock, backend.sock, ptr::null_mut());

        if rc == -1i32 {
            Err(errno_to_error())
        } else {
            Ok(())
        }
    }
}

pub fn proxy_with_capture(frontend: &mut Socket,
                          backend: &mut Socket,
                          capture: &mut Socket) -> Result<()> {
    unsafe {
        let rc = zmq_sys::zmq_proxy(frontend.sock, backend.sock, capture.sock);

        if rc == -1i32 {
            Err(errno_to_error())
        } else {
            Ok(())
        }
    }
}

pub fn has(capability: &str) -> bool {
    let c_str = ffi::CString::new(capability.as_bytes()).unwrap();

    unsafe {
        zmq_sys::zmq_has(c_str.as_ptr()) == 1
    }
}

#[cfg(ZMQ_HAS_CURVE = "1")]
pub struct CurveKeyPair {
    pub public_key: String,
    pub secret_key: String,
}

#[cfg(ZMQ_HAS_CURVE = "1")]
impl CurveKeyPair {
    pub fn new() -> Result<CurveKeyPair> {
        // Curve keypairs are currently 40 bytes long.
        let mut ffi_public_key = vec![0u8; 40];
        let mut ffi_secret_key = vec![0u8; 40];

        unsafe {
            let rc = zmq_sys::zmq_curve_keypair(
                ffi_public_key.as_mut_ptr() as *mut libc::c_char,
                ffi_secret_key.as_mut_ptr() as *mut libc::c_char);

            let public_key = String::from_utf8(ffi_public_key).expect("key not utf8");
            let secret_key = String::from_utf8(ffi_secret_key).expect("key not utf8");

            if rc == -1i32 {
                Err(errno_to_error())
            } else {
                Ok(CurveKeyPair {
                    public_key: public_key,
                    secret_key: secret_key,
                })
            }
        }
    }
}

#[derive(Debug)]
pub enum EncodeError {
    BadLength,
    FromUtf8Error(FromUtf8Error),
}

impl From<FromUtf8Error> for EncodeError {
    fn from(err: FromUtf8Error) -> Self {
        EncodeError::FromUtf8Error(err)
    }
}

impl fmt::Display for EncodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            EncodeError::BadLength => write!(f, "Invalid data length. Should be multiple of 4."),
            EncodeError::FromUtf8Error(ref e) => write!(f, "UTF8 conversion error: {}", e),
        }
    }
}

pub fn z85_encode(data: &[u8]) -> result::Result<String, EncodeError> {
    if data.len() % 4 != 0 {
        return Err(EncodeError::BadLength);
    }

    let len = data.len() * 5 / 4;
    let mut dest = vec![0u8; len];

    unsafe {
        zmq_sys::zmq_z85_encode(
            dest.as_mut_ptr() as *mut libc::c_char,
            data.as_ptr(),
            data.len());
    }

    String::from_utf8(dest).map_err(EncodeError::FromUtf8Error)
}

#[derive(Debug)]
pub enum DecodeError {
    BadLength,
    NulError(ffi::NulError),
}

impl From<ffi::NulError> for DecodeError {
    fn from(err: ffi::NulError) -> Self {
        DecodeError::NulError(err)
    }
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DecodeError::BadLength => write!(f, "Invalid data length. Should be multiple of 5."),
            DecodeError::NulError(ref e) => write!(f, "Nul byte error: {}", e),
        }
    }
}

pub fn z85_decode(data: &str) -> result::Result<Vec<u8>, DecodeError> {
    if data.len() % 5 != 0 {
        return Err(DecodeError::BadLength);
    }

    let len = data.len() * 4 / 5;
    let mut dest = vec![0u8; len];

    let c_str = try!(ffi::CString::new(data));

    unsafe {
        zmq_sys::zmq_z85_decode(dest.as_mut_ptr(), c_str.into_raw());
    }

    Ok(dest)
}

impl fmt::Debug for Error {
    /// Return the error string for an error.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unsafe {
            let s = zmq_sys::zmq_strerror(*self as c_int);
            write!(f, "{}",
                   str::from_utf8(ffi::CStr::from_ptr(s).to_bytes()).unwrap())
        }
    }
}

macro_rules! getsockopt_num(
    ($name:ident, $c_ty:ty, $ty:ty) => (
        #[allow(trivial_casts)]
        fn $name(sock: *mut c_void, opt: c_int) -> Result<$ty> {
            unsafe {
                let mut value: $c_ty = 0;
                let value_ptr = &mut value as *mut $c_ty;
                let mut size = mem::size_of::<$c_ty>() as size_t;

                let rc = zmq_sys::zmq_getsockopt(
                    sock,
                    opt,
                    value_ptr as *mut c_void,
                    &mut size);

                if rc == -1 {
                    Err(errno_to_error())
                } else {
                    Ok(value as $ty)
                }
            }
        }
    )
);

getsockopt_num!(getsockopt_i32, c_int, i32);
getsockopt_num!(getsockopt_i64, int64_t, i64);
getsockopt_num!(getsockopt_u64, uint64_t, u64);

fn getsockopt_string(sock: *mut c_void, opt: c_int, size: size_t, remove_nulbyte: bool) -> Result<result::Result<String, Vec<u8>>> {
    let mut size = size;
    let mut value = vec![0u8; size];

    let r = unsafe {
        zmq_sys::zmq_getsockopt(
            sock,
            opt,
            value.as_mut_ptr() as *mut c_void,
            &mut size)
    };

    if r == -1i32 {
        Err(errno_to_error())
    } else {
        if remove_nulbyte {
            size -= 1;
        }
        value.truncate(size);

        if let Ok(s) = str::from_utf8(&value) {
            return Ok(Ok(s.to_string()));
        }

        Ok(Err(value))
    }
}

macro_rules! setsockopt_num(
    ($name:ident, $ty:ty) => (
        #[allow(trivial_casts)]
        fn $name(sock: *mut c_void, opt: c_int, value: $ty) -> Result<()> {
            let size = mem::size_of::<$ty>() as size_t;

            let rc = unsafe {
                zmq_sys::zmq_setsockopt(
                    sock,
                    opt,
                    (&value as *const $ty) as *const c_void,
                    size)
            };

            if rc == -1 {
                Err(errno_to_error())
            } else {
                Ok(())
            }
        }
    )
);

setsockopt_num!(setsockopt_i32, i32);
setsockopt_num!(setsockopt_i64, i64);
setsockopt_num!(setsockopt_u64, u64);

fn setsockopt_bytes(sock: *mut c_void, opt: c_int, value: &[u8]) -> Result<()> {
    let r = unsafe {
        zmq_sys::zmq_setsockopt(
            sock,
            opt,
            value.as_ptr() as *const c_void,
            value.len() as size_t
        )
    };

    if r == -1i32 {
        Err(errno_to_error())
    } else {
        Ok(())
    }
}

fn setsockopt_null(sock: *mut c_void, opt: c_int) -> Result<()> {
    let r = unsafe {
        zmq_sys::zmq_setsockopt(
            sock,
            opt,
            ptr::null(),
            0
        )
    };

    if r == -1i32 {
        Err(errno_to_error())
    } else {
        Ok(())
    }
}

fn errno_to_error() -> Error {
    Error::from_raw(unsafe { zmq_sys::zmq_errno() })
}
